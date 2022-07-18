pub mod triangulation;

use crate::{camera::CameraViewPort, math::projection::Projection};

use al_core::VecData;
use al_core::{shader::ShaderBound, Texture2D, VertexArrayObject, WebGlContext};

pub trait RayTracingProjection {
    fn get_raytracer_vertex_array_object(raytracer: &RayTracer) -> &VertexArrayObject;
}
pub use triangulation::{Triangulate, Triangulation};

fn create_vertices_array<P: Projection>() -> (Vec<f32>, Vec<u16>) {
    let Triangulation { vertices, idx } = P::triangulate();

    let vertices = vertices
        .into_iter().flat_map(|pos_clip_space| {
            // Cast all the double into float
            // simple precision because this buffer
            // is sent to the GPU
            vec![pos_clip_space.x as f32, pos_clip_space.y as f32/*, pos_world_space.x as f32, pos_world_space.y as f32, pos_world_space.z as f32*/]
        })
        .collect::<Vec<_>>();

    (vertices, idx)
}

use web_sys::WebGl2RenderingContext;

pub struct RayTracer {
    vao: VertexArrayObject,
    position_tex: Texture2D,
    #[cfg(feature = "webgl1")]
    ang2pix_tex: Texture2D,
}
use cgmath::{InnerSpace, Vector2};

const SIZE_POSITION_TEX: usize = 2048;
fn generate_xyz_position<P: Projection>() -> Vec<f32> {
    let (w, h) = (SIZE_POSITION_TEX as f64, SIZE_POSITION_TEX as f64);
    let mut data = vec![];
    for y in 0..(h as u32) {
        for x in 0..(w as u32) {
            let xy = Vector2::new(x, y);
            let clip_xy = Vector2::new(
                2.0 * ((xy.x as f64) / (w as f64)) - 1.0,
                2.0 * ((xy.y as f64) / (h as f64)) - 1.0,
            );
            if let Some(pos) = P::clip_to_world_space(&clip_xy) {
                let pos = pos.truncate().normalize();
                /*let mut d: u32 = 0;
                d |= 3 << 30;
                d |= (((pos.z * 0.5 + 0.5) * (1024.0 as f64)) as u32) << 20;
                d |= (((pos.y * 0.5 + 0.5) * (1024.0 as f64)) as u32) << 10;
                d |= ((pos.x * 0.5 + 0.5) * (1024.0 as f64)) as u32;

                data.push(d);*/
                data.extend(&[pos.x as f32, pos.y as f32, pos.z as f32]);
            } else {
                data.extend(&[1.0, 1.0, 1.0]);
            }
        }
    }

    data
}
/*
fn generate_lonlat_position<P: Projection>() -> Vec<f32> {
    let (w, h) = (SIZE_POSITION_TEX as f64, SIZE_POSITION_TEX as f64);
    let mut data = vec![];
    for y in 0..(h as u32) {
        for x in 0..(w as u32) {
            let xy = Vector2::new(x, y);
            let clip_xy = Vector2::new(
                2.0 * ((xy.x as f64) / (w as f64)) - 1.0,
                2.0 * ((xy.y as f64) / (h as f64)) - 1.0,
            );
            if let Some(pos) = P::clip_to_world_space(&clip_xy) {
                let pos = pos.truncate().normalize();
                let (lon, lat) = crate::math::lonlat::xyz_to_radec::<f64>(&pos);
                /*let mut d: u32 = 0;
                d |= 3 << 30;
                d |= (((pos.z * 0.5 + 0.5) * (1024.0 as f64)) as u32) << 20;
                d |= (((pos.y * 0.5 + 0.5) * (1024.0 as f64)) as u32) << 10;
                d |= ((pos.x * 0.5 + 0.5) * (1024.0 as f64)) as u32;

                data.push(d);*/
                data.extend(&[lon.0 as f32, lat.0 as f32, 1.0]);
            } else {
                data.extend(&[1.0, 1.0, 1.0]);
            }
        }
    }

    data
}
*/
#[cfg(feature = "webgl1")]
use cgmath::Rad;
#[cfg(feature = "webgl1")]
fn generate_hash_dxdy<P: Projection>(depth: u8) -> Vec<f32> {
    let (w, h) = (SIZE_POSITION_TEX as f64, SIZE_POSITION_TEX as f64);
    let mut data = vec![];
    for y in 0..(h as u32) {
        for x in 0..(w as u32) {
            let xy = Vector2::new(x, y);
            let lonlat = LonLatT::new(
                Rad(((xy.x as f64) / (w as f64)) * std::f64::consts::PI * 2.0
                    + std::f64::consts::PI)
                .into(),
                Rad((2.0 * ((xy.y as f64) / (h as f64)) - 1.0) * std::f64::consts::FRAC_PI_2)
                    .into(),
            );
            let (idx, dx, dy) =
                cdshealpix::nested::hash_with_dxdy(depth, lonlat.lon().0, lonlat.lat().0);
            data.extend(&[(idx as f32), dx as f32, dy as f32]);
        }
    }

    data
}

