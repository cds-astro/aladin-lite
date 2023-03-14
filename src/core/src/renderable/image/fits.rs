use std::vec;
use std::marker::Unpin;
use std::fmt::Debug;
//use std::io::Cursor;

use futures::stream::{TryStreamExt};
use futures::AsyncRead;

use wasm_bindgen::{JsValue, UnwrapThrowExt};

use web_sys::WebGl2RenderingContext;

use moclib::moc::range::RangeMOC;
use moclib::qty::Hpx;
use moclib::elem::cell::Cell;
use moclib::moc::{RangeMOCIterator, RangeMOCIntoIterator};

use fitsrs::{
    fits::AsyncFits,
    hdu::{
        data::stream,
        header::extension::Xtension
    }
};
use wcs::{ImgXY, WCS, LonLat};

use al_api::cell::HEALPixCellProjeted;
use al_api::coo_system::CooSystem;
use al_api::hips::ImageMetadata;

use al_core::{VertexArrayObject, Texture2D};
use al_core::WebGlContext;
use al_core::VecData;
use al_core::webgl_ctx::GlWrapper;
use al_core::image::format::*;
use al_core::image::format::ImageFormatType;

use crate::math::projection::coo_space::XYNDC;
use crate::camera::CameraViewPort;
use crate::ProjectionType;
use crate::healpix::cell::HEALPixCell;
use crate::ShaderManager;
use crate::Colormaps;
use super::subdivide_texture::build;

pub struct FitsImage {
    // The vertex array object of the screen in NDC
    vao: VertexArrayObject,
    moc: RangeMOC<u64, Hpx<u64>>,
    wcs: WCS,

    pos: Vec<f32>,
    uv: Vec<f32>,
    indices: Vec<u32>,

    gl: WebGlContext,

    textures: Vec<Texture2D>,

    blank: f32,
    scale: f32,
    offset: f32,

    center: LonLat,

    format: ImageFormatType,
}

use futures::io::BufReader;
impl FitsImage {
    /*pub fn from_bytes(
        gl: &WebGlContext,
        bytes: &[u8]
    ) -> Result<Self, JsValue> {
        // Load the fits file
        let mut reader = Cursor::new(bytes);
        let Fits { hdu } = Fits::from_reader(&mut reader)
            .map_err(|_| JsValue::from_str("Fits cannot be parsed"))?;

        let header = hdu.get_header();
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
        al_core::log("jdjdjdj2");

        // Create a WCS from a specific header unit
        let wcs = WCS::new(&header).map_err(|_| JsValue::from_str("Failed to parse the WCS"))?;

        let (w, h) = wcs.img_dimensions();
        let width = w as f64;
        let height = h as f64;
        let tex_params = &[
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
        ];

        let data = hdu.get_data();
        let textures = vec![match data {
            InMemData::U8(data) => {
                Texture2D::create_from_raw_pixels::<al_core::image::format::R8UI>(gl, w as i32, h as i32, tex_params, Some(&data))?
            },
            InMemData::I16(data) => {
                Texture2D::create_from_raw_pixels::<al_core::image::format::R16I>(gl, w as i32, h as i32, tex_params, Some(&data))?
            },
            InMemData::I32(data) => {
                Texture2D::create_from_raw_pixels::<al_core::image::format::R32I>(gl, w as i32, h as i32, tex_params, Some(&data))?
            },
            InMemData::I64(data) => {
                let values: Vec<f32> = data.iter().map(|v| {
                    *v as f32
                })
                .collect();

                Texture2D::create_from_raw_pixels::<al_core::image::format::R32F>(gl, w as i32, h as i32, tex_params, Some(&values))?
            },
            InMemData::F32(data) => {
                Texture2D::create_from_raw_pixels::<al_core::image::format::R32F>(gl, w as i32, h as i32, tex_params, Some(data))?
            },
            InMemData::F64(data) => {
                let values: Vec<f32> = data.iter().map(|v| {
                    *v as f32
                })
                .collect();

                Texture2D::create_from_raw_pixels::<al_core::image::format::R32F>(gl, w as i32, h as i32, tex_params, Some(&values))?
            },
        }];

        let bl = wcs.unproj_lonlat(&ImgXY::new(0.0, 0.0)).ok_or(JsValue::from_str("(0, 0) px cannot be unprojected"))?;
        let br = wcs.unproj_lonlat(&ImgXY::new(width - 1.0, 0.0)).ok_or(JsValue::from_str("(w - 1, 0) px cannot be unprojected"))?;
        let tr = wcs.unproj_lonlat(&ImgXY::new(width - 1.0, height - 1.0)).ok_or(JsValue::from_str("(w - 1, h - 1) px cannot be unprojected"))?;
        let tl = wcs.unproj_lonlat(&ImgXY::new(0.0, height - 1.0)).ok_or(JsValue::from_str("(0, h - 1) px cannot be unprojected"))?;

        let center = wcs.unproj_lonlat(&ImgXY::new(width / 2.0, height / 2.0)).ok_or(JsValue::from_str("(w / 2, h / 2) px cannot be unprojected"))?;
        
        let mut num_moc_cells = std::usize::MAX;
        let mut depth = 11;
        let mut moc = RangeMOC::new_empty(0);
        while num_moc_cells > 5 && depth > 3 {
            depth = depth - 1;
            moc = RangeMOC::from_polygon_with_control_point(
                &[
                    (bl.lon(), bl.lat()),
                    (br.lon(), br.lat()),
                    (tr.lon(), tr.lat()),
                    (tl.lon(), tl.lat()),
                ],
                (center.lon(), center.lat()),
                depth
            );

            num_moc_cells = (&moc).into_range_moc_iter().cells().count();
        }
        
        let pos = vec![];
        let uv = vec![];
        let indices = vec![];
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

        let gl = gl.clone();
        let image = FitsImage {
            vao,
            wcs,
            moc,
            gl,

            pos,
            uv,
            indices,

            textures,
            scale,
            offset,
            blank,

            center,
        };

        Ok(image)
    }*/

