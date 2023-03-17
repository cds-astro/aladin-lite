pub mod grid;
pub mod subdivide_texture;

use std::vec;
use std::marker::Unpin;
use std::fmt::Debug;
//use std::io::Cursor;

use al_core::texture::MAX_TEX_SIZE;
use futures::stream::{TryStreamExt};
use futures::AsyncRead;

use wasm_bindgen::JsValue;

use web_sys::WebGl2RenderingContext;

use fitsrs::{
    hdu::{
        data::stream,
    }
};
use wcs::{ImgXY, WCS};

use al_api::hips::ImageMetadata;
use al_api::fov::CenteredFoV;

use al_core::{VertexArrayObject, Texture2D};
use al_core::WebGlContext;
use al_core::VecData;
use al_core::webgl_ctx::GlWrapper;
use al_core::image::format::*;
use al_core::image::format::ImageFormatType;

use crate::camera::CameraViewPort;
use crate::ProjectionType;
use crate::ShaderManager;
use crate::Colormaps;
use crate::math::lonlat::LonLat;

pub struct Image {
    /// A reference to the GL context
    gl: WebGlContext,

    /// The vertex array object of the screen in NDC
    vao: VertexArrayObject,
    num_indices: Vec<u32>,
    indices: Vec<u32>,
    pos: Vec<f32>,
    uv: Vec<f32>,

    /// Parameters extracted from the fits
    wcs: WCS,
    blank: f32,
    scale: f32,
    offset: f32,

    /// The center of the fits
    centered_fov: CenteredFoV,

    //+ Texture format
    format: ImageFormatType,
    /// Texture chunks objects
    textures: Vec<Texture2D>,
    /// Texture indices that must be drawn
    idx_tex: Vec<usize>,
}

use futures::io::BufReader;
use fitsrs::hdu::AsyncHDU;
use fitsrs::hdu::header::extension;

