pub mod cuts;
pub mod grid;
pub mod subdivide_texture;

use al_core::webgl_ctx::WebGlRenderingCtx;
use al_core::convert::Cast;
use std::fmt::Debug;
use std::marker::Unpin;
use std::vec;

use al_api::coo_system::CooSystem;
use cgmath::Vector3;
use futures::stream::TryStreamExt;
use futures::AsyncRead;

use wasm_bindgen::JsValue;

use web_sys::WebGl2RenderingContext;

use fitsrs::hdu::data::stream;
use wcs::{ImgXY, WCS};

use al_api::fov::CenteredFoV;
use al_api::hips::ImageMetadata;

use al_core::image::format::*;
use al_core::webgl_ctx::GlWrapper;
use al_core::VecData;
use al_core::WebGlContext;
use al_core::{Texture2D, VertexArrayObject};

use crate::camera::CameraViewPort;
use crate::math::sph_geom::region::Region;
use crate::Colormaps;
use crate::ProjectionType;
use crate::ShaderManager;

use std::ops::Range;
type PixelItem<F> = <<F as ImageFormat>::P as Pixel>::Item;

pub struct Image {
    /// A reference to the GL context
    gl: WebGlContext,

    /// The vertex array object of the screen in NDC
    vao: VertexArrayObject,
    num_indices: Vec<u32>,
    indices: Vec<u16>,
    pos: Vec<f32>,
    uv: Vec<f32>,

    /// Parameters extracted from the fits
    wcs: WCS,
    blank: Option<f32>,
    scale: f32,
    offset: f32,
    cuts: Range<f32>,
    /// The center of the fits
    centered_fov: CenteredFoV,

    //+ Texture format
    channel: ChannelType,
    /// Texture chunks objects
    textures: Vec<Texture2D>,
    /// Texture indices that must be drawn
    idx_tex: Vec<usize>,
    /// The maximum webgl supported texture size
    max_tex_size_x: usize,
    max_tex_size_y: usize,

    reg: Region,
    // The coo system in which the polygonal region has been defined
    coo_sys: CooSystem,
}
use al_core::pixel::Pixel;
use fitsrs::hdu::header::extension;
use fitsrs::hdu::AsyncHDU;
use futures::io::BufReader;
use futures::AsyncReadExt;