fn create_f32_texture_from_raw(
    gl: &WebGlContext,
    width: i32,
    height: i32,
    data: &[f32],
) -> Texture2D {
    let tex = Texture2D::create_empty_with_format::<al_core::image::format::RGB32F>(
        gl,
        width,
        height,
        &[
            (
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST,
            ),
            (
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::NEAREST,
            ),
            // Prevents s-coordinate wrapping (repeating)
            (
                WebGl2RenderingContext::TEXTURE_WRAP_S,
                WebGl2RenderingContext::CLAMP_TO_EDGE,
            ),
            // Prevents t-coordinate wrapping (repeating)
            (
                WebGl2RenderingContext::TEXTURE_WRAP_T,
                WebGl2RenderingContext::CLAMP_TO_EDGE,
            ),
        ],
    )
    .unwrap();

    let buf_data = unsafe { js_sys::Float32Array::view(data) };
    tex.bind()
        .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
            0,
            0,
            width,
            height,
            Some(&buf_data),
        );

    tex
}
use al_api::color::Color;
impl RayTracer {
    pub fn new<P: Projection>(gl: &WebGlContext) -> RayTracer {
        let (vertices, idx) = create_vertices_array::<P>();

        let mut vao = VertexArrayObject::new(gl);
        // layout (location = 0) in vec2 pos_clip_space;
        #[cfg(feature = "webgl2")]
        vao.bind_for_update()
            .add_array_buffer(
                "vertices",
                2 * std::mem::size_of::<f32>(),
                &[2],
                &[0],
                WebGl2RenderingContext::STATIC_DRAW,
                VecData::<f32>(&vertices),
            )
            /*.add_array_buffer(
                5 * std::mem::size_of::<f32>(),
                &[2, 3],
                &[0, 2 * std::mem::size_of::<f32>()],
                WebGl2RenderingContext::STATIC_DRAW,
                VecData::<f32>(&vertices),
            )*/
            // Set the element buffer
            .add_element_buffer(WebGl2RenderingContext::STATIC_DRAW, VecData::<u16>(&idx))
            // Unbind the buffer
            .unbind();
        #[cfg(feature = "webgl1")]
        vao.bind_for_update()
            .add_array_buffer(
                2,
                "pos_clip_space",
                WebGl2RenderingContext::STATIC_DRAW,
                VecData::<f32>(&vertices),
            )
            // Set the element buffer
            .add_element_buffer(WebGl2RenderingContext::STATIC_DRAW, VecData::<u16>(&idx))
            // Unbind the buffer
            .unbind();
        // create position data
        let data = generate_xyz_position::<P>();
        let position_tex = create_f32_texture_from_raw(
            &gl,
            SIZE_POSITION_TEX as i32,
            SIZE_POSITION_TEX as i32,
            &data,
        );

        // create ang2pix texture for webgl1 app
        #[cfg(feature = "webgl1")]
        let ang2pix_tex = {
            let data = generate_hash_dxdy::<P>(0);
            create_f32_texture_from_raw(
                &gl,
                SIZE_POSITION_TEX as i32,
                SIZE_POSITION_TEX as i32,
                &data,
            )
        };

        RayTracer {
            vao,

            position_tex,
            #[cfg(feature = "webgl1")]
            ang2pix_tex,
        }
    }

    pub fn draw_font_color<'a>(&self, shader: &ShaderBound<'a>, color: &Color) {
        shader
            .attach_uniform("font_color", color)
            .bind_vertex_array_object_ref(&self.vao)
            .draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                None,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                0,
            );
    }

    pub fn draw<'a>(&self, shader: &ShaderBound<'a>) {
        #[cfg(feature = "webgl1")]
        shader
            .attach_uniform("position_tex", &self.position_tex)
            .attach_uniform("u_ang2pixd", &self.ang2pix_tex)
            .bind_vertex_array_object_ref(&self.vao)
            .draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                None,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                0,
            );
        #[cfg(feature = "webgl2")]
        shader
            .attach_uniform("position_tex", &self.position_tex)
            .bind_vertex_array_object_ref(&self.vao)
            .draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                None,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                0,
            )
    }

    pub fn is_rendering<P: Projection>(&self, camera: &CameraViewPort, depth: u8) -> bool {
        //camera.get_aperture() > P::RASTER_THRESHOLD_ANGLE
        camera.get_field_of_view().is_allsky() || depth == 0
    }
}