impl Image {
    pub async fn from_fits_hdu_async<'a, R>(
        gl: &WebGlContext,
        hdu: &mut AsyncHDU<'a, BufReader<R>, extension::image::Image>,
        //reader: &'a mut BufReader<R>,
    ) -> Result<Self, JsValue>
    where
        R: AsyncRead + Unpin + Debug + 'a
    {
        // Load the fits file
        let header = hdu.get_header();

        let naxis = header.get_xtension().get_naxis();

        if naxis == 0 {
            return Err(JsValue::from_str("The fits is empty, NAXIS=0"));
        } else if naxis != 2 {
            return Err(JsValue::from_str("Multi dimentional cubes are not supported"))
        }

        let scale = header
            .get_parsed::<f64>(b"BSCALE  ")
            .unwrap_or(Ok(1.0))
            .unwrap() as f32;
        let offset = header
            .get_parsed::<f64>(b"BZERO   ")
            .unwrap_or(Ok(0.0))
            .unwrap() as f32;
        let blank = header
            .get_parsed::<f64>(b"BLANK   ")
            .unwrap_or(Ok(std::f64::NAN))
            .unwrap() as f32;

        // Create a WCS from a specific header unit
        let wcs = WCS::new(&header).map_err(|_| JsValue::from_str("Failed to parse the WCS"))?;

        let (w, h) = wcs.img_dimensions();
        let width = w as f64;
        let height = h as f64;

        let data = hdu.get_data_mut();
        
        let (textures, format) = match data {
            stream::Data::U8(data) => {
                let reader = data
                    .map_ok(|v| {
                        v[0].to_le_bytes()
                    })
                    .into_async_read();

                let textures = subdivide_texture::build::<R8UI, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R8UI)
            },
            stream::Data::I16(data) => {
                let reader = data
                    .map_ok(|v| {
                        v[0].to_le_bytes()
                    })
                    .into_async_read();

                let textures = subdivide_texture::build::<R16I, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R16I)
            },
            stream::Data::I32(data) => {
                let reader = data
                    .map_ok(|v| {
                        v[0].to_le_bytes()
                    })
                    .into_async_read();

                let textures = subdivide_texture::build::<R32I, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R32I)
            },
            stream::Data::I64(data) => {
                let reader = data
                    .map_ok(|v| {
                        let v = v[0] as i32;
                        v.to_le_bytes()
                    })
                    .into_async_read();

                let textures = subdivide_texture::build::<R32I, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R32I)
            },
            stream::Data::F32(data) => {
                let reader = data
                    .map_ok(|v| {
                        v[0].to_le_bytes()
                    })
                    .into_async_read();

                let textures = subdivide_texture::build::<R32F, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R32F)
            },
            stream::Data::F64(data) => {
                let reader = data
                    .map_ok(|v| {
                        let v = v[0] as f32;
                        v.to_le_bytes()
                    })
                    .into_async_read();

                let textures = subdivide_texture::build::<R32F, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R32F)
            },
        };

        let num_indices = vec![];
        let indices = vec![];
        let pos = vec![];
        let uv = vec![];
        // Define the buffers
        let vao = {
            let mut vao = VertexArrayObject::new(gl);
            
            #[cfg(feature = "webgl2")]
            vao.bind_for_update()
                // layout (location = 0) in vec2 ndc_pos;
                .add_array_buffer_single(
                    2,
                    "ndc_pos",
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<f32>(&pos),
                )
                .add_array_buffer_single(
                    2,
                    "uv",
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<f32>(&uv),
                )
                // Set the element buffer
                .add_element_buffer(
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<u32>(&indices),
                )
                .unbind();
            #[cfg(feature = "webgl1")]
            vao.bind_for_update()
                .add_array_buffer_single(
                    2,
                    "ndc_pos",
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<f32>(&pos),
                )
                .add_array_buffer_single(
                    2,
                    "uv",
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<f32>(&uv),
                )
                // Set the element buffer
                .add_element_buffer(
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<u32>(&indices),
                )
                .unbind();

            vao
        };

        let gl = gl.clone();

        // Compute the fov
        let center = wcs.unproj_lonlat(&ImgXY::new(width / 2.0, height / 2.0))
            .ok_or(JsValue::from_str("(w / 2, h / 2) px cannot be unprojected"))?;
        let top_lonlat = wcs.unproj_lonlat(&ImgXY::new(width / 2.0, height))
            .ok_or(JsValue::from_str("(w / 2, h) px cannot be unprojected"))?;
        let left_lonlat = wcs.unproj_lonlat(&ImgXY::new(0.0, height / 2.0))
            .ok_or(JsValue::from_str("(0, h / 2) px cannot be unprojected"))?;

        let half_fov1 = crate::math::lonlat::ang_between_lonlat(
            top_lonlat.into(),
            center.clone().into()
        );
        let half_fov2 = crate::math::lonlat::ang_between_lonlat(
            left_lonlat.into(),
            center.clone().into()
        );

        let half_fov = half_fov1.max(half_fov2);

        let centered_fov = CenteredFoV {
            ra: center.lon().to_degrees(),
            dec: center.lat().to_degrees(),
            fov: 2.0 * half_fov.to_degrees(),
        };

        let idx_tex = (0..textures.len()).collect();

        // Automatic methods to compute the min and max cut values
        /*let mut values = values.into_iter()
            .filter(|x| !x.is_nan() && *x != blank)
            .collect::<Vec<_>>();
        
        let n = values.len();
        let first_pct_idx = (0.05 * (n as f32)) as usize;
        let last_pct_idx = (0.95 * (n as f32)) as usize;

        let min_val = crate::utils::select_kth_smallest(&mut values[..], 0, n - 1, first_pct_idx);
        let max_val = crate::utils::select_kth_smallest(&mut values[..], 0, n - 1, last_pct_idx);
        */
        //al_core::log(&format!("values: {} {}", min_val, max_val));

        let image = Image {
            gl,

            // The positions
            vao,
            num_indices,
            pos,
            uv,
            indices,

            // Metadata extracted from the fits
            wcs,
            scale,
            offset,
            blank,

            // Centered field of view allowing to locate the fits
            centered_fov,

            // Texture parameters
            format,
            textures,
            // Indices of textures that must be drawn
            idx_tex,
        };

        Ok(image)
    }

    pub fn update(&mut self, camera: &CameraViewPort, projection: &ProjectionType) -> Result<(), JsValue> {
        if !camera.has_moved() {
            return Ok(());
        }

        let (width, height) = self.wcs.img_dimensions();
        let width = width as f64;
        let height = height as f64;

        // Determine the x and y pixels ranges that must be drawn into the screen
        let (x_mesh_range, y_mesh_range) = if let Some(vertices) = camera.get_vertices() {
            // The field of view is defined, so we can compute its projection into the wcs
            let (mut x_fov_proj_range, mut y_fov_proj_range) = (std::f64::INFINITY..std::f64::NEG_INFINITY, std::f64::INFINITY..std::f64::NEG_INFINITY);
            
            for vertex in vertices.iter() {
                let lonlat = vertex.lonlat();

                let lon = lonlat.lon();
                let lat = lonlat.lat();

                let img_vert = self.wcs.proj(&wcs::LonLat::new(lon.to_radians(), lat.to_radians()));

                if let Some(img_vert) = img_vert {
                    x_fov_proj_range.start = x_fov_proj_range.start.min(img_vert.x());
                    x_fov_proj_range.end = x_fov_proj_range.end.max(img_vert.x());

                    y_fov_proj_range.start = y_fov_proj_range.start.min(img_vert.y());
                    y_fov_proj_range.end = y_fov_proj_range.end.max(img_vert.y());
                }
            }

            // Check if the FoV is overlapping the image
            // If not we can exit this update faster
            let is_ranges_overlapping = |x: &std::ops::Range<f64>, y: &std::ops::Range<f64>| {
                x.start <= y.end && y.start <= x.end
            };
            let fov_image_overlapping = is_ranges_overlapping(&x_fov_proj_range, &(0.0..width)) && is_ranges_overlapping(&y_fov_proj_range, &(0.0..height));

            if !fov_image_overlapping {
                self.idx_tex.clear();
                al_core::log(&format!("fov and image do not overlap"));

                return Ok(());
            }

            // The fov is overlapping the image, we must render it!
            // clamp the texture
            let x_mesh_range = x_fov_proj_range.start.max(0.0)..x_fov_proj_range.end.min(width);
            let y_mesh_range = y_fov_proj_range.start.max(0.0)..y_fov_proj_range.end.min(height);

            // Select the textures overlapping the fov
            let id_min_tx = (x_mesh_range.start as u64) / (MAX_TEX_SIZE as u64);
            let id_min_ty = (y_mesh_range.start as u64) / (MAX_TEX_SIZE as u64);

            let id_max_tx = (x_mesh_range.end as u64) / (MAX_TEX_SIZE as u64);
            let id_max_ty = (y_mesh_range.end as u64) / (MAX_TEX_SIZE as u64);

            let num_texture_y = (((height as i32) / (MAX_TEX_SIZE as i32)) + 1) as u64;

            self.idx_tex = (id_min_tx..=id_max_tx)
                .flat_map(|id_tx| {
                    (id_min_ty..=id_max_ty).map(move |id_ty| (id_ty + id_tx*num_texture_y) as usize)
                })
                .collect::<Vec<_>>();

            (x_mesh_range, y_mesh_range)
        } else {
            self.idx_tex = (0..self.textures.len()).collect();

            (0.0..width, 0.0..height)
        };

        const MAX_NUM_TRI_PER_SIDE_IMAGE: usize = 25; 
        let num_vertices = ((self.centered_fov.fov / 360.0) * (MAX_NUM_TRI_PER_SIDE_IMAGE as f64)).ceil() as u64;

        let (pos, uv, indices, num_indices) = grid::get_grid_vertices(
            &(x_mesh_range.start, y_mesh_range.start),
            &(x_mesh_range.end.ceil(), y_mesh_range.end.ceil()),
            MAX_TEX_SIZE as u64,
            num_vertices,
            camera,
            &self.wcs,
            projection
        );
        self.pos = unsafe { crate::utils::transmute_vec(pos).map_err(|s| JsValue::from_str(s))? };
        self.uv = unsafe { crate::utils::transmute_vec(uv).map_err(|s| JsValue::from_str(s))? };

        // Update num_indices
        self.indices = indices;
        self.num_indices = num_indices;

        // vertices contains ndc positions and texture UVs
        self.vao.bind_for_update()
            .update_array(
                "ndc_pos",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&self.pos),
            )
            .update_array(
                "uv",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&self.uv),
            )
            .update_element_array(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u32>(&self.indices),
            );

        Ok(())
    }

    // Draw the image
    pub fn draw(&self, shaders: &mut ShaderManager, colormaps: &Colormaps, cfg: &ImageMetadata) -> Result<(), JsValue> {
        self.gl.enable(WebGl2RenderingContext::BLEND);

        let ImageMetadata {
            color,
            opacity,
            blend_cfg,
            ..
        } = cfg;

        let shader = match self.format {
            ImageFormatType::R32F => crate::shader::get_shader(&self.gl, shaders, "FitsVS", "FitsFS")?,
            #[cfg(feature = "webgl2")]
            ImageFormatType::R32I => crate::shader::get_shader(&self.gl, shaders, "FitsVS", "FitsFSInteger")?,
            #[cfg(feature = "webgl2")]
            ImageFormatType::R16I => crate::shader::get_shader(&self.gl, shaders, "FitsVS", "FitsFSInteger")?,
            #[cfg(feature = "webgl2")]
            ImageFormatType::R8UI => crate::shader::get_shader(&self.gl, shaders, "FitsVS", "FitsFSUnsigned")?,
            _ => return Err(JsValue::from_str("Image format type not supported"))
        };

        // 2. Draw it if its opacity is not null
        blend_cfg.enable(&self.gl, || {
            let mut off_indices = 0;
            for (idx, &idx_tex) in self.idx_tex.iter().enumerate() {
                let texture = &self.textures[idx_tex];
                let num_indices = self.num_indices[idx] as i32;

                shader.bind(&self.gl)               
                    .attach_uniforms_from(colormaps)
                    .attach_uniforms_with_params_from(color, colormaps)
                    .attach_uniform("opacity", opacity)
                    .attach_uniform("tex", texture)
                    .attach_uniform("scale", &self.scale)
                    .attach_uniform("offset", &self.offset)
                    .attach_uniform("blank", &self.blank)
                    .bind_vertex_array_object_ref(&self.vao)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        Some(num_indices),
                        WebGl2RenderingContext::UNSIGNED_INT,
                        ((off_indices as usize) * std::mem::size_of::<u32>()) as i32,
                    );

                    off_indices += self.num_indices[idx];
            }

            Ok(())
        })?;

        self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }

    #[inline]
    pub fn get_centered_fov(&self) -> &CenteredFoV {
        &self.centered_fov
    }
}
