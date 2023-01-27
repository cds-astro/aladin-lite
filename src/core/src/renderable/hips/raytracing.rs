use crate::{camera::CameraViewPort, math::projection::Projection};
use crate::domain::sdf::ProjDefType;

use al_core::VecData;
use al_core::{shader::ShaderBound, Texture2D, VertexArrayObject, WebGlContext};

pub use super::triangulation::Triangulation;

pub trait RayTracingProjection {
    fn get_raytracer_vertex_array_object(raytracer: &RayTracer) -> &VertexArrayObject;
}

fn create_vertices_array(proj_area: &ProjDefType) -> (Vec<f32>, Vec<u16>) {
    let Triangulation { vertices, idx } = Triangulation::build(proj_area);

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
fn generate_xyz_position(projection: &ProjectionType) -> Vec<f32> {
    let (w, h) = (SIZE_POSITION_TEX, SIZE_POSITION_TEX);
    let mut data = Vec::with_capacity(SIZE_POSITION_TEX * SIZE_POSITION_TEX * 3);
    unsafe { data.set_len(SIZE_POSITION_TEX * SIZE_POSITION_TEX * 3); }

    let mut set_pixel = |r: f32, g: f32, b: f32, x: usize, y: usize| {
        data[3*(y*w + x)] = r;
        data[3*(y*w + x) + 1] = g;
        data[3*(y*w + x) + 2] = b;
    };

    let mut t1 = 1.0;
    let mut t2 = 0.0;
    let mut t3 = 0.0;
    for y in 0..h {
        for x in 0..w {
            let xy = Vector2::new(x, y);
            let clip_xy = Vector2::new(
                2.0 * ((xy.x as f64) / (w as f64)) - 1.0,
                2.0 * ((xy.y as f64) / (h as f64)) - 1.0,
            );
            if let Some(pos) = projection.clip_to_world_space(&clip_xy) {
                let pos = pos.truncate().normalize();
                /*let mut d: u32 = 0;
                d |= 3 << 30;
                d |= (((pos.z * 0.5 + 0.5) * (1024.0 as f64)) as u32) << 20;
                d |= (((pos.y * 0.5 + 0.5) * (1024.0 as f64)) as u32) << 10;
                d |= ((pos.x * 0.5 + 0.5) * (1024.0 as f64)) as u32;

                data.push(d);*/
                
                t1 = pos.x as f32;
                t2 = pos.y as f32;
                t3 = pos.z as f32;
                set_pixel(t1, t2, t3, x, y);
                if x > 0 {
                    set_pixel(t1, t2, t3, x-1, y);
                }

                if y > 0 {
                    set_pixel(t1, t2, t3, x, y-1);
                }

                if x < w - 1 {
                    set_pixel(t1, t2, t3, x+1, y);
                }

                if y < h - 1 {
                    set_pixel(t1, t2, t3, x, y+1);
                }
            } else {
                set_pixel(t1, t2, t3, x, y);
            }
        }
    }

    data
}

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
use crate::ProjectionType;
use wasm_bindgen::JsValue;
impl RayTracer {
    pub fn new(gl: &WebGlContext, proj: &ProjectionType) -> Result<RayTracer, JsValue> {
        let proj_area = proj.get_area();
        let (vertices, idx) = create_vertices_array(proj_area);

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
        let data = generate_xyz_position(proj);
        let position_tex = Texture2D::create_from_raw_pixels::<al_core::image::format::RGB32F>(
            gl,
            SIZE_POSITION_TEX as i32,
            SIZE_POSITION_TEX as i32,
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
            Some(&data),
        )?;

        // create ang2pix texture for webgl1 app
        #[cfg(feature = "webgl1")]
        let ang2pix_tex = {
            let data = generate_hash_dxdy(0, proj);
            create_f32_texture_from_raw(
                &gl,
                SIZE_POSITION_TEX as i32,
                SIZE_POSITION_TEX as i32,
                &data,
            )
        };

        Ok(RayTracer {
            vao,

            position_tex,

            #[cfg(feature = "webgl1")]
            ang2pix_tex,
        })
    }

    pub fn get_vao(&self) -> &VertexArrayObject {
        &self.vao
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

    pub fn is_rendering(&self, camera: &CameraViewPort) -> bool {
        // Check whether the tile depth is 0 for square projection
        // definition domains i.e. Mercator
        let depth = camera.get_tile_depth();
        camera.is_allsky() || depth == 0
    }
}
