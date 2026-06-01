pub mod cuts;
pub mod grid;
pub mod subdivide_texture;

use al_core::texture::format::PixelType;
use al_core::texture::format::RGBA8U;
use al_core::texture::format::{R16I, R32F, R32I, R8U};
use al_core::webgl_ctx::WebGlRenderingCtx;
use fitsrs::hdu::header::Bitpix;
use std::vec;

use al_api::coo_system::CooSystem;
use cgmath::Vector3;

use wasm_bindgen::JsValue;

use web_sys::WebGl2RenderingContext;

use fitsrs::wcs::{ImgXY, WCS};

use al_api::fov::CenteredFoV;
use al_api::hips::ImageMetadata;

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

use self::subdivide_texture::crop_image;
use self::subdivide_texture::ImagePatches;

pub struct Image {
    /// A reference to the GL context
    gl: WebGlContext,

    /// The vertex array object of the screen in NDC
    vao: VertexArrayObject,
    num_indices: Vec<u32>,
    indices: Vec<u16>,
    pos: Vec<f32>,
    uv: Vec<f32>,

    /// WCS allowing to locate the image on the sky
    wcs: WCS,

    /// Some parameters, only defined for image coming from FITS files
    blank: Option<f32>,
    bscale: f32,
    bzero: f32,

    cuts: Range<f32>,
    /// The center of the fits
    centered_fov: CenteredFoV,

    //+ Texture format
    pixel_type: PixelType,
    /// Texture chunks objects
    textures: Vec<Texture2D>,
    /// Texture indices that must be drawn
    idx_tex: Vec<usize>,
    /// The size of a textured image patch
    /// that can be uploaded to the GPU
    w_patch: usize,
    h_patch: usize,

    reg: Region,
    // The coo system in which the polygonal region has been defined
    coo_sys: CooSystem,
}