impl Image {
    pub async fn from_reader_and_wcs<R, F>(
        gl: &WebGlContext,
        mut reader: R,
        wcs: WCS,
        scale: Option<f32>,
        offset: Option<f32>,
        blank: Option<f32>,
        // Coo sys of the view
        coo_sys: CooSystem,
    ) -> Result<Self, JsValue>
    where
        F: ImageFormat,
        R: AsyncReadExt + Unpin,
    {
        let (width, height) = wcs.img_dimensions();

        let max_tex_size =
            WebGl2RenderingContext::get_parameter(gl, WebGl2RenderingContext::MAX_TEXTURE_SIZE)?
                .as_f64()
                .unwrap_or(4096.0) as usize;

        let mut max_tex_size_x = max_tex_size;
        let mut max_tex_size_y = max_tex_size;

        // apply bscale to the cuts
        let offset = offset.unwrap_or(0.0);
        let scale = scale.unwrap_or(1.0);

        let (textures, cuts) = if width <= max_tex_size as u64 && height <= max_tex_size as u64
        {
            max_tex_size_x = width as usize;
            max_tex_size_y = height as usize;
            // can fit in one texture

            let num_pixels_to_read = (width as usize) * (height as usize);
            let num_bytes_to_read =
                num_pixels_to_read * std::mem::size_of::<F::P>();
            let mut buf = vec![0; num_bytes_to_read];

            let _ = reader
                .read_exact(&mut buf[..num_bytes_to_read])
                .await
                .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

            // bytes aligned
            unsafe {
                let data = std::slice::from_raw_parts_mut(
                    buf[..].as_mut_ptr() as *mut PixelItem<F>,
                    (num_pixels_to_read as usize) * F::NUM_CHANNELS,
                );

                let texture = Texture2D::create_from_raw_pixels::<F>(
                    gl,
                    width as i32,
                    height as i32,
                    &[
                        (
                            WebGlRenderingCtx::TEXTURE_MIN_FILTER,
                            WebGlRenderingCtx::NEAREST_MIPMAP_NEAREST,
                        ),
                        (
                            WebGlRenderingCtx::TEXTURE_MAG_FILTER,
                            WebGlRenderingCtx::NEAREST,
                        ),
                        // Prevents s-coordinate wrapping (repeating)
                        (
                            WebGlRenderingCtx::TEXTURE_WRAP_S,
                            WebGlRenderingCtx::CLAMP_TO_EDGE,
                        ),
                        // Prevents t-coordinate wrapping (repeating)
                        (
                            WebGlRenderingCtx::TEXTURE_WRAP_T,
                            WebGlRenderingCtx::CLAMP_TO_EDGE,
                        ),
                    ],
                    Some(data),
                )?;

                let cuts = match F::CHANNEL_TYPE {
                    ChannelType::R32F | ChannelType::R64F => {
                        let pixels = std::slice::from_raw_parts(data.as_ptr() as *const f32, data.len() / 4);

                        let mut sub_pixels = pixels.iter()
                            .step_by(100)
                            .filter(|pixel| (*pixel).is_finite())
                            .cloned()
                            .collect::<Vec<_>>();

                        cuts::first_and_last_percent(&mut sub_pixels, 1, 99)
                    }
                    ChannelType::R8UI | ChannelType::R16I | ChannelType::R32I => {
                        // BLANK is only valid for those channels/BITPIX (> 0)
                        if let Some(blank) = blank {
                            let mut sub_pixels = data.iter()
                                .step_by(100)
                                .filter_map(|pixel| {
                                    let pixel = <PixelItem::<F> as Cast<f32>>::cast(*pixel);

                                    if pixel != blank {
                                        Some(pixel)
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>();

                            cuts::first_and_last_percent(&mut sub_pixels, 1, 99)
                        } else {
                            // No blank value => we consider all the values
                            let mut sub_pixels = data.iter()
                                .step_by(100)
                                .map(|pixel| <PixelItem::<F> as Cast<f32>>::cast(*pixel))
                                .collect::<Vec<_>>();

                            cuts::first_and_last_percent(&mut sub_pixels, 1, 99)
                        }
                    }
                    // RGB(A) images
                    _ => 0.0..1.0
                };

                (vec![texture], cuts)
            }
        } else {
            subdivide_texture::crop_image::<F, R>(
                gl,
                width,
                height,
                reader,
                max_tex_size as u64,
                blank,
            )
            .await?
        };

        for tex in &textures {
            tex.generate_mipmap();
        }

        let start = cuts.start * scale + offset;
        let end = cuts.end * scale + offset;

        let cuts = start..end;

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
                    VecData::<u16>(&indices),
                )
                .unbind();

            vao
        };
        let gl = gl.clone();

        // Compute the fov
        let center = wcs
            .unproj_lonlat(&ImgXY::new(width as f64 / 2.0, height as f64 / 2.0))
            .ok_or(JsValue::from_str("(w / 2, h / 2) px cannot be unprojected"))?;
        let center_xyz = center.to_xyz();
        let inside = crate::coosys::apply_coo_system(
            CooSystem::ICRS,
            coo_sys,
            &Vector3::new(center_xyz.y(), center_xyz.z(), center_xyz.x()),
        );

        let vertices = [
            wcs.unproj_lonlat(&ImgXY::new(0.0, 0.0))
                .ok_or(JsValue::from_str("(0, 0) does not lie in the sky"))?,
            wcs.unproj_lonlat(&ImgXY::new(width as f64 - 1.0, 0.0))
                .ok_or(JsValue::from_str("(w - 1, 0) does not lie in the sky"))?,
            wcs.unproj_lonlat(&ImgXY::new(width as f64 - 1.0, height as f64 - 1.0))
                .ok_or(JsValue::from_str("(w - 1, h - 1) does not lie in the sky"))?,
            wcs.unproj_lonlat(&ImgXY::new(0.0, height as f64 - 1.0))
                .ok_or(JsValue::from_str("(0, h - 1) does not lie in the sky"))?,
        ]
        .iter()
        .map(|lonlat| {
            let xyz = lonlat.to_xyz();

            crate::coosys::apply_coo_system(
                CooSystem::ICRS,
                coo_sys,
                &Vector3::new(xyz.y(), xyz.z(), xyz.x()),
            )
        })
        .collect::<Vec<_>>();

        let reg = Region::from_vertices(&vertices, &inside);

        // ra and dec must be given in ICRS coo system, which is the case because wcs returns
        // only ICRS coo
        let centered_fov = CenteredFoV {
            ra: center.lon().to_degrees(),
            dec: center.lat().to_degrees(),
            fov: wcs.field_of_view().0,
        };

        let idx_tex = (0..textures.len()).collect();

        Ok(Image {
            gl,

            // The positions
            vao,
            num_indices,
            pos,
            uv,
            indices,

            // Metadata extracted from the fits
            wcs,
            // CooSystem of the wcs, this should belong to the WCS
            scale,
            offset,
            blank,

            // Centered field of view allowing to locate the fits
            centered_fov,

            // Texture parameters
            channel: F::CHANNEL_TYPE,
            textures,
            cuts,
            max_tex_size_x,
            max_tex_size_y,
            // Indices of textures that must be drawn
            idx_tex,
            // The polygonal region in the sky
            reg,
            // The coo system in which the polygonal region has been defined
            coo_sys,
        })
    }

    pub fn get_cuts(&self) -> &Range<f32> {
        &self.cuts
    }

    pub async fn from_fits_hdu_async<'a, R>(
        gl: &WebGlContext,
        hdu: &mut AsyncHDU<'a, BufReader<R>, extension::image::Image>,
        coo_sys: CooSystem,
    ) -> Result<Self, JsValue>
    where
        R: AsyncRead + Unpin + Debug + 'a,
    {
        // Load the FITS file
        let header = hdu.get_header();

        let scale = header
            .get_parsed::<f64>(b"BSCALE  ")
            .map(|v| v.unwrap());
        let offset = header
            .get_parsed::<f64>(b"BZERO   ")
            .map(|v| v.unwrap());
        let blank = header
            .get_parsed::<f64>(b"BLANK   ")
            .map(|v| v.unwrap());

        // Create a WCS from a specific header unit
        let wcs = WCS::from_fits_header(header)
            .map_err(|e| JsValue::from_str(&format!("WCS parsing error: reason: {}", e)))?;

        let data = hdu.get_data_mut();

        match data {
            stream::Data::U8(data) => {
                let reader = data.map_ok(|v| v[0].to_le_bytes()).into_async_read();

                Self::from_reader_and_wcs::<_, R8UI>(
                    gl,
                    reader,
                    wcs,
                    scale.map(|v| v as f32),
                    offset.map(|v| v as f32),
                    blank.map(|v| v as f32),
                    coo_sys,
                )
                .await
            }
            stream::Data::I16(data) => {
                let reader = data.map_ok(|v| v[0].to_le_bytes()).into_async_read();

                Self::from_reader_and_wcs::<_, R16I>(
                    gl,
                    reader,
                    wcs,
                    scale.map(|v| v as f32),
                    offset.map(|v| v as f32),
                    blank.map(|v| v as f32),
                    coo_sys,
                )
                .await
            }
            stream::Data::I32(data) => {
                let reader = data.map_ok(|v| v[0].to_le_bytes()).into_async_read();

                Self::from_reader_and_wcs::<_, R32I>(
                    gl,
                    reader,
                    wcs,
                    scale.map(|v| v as f32),
                    offset.map(|v| v as f32),
                    blank.map(|v| v as f32),
                    coo_sys,
                )
                .await
            }
            stream::Data::I64(data) => {
                let reader = data
                    .map_ok(|v| {
                        let v = v[0] as i32;
                        v.to_le_bytes()
                    })
                    .into_async_read();

                Self::from_reader_and_wcs::<_, R32I>(
                    gl,
                    reader,
                    wcs,
                    scale.map(|v| v as f32),
                    offset.map(|v| v as f32),
                    blank.map(|v| v as f32),
                    coo_sys,
                )
                .await
            }
            stream::Data::F32(data) => {
                let reader = data.map_ok(|v| v[0].to_le_bytes()).into_async_read();

                Self::from_reader_and_wcs::<_, R32F>(
                    gl,
                    reader,
                    wcs,
                    scale.map(|v| v as f32),
                    offset.map(|v| v as f32),
                    blank.map(|v| v as f32),
                    coo_sys,
                )
                .await
            }
            stream::Data::F64(data) => {
                let reader = data
                    .map_ok(|v| {
                        let v = v[0] as f32;
                        v.to_le_bytes()
                    })
                    .into_async_read();

                Self::from_reader_and_wcs::<_, R32F>(
                    gl,
                    reader,
                    wcs,
                    scale.map(|v| v as f32),
                    offset.map(|v| v as f32),
                    blank.map(|v| v as f32),
                    coo_sys,
                )
                .await
            }
        }
    }