    pub async fn new_async<'a, R>(
        gl: &WebGlContext,
        reader: &'a mut BufReader<R>,
    ) -> Result<Self, JsValue>
    where
        R: AsyncRead + Unpin + Debug
    {
        // Load the fits file
        let AsyncFits { mut hdu } = AsyncFits::from_reader(reader)
            .await
            .map_err(|_| JsValue::from_str("Error parsing the fits"))?;

        let header = hdu.get_header();

        let num_bytes_to_read = header.get_xtension().get_num_bytes_data_block();
        let naxis1 = *header.get_xtension().get_naxisn(1).unwrap_throw();
        let naxis2 = *header.get_xtension().get_naxisn(2).unwrap_throw();
        let bitpix = header.get_xtension().get_bitpix() as i32;

        al_core::log(&format!("size image {naxis1} {naxis2} {num_bytes_to_read} {bitpix}"));

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

                let textures = build::<R8UI, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R8UI)
            },
            stream::Data::I16(data) => {
                let reader = data
                    .map_ok(|v| {
                        v[0].to_le_bytes()
                    })
                    .into_async_read();

                let textures = build::<R16I, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R16I)
            },
            stream::Data::I32(data) => {
                let reader = data
                    .map_ok(|v| {
                        v[0].to_le_bytes()
                    })
                    .into_async_read();

                let textures = build::<R32I, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R32I)
            },
            stream::Data::I64(data) => {
                let reader = data
                    .map_ok(|v| {
                        let v = v[0] as i32;
                        v.to_le_bytes()
                    })
                    .into_async_read();

                let textures = build::<R32I, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R32I)
            },
            stream::Data::F32(data) => {
                let reader = data
                    .map_ok(|v| {
                        v[0].to_le_bytes()
                    })
                    .into_async_read();

                let textures = build::<R32F, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R32F)
            },
            stream::Data::F64(data) => {
                let reader = data
                    .map_ok(|v| {
                        let v = v[0] as f32;
                        v.to_le_bytes()
                    })
                    .into_async_read();

                let textures = build::<R32F, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R32F)
            },
        };

        let bl = wcs.unproj_lonlat(&ImgXY::new(0.0, 0.0)).ok_or(JsValue::from_str("(0, 0) px cannot be unprojected"))?;
        let br = wcs.unproj_lonlat(&ImgXY::new(width - 1.0, 0.0)).ok_or(JsValue::from_str("(w - 1, 0) px cannot be unprojected"))?;
        let tr = wcs.unproj_lonlat(&ImgXY::new(width - 1.0, height - 1.0)).ok_or(JsValue::from_str("(w - 1, h - 1) px cannot be unprojected"))?;
        let tl = wcs.unproj_lonlat(&ImgXY::new(0.0, height - 1.0)).ok_or(JsValue::from_str("(0, h - 1) px cannot be unprojected"))?;

        let center = wcs.unproj_lonlat(&ImgXY::new(width / 2.0, height / 2.0)).ok_or(JsValue::from_str("(w / 2, h / 2) px cannot be unprojected"))?;

        let mut num_moc_cells = std::usize::MAX;
        let mut depth = 11;
        let mut moc = RangeMOC::new_empty(0);
        while num_moc_cells > 5 && depth > 3 {
            depth = depth - 1;
            moc = RangeMOC::from_polygon_with_control_point(
                &[
                    (bl.lon(), bl.lat()),
                    (br.lon(), br.lat()),
                    (tr.lon(), tr.lat()),
                    (tl.lon(), tl.lat()),
                ],
                (center.lon(), center.lat()),
                depth
            );

            num_moc_cells = (&moc).into_range_moc_iter().cells().count();
        }

        let pos = vec![];
        let uv = vec![];
        let indices = vec![];
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

        let gl = gl.clone();
        let image = FitsImage {
            vao,
            wcs,
            moc,
            gl,

            pos,
            uv,
            indices,

            textures,
            scale,
            offset,
            blank,

            center,
            format,
        };

        Ok(image)
    }

    pub fn update(&mut self, camera: &CameraViewPort, projection: &ProjectionType) -> Result<(), JsValue> {
        if !camera.has_moved() {
            return Ok(());
        }
        self.indices.clear();
        self.uv.clear();
        self.pos.clear();

        let mut idx_off = 0;

        let depth_max = self.moc.depth_max();

        for Cell { depth, idx, .. } in (&self.moc).into_range_moc_iter().cells() {
            let delta_depth = (depth_max as i32 - depth as i32).max(0);
            let n_segment_by_side = (1 << delta_depth) as usize;

            let cell = HEALPixCell(depth, idx);
            if depth < 3 {
                let mut ndc_cells_d3 = vec![];
                let mut uv_cells_d3 = vec![];

                let depth_sub_cell = 3;
                let delta_depth_sub_cell = depth_max - depth_sub_cell;
                let n_segment_by_side_sub_cell = (1 << delta_depth_sub_cell) as usize;

                for sub_cell in cell.get_children_cells(3 - depth) {
                    if let Some((ndc_sub_cell, uv_sub_cell, indices_sub_cell)) = self::rasterize_hpx_cell(
                        &sub_cell,
                        n_segment_by_side_sub_cell,
                        &mut idx_off,
                        camera,
                        projection,
                        &self.wcs
                    ) {
                        self.indices.extend(indices_sub_cell);
                        ndc_cells_d3.extend(ndc_sub_cell);
                        uv_cells_d3.extend(uv_sub_cell);
                    }
                }

                self.pos.extend(&ndc_cells_d3);
                self.uv.extend(&uv_cells_d3);
            } else if let Some((ndc_cell, uv_cell, indices_cell)) = self::rasterize_hpx_cell(
                &cell,
                n_segment_by_side,
                &mut idx_off,
                camera,
                projection,
                &self.wcs,
            ) {
                // Generate the iterator: idx_off + 1, idx_off + 1, .., idx_off + 4*n_segment - 1, idx_off + 4*n_segment - 1
                self.indices.extend(indices_cell);

                self.pos.extend(&ndc_cell);
                self.uv.extend(&uv_cell);
            }
        }

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

        // 2. Draw it if its opacity is not null
        blend_cfg.enable(&self.gl, || {
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

            shader
                .bind(&self.gl)
                .attach_uniforms_from(colormaps)
                .attach_uniforms_with_params_from(color, colormaps)
                .attach_uniform("opacity", opacity)
                .attach_uniform("tex", &self.textures[0])
                .attach_uniform("scale", &self.scale)
                .attach_uniform("offset", &self.offset)
                .attach_uniform("blank", &self.blank)
                .bind_vertex_array_object_ref(&self.vao)
                .draw_elements_with_i32(
                    WebGl2RenderingContext::TRIANGLES,
                    Some(self.indices.len() as i32),
                    WebGl2RenderingContext::UNSIGNED_INT,
                    0,
                );

            Ok(())
        })?;

        self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }

    pub fn get_center(&self) -> &LonLat {
        &self.center
    }
}