const TEX_PARAMS: &[(u32, u32)] = &[
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
];
impl Image {
    #[allow(clippy::too_many_arguments)]
    fn init_buffers(
        gl: WebGlContext,
        patches: ImagePatches,
        wcs: WCS,
        bscale: f32,
        bzero: f32,
        blank: Option<f32>,
        coo_sys: CooSystem,
    ) -> Result<Self, JsValue> {
        let dim = wcs.img_dimensions();
        let (width, height) = (dim[0] as u64, dim[1] as u64);

        let ImagePatches {
            pixel_type,
            texture_patches: textures,
            initial_cuts: cuts,
            w_patch,
            h_patch,
        } = patches;

        for tex in &textures {
            tex.generate_mipmap();
        }

        let start = cuts.start * bscale + bzero;
        let end = cuts.end * bscale + bzero;

        let cuts = start..end;

        let num_indices = vec![];
        let indices = vec![];
        let pos = vec![];
        let uv = vec![];
        // Define the buffers
        let vao = {
            let mut vao = VertexArrayObject::new(&gl);

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

        // Compute the fov
        let center = wcs
            .unproj_lonlat(&ImgXY::new(
                (width as f64 / 2.0) + 0.5,
                (height as f64 / 2.0) + 0.5,
            ))
            .ok_or(JsValue::from_str("(w / 2, h / 2) px cannot be unprojected"))?;
        let center_xyz = center.to_xyz();
        let inside = crate::coosys::apply_coo_system(
            CooSystem::ICRS,
            coo_sys,
            &Vector3::new(center_xyz.y(), center_xyz.z(), center_xyz.x()),
        );

        let vertices = [
            wcs.unproj_lonlat(&ImgXY::new(0.5, 0.5))
                .ok_or(JsValue::from_str("(0, 0) does not lie in the sky"))?,
            wcs.unproj_lonlat(&ImgXY::new(width as f64 - 0.5, 0.5))
                .ok_or(JsValue::from_str("(w - 1, 0) does not lie in the sky"))?,
            wcs.unproj_lonlat(&ImgXY::new(width as f64 - 0.5, height as f64 - 0.5))
                .ok_or(JsValue::from_str("(w - 1, h - 1) does not lie in the sky"))?,
            wcs.unproj_lonlat(&ImgXY::new(0.5, height as f64 - 0.5))
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

        Ok(Self {
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
            bscale,
            bzero,
            blank,

            // Centered field of view allowing to locate the fits
            centered_fov,

            // Texture parameters
            pixel_type,
            textures,
            cuts,
            w_patch,
            h_patch,
            // Indices of textures that must be drawn
            idx_tex,
            // The polygonal region in the sky
            reg,
            // The coo system in which the polygonal region has been defined
            coo_sys,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_fits_hdu(
        gl: &WebGlContext,
        // wcs extracted from the image HDU
        wcs: fitsrs::WCS,
        // bitpix extracted from the image HDU
        bitpix: fitsrs::hdu::header::Bitpix,
        // bytes slice extracted from the HDU
        bytes: &[u8],
        // other keywords extracted from the header of the image HDU
        bscale: f32,
        bzero: f32,
        blank: Option<f32>,
        // Coo sys of the view
        coo_sys: CooSystem,
    ) -> Result<Self, JsValue> {
        let dim = wcs.img_dimensions();
        let (width, height) = (dim[0] as u64, dim[1] as u64);

        let max_tex_size =
            WebGl2RenderingContext::get_parameter(gl, WebGl2RenderingContext::MAX_TEXTURE_SIZE)?
                .as_f64()
                .unwrap_or(4096.0) as usize;

        let patches = if width <= max_tex_size as u64 && height <= max_tex_size as u64 {
            // can fit in one texture
            // bytes aligned
            match bitpix {
                Bitpix::I64 => {
                    // one must convert the data to i32
                    let bytes_from_i32 = bytes
                        .chunks(8)
                        .flat_map(|bytes| {
                            let l = i64::from_be_bytes([
                                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
                                bytes[6], bytes[7],
                            ]);
                            let i = l as i32;

                            i32::to_be_bytes(i)
                        })
                        .collect::<Vec<_>>();

                    let texture = Texture2D::create_from_raw_bytes::<R32I>(
                        gl,
                        width as i32,
                        height as i32,
                        TEX_PARAMS,
                        bytes_from_i32.as_slice(),
                    )?;

                    let mut sub_pixels = bytes_from_i32
                        .chunks(std::mem::size_of::<i32>())
                        .step_by(100)
                        .filter_map(|p| {
                            let p = i32::from_be_bytes([p[0], p[1], p[2], p[3]]) as f32;
                            if let Some(blank) = blank {
                                if p != blank {
                                    Some(p)
                                } else {
                                    None
                                }
                            } else {
                                Some(p)
                            }
                        })
                        .collect::<Vec<_>>();

                    let cuts = cuts::first_and_last_percent(&mut sub_pixels, 1, 99);
                    ImagePatches::new(
                        PixelType::R32I,
                        vec![texture],
                        cuts,
                        width as usize,
                        height as usize,
                    )
                }
                Bitpix::F64 => {
                    // one must convert the data to f32
                    let bytes_from_f32 = bytes
                        .chunks(8)
                        .flat_map(|bytes| {
                            let d = f64::from_be_bytes([
                                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
                                bytes[6], bytes[7],
                            ]);
                            let f = d as f32;

                            f32::to_be_bytes(f)
                        })
                        .collect::<Vec<_>>();

                    let texture = Texture2D::create_from_raw_bytes::<R32F>(
                        gl,
                        width as i32,
                        height as i32,
                        TEX_PARAMS,
                        bytes_from_f32.as_slice(),
                    )?;

                    let mut sub_pixels = bytes_from_f32
                        .chunks(std::mem::size_of::<f32>())
                        .step_by(100)
                        .filter_map(|p| {
                            let p = f32::from_be_bytes([p[0], p[1], p[2], p[3]]);
                            if p.is_finite() {
                                Some(p)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    let cuts = cuts::first_and_last_percent(&mut sub_pixels, 1, 99);
                    ImagePatches::new(
                        PixelType::R32F,
                        vec![texture],
                        cuts,
                        width as usize,
                        height as usize,
                    )
                }
                Bitpix::U8 => {
                    let texture = Texture2D::create_from_raw_bytes::<R8U>(
                        gl,
                        width as i32,
                        height as i32,
                        TEX_PARAMS,
                        bytes,
                    )?;

                    let mut sub_pixels = bytes
                        .iter()
                        .step_by(100)
                        .filter_map(|p| {
                            let p = *p as f32;
                            if let Some(blank) = blank {
                                if p != blank {
                                    Some(p)
                                } else {
                                    None
                                }
                            } else {
                                Some(p)
                            }
                        })
                        .collect::<Vec<_>>();

                    let cuts = cuts::first_and_last_percent(&mut sub_pixels, 1, 99);
                    ImagePatches::new(
                        PixelType::R8U,
                        vec![texture],
                        cuts,
                        width as usize,
                        height as usize,
                    )
                }
                Bitpix::I16 => {
                    let texture = Texture2D::create_from_raw_bytes::<R16I>(
                        gl,
                        width as i32,
                        height as i32,
                        TEX_PARAMS,
                        bytes,
                    )?;

                    let mut sub_pixels = bytes
                        .chunks(2)
                        .step_by(100)
                        .filter_map(|p| {
                            let p = i16::from_be_bytes([p[0], p[1]]) as f32;

                            if let Some(blank) = blank {
                                if p != blank {
                                    Some(p)
                                } else {
                                    None
                                }
                            } else {
                                Some(p)
                            }
                        })
                        .collect::<Vec<_>>();

                    let cuts = cuts::first_and_last_percent(&mut sub_pixels, 1, 99);
                    ImagePatches::new(
                        PixelType::R16I,
                        vec![texture],
                        cuts,
                        width as usize,
                        height as usize,
                    )
                }
                Bitpix::I32 => {
                    let texture = Texture2D::create_from_raw_bytes::<R32I>(
                        gl,
                        width as i32,
                        height as i32,
                        TEX_PARAMS,
                        bytes,
                    )?;

                    let mut sub_pixels = bytes
                        .chunks(4)
                        .step_by(100)
                        .filter_map(|p| {
                            let p = i32::from_be_bytes([p[0], p[1], p[2], p[3]]) as f32;

                            if let Some(blank) = blank {
                                if p != blank {
                                    Some(p)
                                } else {
                                    None
                                }
                            } else {
                                Some(p)
                            }
                        })
                        .collect::<Vec<_>>();

                    let cuts = cuts::first_and_last_percent(&mut sub_pixels, 1, 99);
                    ImagePatches::new(
                        PixelType::R32I,
                        vec![texture],
                        cuts,
                        width as usize,
                        height as usize,
                    )
                }
                Bitpix::F32 => {
                    let texture = Texture2D::create_from_raw_bytes::<R32F>(
                        gl,
                        width as i32,
                        height as i32,
                        TEX_PARAMS,
                        bytes,
                    )?;

                    let mut sub_pixels = bytes
                        .chunks(std::mem::size_of::<f32>())
                        .step_by(100)
                        .filter_map(|p| {
                            let p = f32::from_be_bytes([p[0], p[1], p[2], p[3]]);
                            if p.is_finite() {
                                Some(p)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    let cuts = cuts::first_and_last_percent(&mut sub_pixels, 1, 99);
                    ImagePatches::new(
                        PixelType::R32F,
                        vec![texture],
                        cuts,
                        width as usize,
                        height as usize,
                    )
                }
            }
        } else {
            // We cut the image in 4096x4096 patches. It is already 64MB to allocate for a f32 image of this dimensions.
            match bitpix {
                Bitpix::U8 => crop_image::<R8U>(gl, width, height, bytes, 4096, blank)?,
                Bitpix::I16 => crop_image::<R16I>(gl, width, height, bytes, 4096, blank)?,
                Bitpix::I32 => crop_image::<R32I>(gl, width, height, bytes, 4096, blank)?,
                Bitpix::F32 => crop_image::<R32F>(gl, width, height, bytes, 4096, blank)?,
                Bitpix::F64 => {
                    let bytes_from_f32 = bytes
                        .chunks(8)
                        .flat_map(|bytes| {
                            let d = f64::from_be_bytes([
                                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
                                bytes[6], bytes[7],
                            ]);
                            let f = d as f32;

                            f32::to_be_bytes(f)
                        })
                        .collect::<Vec<_>>();

                    crop_image::<R32F>(gl, width, height, &bytes_from_f32, 4096, blank)?
                }
                _ => {
                    return Err(JsValue::from_str(
                        "I64/F64 for big fits images not supported.",
                    ))
                }
            }
        };

        Self::init_buffers(gl.clone(), patches, wcs, bscale, bzero, blank, coo_sys)
    }

    pub fn from_rgba_bytes(
        gl: &WebGlContext,
        // bytes in TextureFormat
        bytes: &[u8],
        // wcs extracted from the image HDU
        wcs: fitsrs::WCS,
        // Coo sys of the view
        coo_sys: CooSystem,
    ) -> Result<Self, JsValue> {
        let dim = wcs.img_dimensions();
        let (width, height) = (dim[0] as u64, dim[1] as u64);

        let max_tex_size =
            WebGl2RenderingContext::get_parameter(gl, WebGl2RenderingContext::MAX_TEXTURE_SIZE)?
                .as_f64()
                .unwrap_or(4096.0) as usize;

        let bscale = 1.0;
        let bzero = 0.0;
        let blank = None;

        let image_patches = if width <= max_tex_size as u64 && height <= max_tex_size as u64 {
            // small image case, can fit into a webgl texture
            let textures = vec![Texture2D::create_from_raw_bytes::<RGBA8U>(
                gl,
                width as i32,
                height as i32,
                TEX_PARAMS,
                bytes,
            )?];
            let pixel_ty = PixelType::RGBA8U;
            let cuts = 0.0..1.0;

            ImagePatches::new(pixel_ty, textures, cuts, width as usize, height as usize)
        } else {
            crop_image::<RGBA8U>(gl, width, height, bytes, 4096, None)?
        };

        Self::init_buffers(
            gl.clone(),
            image_patches,
            wcs,
            bscale,
            bzero,
            blank,
            coo_sys,
        )
    }

    pub fn recompute_vertices(
        &mut self,
        camera: &CameraViewPort,
        projection: &ProjectionType,
    ) -> Result<(), JsValue> {
        let dim = self.wcs.img_dimensions();
        let (width, height) = (dim[0] as f64, dim[1] as f64);

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
            self.w_patch as u64,
            self.h_patch as u64,
            num_vertices,
            camera,
            &self.wcs,
            projection,
            self.pixel_type == PixelType::RGB8U || self.pixel_type == PixelType::RGBA8U,
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

            let dim = self.wcs.img_dimensions();
            let (width, height) = (dim[0] as usize, dim[1] as usize);

            // the camera coo system is not sync with the one in which the region
            // has been defined
            // let's redefine the region
            let center = self
                .wcs
                .unproj_lonlat(&ImgXY::new(
                    (width as f64 / 2.0) + 0.5,
                    (height as f64 / 2.0) + 0.5,
                ))
                .ok_or(JsValue::from_str("(w / 2, h / 2) px cannot be unprojected"))?;
            let center_xyz = center.to_xyz();
            let inside = crate::coosys::apply_coo_system(
                CooSystem::ICRS,
                self.coo_sys,
                &Vector3::new(center_xyz.y(), center_xyz.z(), center_xyz.x()),
            );

            let vertices = [
                self.wcs
                    .unproj_lonlat(&ImgXY::new(0.5, 0.5))
                    .ok_or(JsValue::from_str("(0, 0) does not lie in the sky"))?,
                self.wcs
                    .unproj_lonlat(&ImgXY::new(width as f64 - 0.5, 0.5))
                    .ok_or(JsValue::from_str("(w - 1, 0) does not lie in the sky"))?,
                self.wcs
                    .unproj_lonlat(&ImgXY::new(width as f64 - 0.5, height as f64 - 0.5))
                    .ok_or(JsValue::from_str("(w - 1, h - 1) does not lie in the sky"))?,
                self.wcs
                    .unproj_lonlat(&ImgXY::new(0.5, height as f64 - 0.5))
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
            opacity, blending, ..
        } = cfg;

        let shader = match self.pixel_type {
            PixelType::RGBA8U => crate::shader::get_shader(
                &self.gl,
                shaders,
                "image_base.vert",
                "image_sampler.frag",
            )?,
            PixelType::RGB8U => crate::shader::get_shader(
                &self.gl,
                shaders,
                "image_base.vert",
                "image_sampler.frag",
            )?,
            PixelType::R32F => {
                crate::shader::get_shader(&self.gl, shaders, "fits_base.vert", "fits_f32.frag")?
            }
            PixelType::R32I => {
                crate::shader::get_shader(&self.gl, shaders, "fits_base.vert", "fits_i32.frag")?
            }
            PixelType::R16I => {
                crate::shader::get_shader(&self.gl, shaders, "fits_base.vert", "fits_i16.frag")?
            }
            PixelType::R8U => {
                crate::shader::get_shader(&self.gl, shaders, "fits_base.vert", "fits_u8.frag")?
            }
        };

        //self.gl.disable(WebGl2RenderingContext::CULL_FACE);

        // 2. Draw it if its opacity is not null
        blending.enable(&self.gl, || {
            let mut off_indices = 0;
            for &idx_tex in self.idx_tex.iter() {
                let texture = &self.textures[idx_tex];
                let num_indices = self.num_indices[idx_tex] as i32;

                let shader_bound = shader.bind(&self.gl);

                shader_bound
                    .attach_uniforms_with_params_from(cfg, colormaps)
                    .attach_uniform("opacity", opacity)
                    .attach_uniform("tex", texture)
                    .attach_uniform("scale", &self.bscale)
                    .attach_uniform("offset", &self.bzero);

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

    #[inline]
    pub fn get_cuts(&self) -> &Range<f32> {
        &self.cuts
    }
}