    pub fn recompute_vertices(
        &mut self,
        camera: &CameraViewPort,
        projection: &ProjectionType,
    ) -> Result<(), JsValue> {
        let (width, height) = self.wcs.img_dimensions();
        let width = width as f64;
        let height = height as f64;

        let (x_mesh_range, y_mesh_range) =
            if camera.get_field_of_view().intersects_region(&self.reg) {
                self.idx_tex = (0..self.textures.len()).collect();

                (0.0..width, 0.0..height)
            } else {
                // out of field of view
                self.idx_tex.clear();

                // terminate here
                return Ok(());
            };

        const MAX_NUM_TRI_PER_SIDE_IMAGE: usize = 15;
        let num_vertices =
            ((self.centered_fov.fov / 180.0) * (MAX_NUM_TRI_PER_SIDE_IMAGE as f64)).ceil() as u64;

        let (pos, uv, indices, num_indices) = grid::vertices(
            &(x_mesh_range.start, y_mesh_range.start),
            &(x_mesh_range.end.ceil(), y_mesh_range.end.ceil()),
            self.max_tex_size_x as u64,
            self.max_tex_size_y as u64,
            num_vertices,
            camera,
            &self.wcs,
            projection,
        );

        self.pos = pos;
        self.uv = uv;

        // Update num_indices
        self.indices = indices;
        self.num_indices = num_indices;

        // vertices contains ndc positions and texture UVs
        self.vao
            .bind_for_update()
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
                VecData::<u16>(&self.indices),
            );