use crate::math::angle::ToAngle;
fn rasterize_hpx_cell(cell: &HEALPixCell, n_segment_by_side: usize, idx_off: &mut u32, camera: &CameraViewPort, projection: &ProjectionType, wcs: &WCS) -> Option<(Vec<f32>, Vec<f32>, Vec<u32>)> {
    let n_vertices_per_segment = n_segment_by_side + 1;

    let (w, h) = wcs.img_dimensions();
    let w = w as f64;
    let h = h as f64;
    let mut uv = vec![];
    let mut ndc_pos = vec![];

    for (lon, lat) in cell.grid(n_segment_by_side as u32).iter() {
        let xyzw = crate::math::lonlat::radec_to_xyzw(lon.to_angle(), lat.to_angle());
        let xyzw = crate::coosys::apply_coo_system(&CooSystem::ICRSJ2000, camera.get_system(), &xyzw);

        if let Some((pos_vert, uv_vert)) = projection.model_to_normalized_device_space(&xyzw, camera)
            .map(|v| {
                wcs.proj(&wcs::LonLat::new(*lon, *lat))
                    .map(|xy| {
                        let uv = ImgXY::new(xy.x() / w, xy.y() / h);
                        (
                            [v.x as f32, v.y as f32],
                            [uv.x() as f32, uv.y() as f32]
                        )
                    })
            }).flatten() {
            ndc_pos.extend(pos_vert);
            uv.extend(uv_vert);
        }
    }

    let cell_inside = ndc_pos.len() == 2*(n_segment_by_side+1)*(n_segment_by_side+1);

    if cell_inside {
        // Generate the iterator: idx_off + 1, idx_off + 1, .., idx_off + 4*n_segment - 1, idx_off + 4*n_segment - 1
        let mut indices = Vec::with_capacity(n_segment_by_side * n_segment_by_side * 6);
        let num_vertices = (n_segment_by_side+1)*(n_segment_by_side+1);

        let longitude_reversed = camera.get_longitude_reversed();
        let invalid_tri = |tri_ccw: bool, reversed_longitude: bool| -> bool {
            (!reversed_longitude && !tri_ccw) || (reversed_longitude && tri_ccw)
        };

        for i in 0..n_segment_by_side {
            for j in 0..n_segment_by_side {
                let idx_0 = j + i * n_vertices_per_segment;
                let idx_1 = j + 1 + i * n_vertices_per_segment;
                let idx_2 = j + (i + 1) * n_vertices_per_segment;
                let idx_3 = j + 1 + (i + 1) * n_vertices_per_segment;

                let c0 = crate::math::projection::ndc_to_screen_space(&XYNDC::new(ndc_pos[2*idx_0] as f64, ndc_pos[2*idx_0 + 1] as f64), camera);
                let c1 = crate::math::projection::ndc_to_screen_space(&XYNDC::new(ndc_pos[2*idx_1] as f64, ndc_pos[2*idx_1 + 1] as f64), camera);
                let c2 = crate::math::projection::ndc_to_screen_space(&XYNDC::new(ndc_pos[2*idx_2] as f64, ndc_pos[2*idx_2 + 1] as f64), camera);
                let c3 = crate::math::projection::ndc_to_screen_space(&XYNDC::new(ndc_pos[2*idx_3] as f64, ndc_pos[2*idx_3 + 1] as f64), camera);

                let first_tri_ccw = !crate::math::vector::ccw_tri(&c0, &c1, &c2);
                let second_tri_ccw = !crate::math::vector::ccw_tri(&c1, &c3, &c2);

                if invalid_tri(first_tri_ccw, longitude_reversed) || invalid_tri(second_tri_ccw, longitude_reversed) {
                    return None;
                }

                let vx = [c0.x, c1.x, c2.x, c3.x];
                let vy = [c0.y, c1.y, c2.y, c3.y];

                let projeted_cell = HEALPixCellProjeted {
                    ipix: cell.idx(),
                    vx,
                    vy
                };

                crate::survey::view::project(projeted_cell, camera, projection)?;

                indices.push(*idx_off + idx_0 as u32);
                indices.push(*idx_off + idx_1 as u32);
                indices.push(*idx_off + idx_2 as u32);

                indices.push(*idx_off + idx_1 as u32);
                indices.push(*idx_off + idx_3 as u32);
                indices.push(*idx_off + idx_2 as u32);
            }
        }

        *idx_off += num_vertices as u32;

        Some((ndc_pos, uv, indices))
    } else {
        None
    }
}