        Ok(())
    }

    // Draw the image
    pub fn draw(
        &mut self,
        shaders: &mut ShaderManager,
        colormaps: &Colormaps,
        cfg: &ImageMetadata,
        camera: &CameraViewPort,
        projection: &ProjectionType,
    ) -> Result<(), JsValue> {
        if self.coo_sys != camera.get_coo_system() {
            self.coo_sys = camera.get_coo_system();

            let (width, height) = self.wcs.img_dimensions();

            // the camera coo system is not sync with the one in which the region
            // has been defined
            // let's redefine the region
            let center = self
                .wcs
                .unproj_lonlat(&ImgXY::new(width as f64 / 2.0, height as f64 / 2.0))
                .ok_or(JsValue::from_str("(w / 2, h / 2) px cannot be unprojected"))?;
            let center_xyz = center.to_xyz();
            let inside = crate::coosys::apply_coo_system(
                CooSystem::ICRS,
                self.coo_sys,
                &Vector3::new(center_xyz.y(), center_xyz.z(), center_xyz.x()),
            );

            let vertices = [
                self.wcs
                    .unproj_lonlat(&ImgXY::new(0.0, 0.0))
                    .ok_or(JsValue::from_str("(0, 0) does not lie in the sky"))?,
                self.wcs
                    .unproj_lonlat(&ImgXY::new(width as f64 - 1.0, 0.0))
                    .ok_or(JsValue::from_str("(w - 1, 0) does not lie in the sky"))?,
                self.wcs
                    .unproj_lonlat(&ImgXY::new(width as f64 - 1.0, height as f64 - 1.0))
                    .ok_or(JsValue::from_str("(w - 1, h - 1) does not lie in the sky"))?,
                self.wcs
                    .unproj_lonlat(&ImgXY::new(0.0, height as f64 - 1.0))
                    .ok_or(JsValue::from_str("(0, h - 1) does not lie in the sky"))?,
            ]
            .iter()
            .map(|lonlat| {
                let xyz = lonlat.to_xyz();

                crate::coosys::apply_coo_system(
                    CooSystem::ICRS,
                    self.coo_sys,
                    &Vector3::new(xyz.y(), xyz.z(), xyz.x()),
                )
            })
            .collect::<Vec<_>>();

            self.reg = Region::from_vertices(&vertices, &inside);
        }

        self.recompute_vertices(camera, projection)?;

        if self.num_indices.is_empty() {
            return Ok(());
        }

        //self.gl.enable(WebGl2RenderingContext::BLEND);

        let ImageMetadata {
            color,
            opacity,
            blend_cfg,
            ..
        } = cfg;

        let shader = match self.channel {
            ChannelType::RGBA8U => crate::shader::get_shader(
                &self.gl,
                shaders,
                "image_base.vert",
                "image_sampler.frag",
            )?,
            ChannelType::R32F => {
                crate::shader::get_shader(&self.gl, shaders, "fits_base.vert", "fits_sampler.frag")?
            }
            #[cfg(feature = "webgl2")]
            ChannelType::R32I => crate::shader::get_shader(
                &self.gl,
                shaders,
                "fits_base.vert",
                "fits_isampler.frag",
            )?,
            #[cfg(feature = "webgl2")]
            ChannelType::R16I => crate::shader::get_shader(
                &self.gl,
                shaders,
                "fits_base.vert",
                "fits_isampler.frag",
            )?,
            #[cfg(feature = "webgl2")]
            ChannelType::R8UI => crate::shader::get_shader(
                &self.gl,
                shaders,
                "fits_base.vert",
                "fits_usampler.frag",
            )?,
            _ => return Err(JsValue::from_str("Image format type not supported")),
        };

        //self.gl.disable(WebGl2RenderingContext::CULL_FACE);

        // 2. Draw it if its opacity is not null

        blend_cfg.enable(&self.gl, || {
            let mut off_indices = 0;
            for (idx, &idx_tex) in self.idx_tex.iter().enumerate() {
                let texture = &self.textures[idx_tex];
                let num_indices = self.num_indices[idx] as i32;

                let shader_bound = shader
                    .bind(&self.gl);

                shader_bound
                    .attach_uniforms_from(colormaps)
                    .attach_uniforms_with_params_from(color, colormaps)
                    .attach_uniform("opacity", opacity)
                    .attach_uniform("tex", texture)
                    .attach_uniform("scale", &self.scale)
                    .attach_uniform("offset", &self.offset);

                if let Some(blank) = self.blank {
                    shader_bound.attach_uniform("blank", &blank);
                }

                shader_bound
                    .bind_vertex_array_object_ref(&self.vao)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        Some(num_indices),
                        WebGl2RenderingContext::UNSIGNED_SHORT,
                        ((off_indices as usize) * std::mem::size_of::<u16>()) as i32,
                    );

                off_indices += num_indices;
            }

            Ok(())
        })?;

        //self.gl.enable(WebGl2RenderingContext::CULL_FACE);

        //self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }

    #[inline]
    pub fn get_centered_fov(&self) -> &CenteredFoV {
        &self.centered_fov
    }
}
