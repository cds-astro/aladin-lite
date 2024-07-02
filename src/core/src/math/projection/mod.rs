// Screen space: pixels space between
// * x_px in [0, width-1]
// * y_px in [0, height-1]

// Homogeneous space
// * x_h in [-1, 1]
// * y_h in [-1, 1]

// World space
use crate::camera::CameraViewPort;
use crate::domain::sdf::ProjDefType;

use coo_space::XYZWModel;
//use crate::num_traits::FloatConst;
use crate::math::PI;

use cgmath::Vector2;

pub mod coo_space;
pub mod domain;

use domain::{basic, full::FullScreen};

/* S <-> NDC space conversion methods */
pub fn screen_to_ndc_space(
    pos_screen_space: &XYScreen<f64>,
    camera: &CameraViewPort,
) -> XYNDC<f64> {
    // Screen space in pixels to homogeneous screen space (values between [-1, 1])
    let window_size = camera.get_screen_size();
    let window_size = Vector2::new(window_size.x as f64, window_size.y as f64);
    // Change of origin
    let dpi = camera.get_dpi() as f64;
    let origin = pos_screen_space * dpi - window_size / 2.0;

    // Scale to fit in [-1, 1]

    Vector2::new(
        2.0 * (origin.x / window_size.x),
        -2.0 * (origin.y / window_size.y),
    )
}

pub fn ndc_to_screen_space(
    pos_normalized_device: &XYNDC<f64>,
    camera: &CameraViewPort,
) -> XYScreen<f64> {
    let window_size = camera.get_screen_size();
    let dpi = camera.get_dpi() as f64;

    let pos_screen_space = Vector2::new(
        (pos_normalized_device.x * 0.5 + 0.5) * (window_size.x as f64),
        (0.5 - pos_normalized_device.y * 0.5) * (window_size.y as f64),
    );

    pos_screen_space / dpi
}

/* NDC <-> CLIP space conversion methods */
pub fn clip_to_ndc_space(pos_clip_space: &XYClip<f64>, camera: &CameraViewPort) -> XYNDC<f64> {
    let ndc_to_clip = camera.get_ndc_to_clip();
    let clip_zoom_factor = camera.get_clip_zoom_factor();

    Vector2::new(
        pos_clip_space.x / (ndc_to_clip.x * clip_zoom_factor),
        pos_clip_space.y / (ndc_to_clip.y * clip_zoom_factor),
    )
}

pub fn ndc_to_clip_space(
    pos_normalized_device: &XYNDC<f64>,
    camera: &CameraViewPort,
) -> XYClip<f64> {
    let ndc_to_clip = camera.get_ndc_to_clip();
    let clip_zoom_factor = camera.get_clip_zoom_factor();

    Vector2::new(
        pos_normalized_device.x * ndc_to_clip.x * clip_zoom_factor,
        pos_normalized_device.y * ndc_to_clip.y * clip_zoom_factor,
    )
}

/* S <-> CLIP space conversion methods */
pub fn clip_to_screen_space(
    pos_clip_space: &XYClip<f64>,
    camera: &CameraViewPort,
) -> XYScreen<f64> {
    let pos_normalized_device = clip_to_ndc_space(pos_clip_space, camera);
    ndc_to_screen_space(&pos_normalized_device, camera)
}

pub fn screen_to_clip_space(
    pos_screen_space: &XYScreen<f64>,
    camera: &CameraViewPort,
) -> XYClip<f64> {
    let pos_normalized_device = screen_to_ndc_space(pos_screen_space, camera);
    ndc_to_clip_space(&pos_normalized_device, camera)
}

use al_api::coo_system::CooSystem;

use crate::coo_space::{XYClip, XYZWWorld};

pub enum ProjectionType {
    // Zenithal projections
    /* TAN,      Gnomonic projection        */
    Tan(mapproj::zenithal::tan::Tan),
    /* STG,	     Stereographic projection   */
    Stg(mapproj::zenithal::stg::Stg),
    /* SIN,	     Orthographic		        */
    Sin(mapproj::zenithal::sin::Sin),
    /* ZEA,	     Equal-area 		        */
    Zea(mapproj::zenithal::zea::Zea),
    /* FEYE,     Fish-eyes                  */
    //Feye(mapproj::zenithal::feye::Feye),
    /* AIR,                                 */
    //Air(mapproj::zenithal::air::Air),
    //AZP: {fov: 180},
    //Azp(mapproj::zenithal::azp::Azp),
    /* ARC,                                 */
    //Arc(mapproj::zenithal::arc::Arc),
    /* NCP,                                 */
    //Ncp(mapproj::zenithal::ncp::Ncp),

    // Pseudo-cylindrical projections
    /* AIT,      Aitoff                     */
    Ait(mapproj::pseudocyl::ait::Ait),
    // MOL,      Mollweide                  */
    Mol(mapproj::pseudocyl::mol::Mol),
    // PAR,                                 */
    //Par(mapproj::pseudocyl::par::Par),
    // SFL,                                 */
    //Sfl(mapproj::pseudocyl::sfl::Sfl),

    // Cylindrical projections
    // MER,      Mercator                   */
    Mer(mapproj::cylindrical::mer::Mer),
    // CAR,                                 */
    //Car(mapproj::cylindrical::car::Car),
    // CEA,                                 */
    //Cea(mapproj::cylindrical::cea::Cea),
    // CYP,                                 */
    //Cyp(mapproj::cylindrical::cyp::Cyp),

    // Conic projections
    // COD,                                 */
    //Cod(mapproj::conic::cod::Cod),

    // HEALPix hybrid projection
    //Hpx(mapproj::hybrid::hpx::Hpx),
}

use crate::math::lonlat::LonLat;
impl ProjectionType {
    pub fn north_pole_celestial_space(&self, camera: &CameraViewPort) -> LonLatT<f64> {
        // This is always defined
        let np_world = self.north_pole_world_space();

        let np_celestial = camera.get_w2m() * np_world;
        np_celestial.lonlat()
    }

    /// Screen to model space deprojection

    /// Perform a screen to the world space deprojection
    ///
    /// # Arguments
    ///
    /// * ``pos_screen_space`` - The position in the screen pixel space (top-left of the screen being the origin
    /// * ``camera`` - The camera object
    pub fn screen_to_world_space(
        &self,
        pos_screen_space: &XYScreen<f64>,
        camera: &CameraViewPort,
    ) -> Option<XYZWWorld<f64>> {
        // Change the screen position according to the dpi
        //let dpi = camera.get_dpi();
        let pos_screen_space = *pos_screen_space;
        let pos_normalized_device = screen_to_ndc_space(&pos_screen_space, camera);

        let pos_clip_space = ndc_to_clip_space(&pos_normalized_device, camera);
        self.clip_to_world_space(&pos_clip_space)
    }

    /// Screen to model space deprojection

    /// Perform a screen to the world space deprojection
    ///
    /// # Arguments
    ///
    /// * ``pos_screen_space`` - The position in the screen pixel space (top-left of the screen being the origin
    /// * ``camera`` - The camera object
    pub fn screen_to_model_space(
        &self,
        pos_screen_space: &XYScreen<f64>,
        camera: &CameraViewPort,
    ) -> Option<XYZWModel<f64>> {
        self.screen_to_world_space(pos_screen_space, camera)
            .map(|world_pos| camera.get_w2m() * world_pos)
    }

    pub fn normalized_device_to_model_space(
        &self,
        ndc_pos: &XYNDC<f64>,
        camera: &CameraViewPort,
    ) -> Option<XYZWModel<f64>> {
        self.normalized_device_to_world_space(ndc_pos, camera)
            .map(|world_pos| camera.get_w2m() * world_pos)
    }

    pub fn model_to_screen_space(
        &self,
        pos_model_space: &XYZWModel<f64>,
        camera: &CameraViewPort,
    ) -> Option<XYScreen<f64>> {
        let m2w = camera.get_m2w();
        let pos_world_space = m2w * pos_model_space;
        self.world_to_screen_space(&pos_world_space, camera)
    }

    pub fn icrs_celestial_to_screen_space(
        &self,
        icrs_celestial_pos: &XYZWModel<f64>,
        camera: &CameraViewPort,
    ) -> Option<XYScreen<f64>> {
        self.icrs_celestial_to_normalized_device_space(icrs_celestial_pos, camera)
            .map(|ndc_pos| crate::ndc_to_screen_space(&ndc_pos, camera))
    }

    pub fn icrs_celestial_to_normalized_device_space(
        &self,
        icrs_celestial_pos: &XYZWModel<f64>,
        camera: &CameraViewPort,
    ) -> Option<XYNDC<f64>> {
        let view_coosys = camera.get_coo_system();
        let c = CooSystem::ICRS.to::<f64>(view_coosys);

        let m2w = camera.get_m2w();
        let pos_world_space = m2w * c * icrs_celestial_pos;
        self.world_to_normalized_device_space(&pos_world_space, camera)
    }

    pub fn model_to_normalized_device_space(
        &self,
        pos_model_space: &XYZWModel<f64>,
        camera: &CameraViewPort,
    ) -> Option<XYNDC<f64>> {
        let m2w = camera.get_m2w();
        let pos_world_space = m2w * pos_model_space;
        self.world_to_normalized_device_space(&pos_world_space, camera)
    }

    pub fn model_to_clip_space(
        &self,
        pos_model_space: &XYZWModel<f64>,
        camera: &CameraViewPort,
    ) -> Option<XYClip<f64>> {
        let m2w = camera.get_m2w();
        let pos_world_space = m2w * pos_model_space;
        self.world_to_clip_space(&pos_world_space)
    }

    /// World to screen space projection

    /// World to screen space transformation
    ///
    /// # Arguments
    ///
    /// * `x` - X mouse position in homogenous screen space (between [-1, 1])
    /// * `y` - Y mouse position in homogenous screen space (between [-1, 1])
    pub fn world_to_normalized_device_space(
        &self,
        pos_world_space: &XYZWWorld<f64>,
        camera: &CameraViewPort,
    ) -> Option<XYNDC<f64>> {
        self.world_to_clip_space(pos_world_space)
            .map(|pos_clip_space| clip_to_ndc_space(&pos_clip_space, camera))
    }

    pub fn normalized_device_to_world_space(
        &self,
        ndc_pos: &XYNDC<f64>,
        camera: &CameraViewPort,
    ) -> Option<XYZWWorld<f64>> {
        let clip_pos = ndc_to_clip_space(ndc_pos, camera);
        self.clip_to_world_space(&clip_pos)
    }

    pub fn world_to_screen_space(
        &self,
        pos_world_space: &XYZWWorld<f64>,
        camera: &CameraViewPort,
    ) -> Option<XYScreen<f64>> {
        self.world_to_normalized_device_space(pos_world_space, camera)
            .map(|pos_normalized_device| ndc_to_screen_space(&pos_normalized_device, camera))
    }

    /*pub(crate) fn is_allsky(&self) -> bool {
        match self {
            ProjectionType::Sin(_) | ProjectionType::Tan(_) => false,
            //| ProjectionType::Feye(_)
            //| ProjectionType::Ncp(_) => false,
            _ => true,
        }
    }*/

    pub fn bounds_size_ratio(&self) -> f64 {
        match self {
            // Zenithal projections
            /* TAN,      Gnomonic projection        */
            ProjectionType::Tan(_) => 1.0,
            /* STG,	     Stereographic projection   */
            ProjectionType::Stg(_) => 1.0,
            /* SIN,	     Orthographic		        */
            ProjectionType::Sin(_) => 1.0,
            /* ZEA,	     Equal-area 		        */
            ProjectionType::Zea(_) => 1.0,
            /* FEYE,     Fish-eyes                  */
            //ProjectionType::Feye(_) => 1.0,
            /* AIR,                                 */
            //ProjectionType::Air(_) => 1.0,
            //AZP: {fov: 180},
            //Azp(mapproj::zenithal::azp::Azp),
            /* ARC,                                 */
            //ProjectionType::Arc(_) => 1.0,
            /* NCP,                                 */
            //ProjectionType::Ncp(_) => 1.0,

            // Pseudo-cylindrical projections
            /* AIT,      Aitoff                     */
            ProjectionType::Ait(_) => 2.0,
            // MOL,      Mollweide                  */
            ProjectionType::Mol(_) => 2.0,
            // PAR,                                 */
            //ProjectionType::Par(_) => 2.0,
            // SFL,                                 */
            //ProjectionType::Sfl(_) => 2.0,

            // Cylindrical projections
            // MER,      Mercator                   */
            ProjectionType::Mer(_) => 1.0,
            // CAR,                                 */
            //ProjectionType::Car(_) => 1.0,
            // CEA,                                 */
            //ProjectionType::Cea(_) => 1.0,
            // CYP,                                 */
            //ProjectionType::Cyp(_) => 1.0,

            // Conic projections
            // COD,                                 */
            //ProjectionType::Cod(_) => 1.0,

            // HEALPix hybrid projection
            //ProjectionType::Hpx(_) => 2.0,
        }
    }

    pub fn aperture_start(&self) -> f64 {
        match self {
            // Zenithal projections
            /* TAN,      Gnomonic projection        */
            ProjectionType::Tan(_) => 150.0,
            /* STG,	     Stereographic projection   */
            ProjectionType::Stg(_) => 360.0,
            /* SIN,	     Orthographic		        */
            ProjectionType::Sin(_) => 180.0,
            /* ZEA,	     Equal-area 		        */
            ProjectionType::Zea(_) => 360.0,
            /* FEYE,     Fish-eyes                  */
            //ProjectionType::Feye(_) => 190.0,
            /* AIR,                                 */
            //ProjectionType::Air(_) => 360.0,
            //AZP: {fov: 180},
            //Azp(mapproj::zenithal::azp::Azp),
            /* ARC,                                 */
            //ProjectionType::Arc(_) => 360.0,
            /* NCP,                                 */
            //ProjectionType::Ncp(_) => 180.0,

            // Pseudo-cylindrical projections
            /* AIT,      Aitoff                     */
            ProjectionType::Ait(_) => 360.0,
            // MOL,      Mollweide                  */
            ProjectionType::Mol(_) => 360.0,
            // PAR,                                 */
            //ProjectionType::Par(_) => 360.0,
            // SFL,                                 */
            //ProjectionType::Sfl(_) => 360.0,

            // Cylindrical projections
            // MER,      Mercator                   */
            ProjectionType::Mer(_) => 360.0,
            // CAR,                                 */
            //ProjectionType::Car(_) => 360.0,
            // CEA,                                 */
            //ProjectionType::Cea(_) => 360.0,
            // CYP,                                 */
            //ProjectionType::Cyp(_) => 360.0,

            // Conic projections
            // COD,                                 */
            //ProjectionType::Cod(_) => 330.0,

            // HEALPix hybrid projection
            //ProjectionType::Hpx(_) => 360.0,
        }
    }

    pub fn get_area(&self) -> &ProjDefType {
        match self {
            // Zenithal projections
            /* TAN,      Gnomonic projection        */
            ProjectionType::Tan(_) => {
                const FULL_SCREEN: ProjDefType = ProjDefType::FullScreen(FullScreen);
                &FULL_SCREEN
            }
            /* STG,	     Stereographic projection   */
            ProjectionType::Stg(_) => {
                const DISK: ProjDefType = ProjDefType::FullScreen(FullScreen);
                &DISK
            }
            /* SIN,	     Orthographic		        */
            ProjectionType::Sin(_) => {
                const DISK: ProjDefType = ProjDefType::Disk(basic::disk::Disk { radius: 1.0 });
                &DISK
            }
            /* ZEA,	     Equal-area 		        */
            ProjectionType::Zea(_) => {
                const DISK: ProjDefType = ProjDefType::Disk(basic::disk::Disk { radius: 1.0 });
                &DISK
            }
            /* FEYE,     Fish-eyes                  */
            /*ProjectionType::Feye(_) => {
                const DISK: ProjDefType = ProjDefType::Disk(basic::disk::Disk { radius: 1.0 });
                &DISK
            }*/
            /* AIR,                                 */
            /*ProjectionType::Air(_) => {
                const DISK: ProjDefType = ProjDefType::FullScreen(FullScreen);
                &DISK
            }*/
            //AZP: {fov: 180},
            //Azp(mapproj::zenithal::azp::Azp),
            /* ARC,                                 */
            /*ProjectionType::Arc(_) => {
                const DISK: ProjDefType = ProjDefType::Disk(basic::disk::Disk { radius: 1.0 });
                &DISK
            }*/
            /* NCP,                                 */
            /*ProjectionType::Ncp(_) => {
                const DISK: ProjDefType = ProjDefType::Disk(basic::disk::Disk { radius: 1.0 });
                &DISK
            }*/

            // Pseudo-cylindrical projections
            /* AIT,      Aitoff                     */
            ProjectionType::Ait(_) => {
                const ELLIPSE: ProjDefType = ProjDefType::Disk(basic::disk::Disk { radius: 1.0 });
                &ELLIPSE
            }
            // MOL,      Mollweide                  */
            ProjectionType::Mol(_) => {
                const ELLIPSE: ProjDefType = ProjDefType::Disk(basic::disk::Disk { radius: 1.0 });
                &ELLIPSE
            }
            // PAR,                                 */
            /*ProjectionType::Par(_) => {
                const PAR: ProjDefType = ProjDefType::Par(Par);
                &PAR
            }*/
            // SFL,                                 */
            /*ProjectionType::Sfl(_) => {
                const PAR: ProjDefType = ProjDefType::Par(Par);
                &PAR
            }*/

            // Cylindrical projections
            // MER,      Mercator                   */
            ProjectionType::Mer(_) => {
                const FULL_SCREEN: ProjDefType = ProjDefType::FullScreen(FullScreen);
                &FULL_SCREEN
            } // CAR,                                 */
              /*ProjectionType::Car(_) => {
                  const FULL_SCREEN: ProjDefType = ProjDefType::FullScreen(FullScreen);
                  &FULL_SCREEN
              }*/
              // CEA,                                 */
              /*ProjectionType::Cea(_) => {
                  const FULL_SCREEN: ProjDefType = ProjDefType::FullScreen(FullScreen);
                  &FULL_SCREEN
              }*/
              // CYP,                                 */
              /*ProjectionType::Cyp(_) => {
                  const FULL_SCREEN: ProjDefType = ProjDefType::FullScreen(FullScreen);
                  &FULL_SCREEN
              }*/

              // Conic projections
              // COD,                                 */
              /*ProjectionType::Cod(_) => {
                  const CONIC: ProjDefType = ProjDefType::Cod(Cod::new());
                  &CONIC
              }*/
              // HEALPix hybrid projection
              /*ProjectionType::Hpx(_) => {
                  const HPX_DEF_REG: ProjDefType = ProjDefType::Hpx(Hpx);
                  &HPX_DEF_REG
              }*/
        }
    }
}

impl Projection for ProjectionType {
    /// Deprojection
    fn clip_to_world_space(&self, xy: &XYClip<f64>) -> Option<XYZWWorld<f64>> {
        match self {
            // Zenithal projections
            /* TAN,      Gnomonic projection        */
            ProjectionType::Tan(tan) => tan.clip_to_world_space(xy),
            /* STG,	     Stereographic projection   */
            ProjectionType::Stg(stg) => stg.clip_to_world_space(xy),
            /* SIN,	     Orthographic		        */
            ProjectionType::Sin(sin) => sin.clip_to_world_space(xy),
            /* ZEA,	     Equal-area 		        */
            ProjectionType::Zea(zea) => zea.clip_to_world_space(xy),
            /* FEYE,     Fish-eyes                  */
            //ProjectionType::Feye(feye) => feye.clip_to_world_space(xy),
            /* AIR,                                 */
            //ProjectionType::Air(air) => air.clip_to_world_space(xy),
            //AZP: {fov: 180},
            //Azp(mapproj::zenithal::azp::Azp),
            /* ARC,                                 */
            //ProjectionType::Arc(arc) => arc.clip_to_world_space(xy),
            /* NCP,                                 */
            //ProjectionType::Ncp(ncp) => ncp.clip_to_world_space(xy),

            // Pseudo-cylindrical projections
            /* AIT,      Aitoff                     */
            ProjectionType::Ait(ait) => ait.clip_to_world_space(xy),
            // MOL,      Mollweide                  */
            ProjectionType::Mol(mol) => mol.clip_to_world_space(xy),
            // PAR,                                 */
            //ProjectionType::Par(par) => par.clip_to_world_space(xy),
            // SFL,                                 */
            //ProjectionType::Sfl(sfl) => sfl.clip_to_world_space(xy),

            // Cylindrical projections
            // MER,      Mercator                   */
            ProjectionType::Mer(mer) => mer.clip_to_world_space(xy),
            // CAR,                                 */
            //ProjectionType::Car(car) => car.clip_to_world_space(xy),
            // CEA,                                 */
            //ProjectionType::Cea(cea) => cea.clip_to_world_space(xy),
            // CYP,                                 */
            //ProjectionType::Cyp(cyp) => cyp.clip_to_world_space(xy),

            // Conic projections
            // COD,                                 */
            /*ProjectionType::Cod(cod) => cod.clip_to_world_space(xy).map(|xyzw| {
                let rot = Rotation::from_sky_position(
                    &LonLatT::new(0.0_f64.to_angle(), (HALF_PI * 0.5).to_angle()).vector(),
                );
                rot.inv_rotate(&xyzw)
            }),*/
            // HEALPix hybrid projection
            //ProjectionType::Hpx(hpx) => hpx.clip_to_world_space(xy),
        }
    }

    // Projection
    fn world_to_clip_space(&self, xyzw: &XYZWWorld<f64>) -> Option<XYClip<f64>> {
        match self {
            // Zenithal projections
            /* TAN,      Gnomonic projection        */
            ProjectionType::Tan(tan) => tan.world_to_clip_space(xyzw),
            /* STG,	     Stereographic projection   */
            ProjectionType::Stg(stg) => stg.world_to_clip_space(xyzw),
            /* SIN,	     Orthographic		        */
            ProjectionType::Sin(sin) => sin.world_to_clip_space(xyzw),
            /* ZEA,	     Equal-area 		        */
            ProjectionType::Zea(zea) => zea.world_to_clip_space(xyzw),
            /* FEYE,     Fish-eyes                  */
            //ProjectionType::Feye(feye) => feye.world_to_clip_space(xyzw),
            /* AIR,                                 */
            //ProjectionType::Air(air) => air.world_to_clip_space(xyzw),
            //AZP: {fov: 180},
            //Azp(mapproj::zenithal::azp::Azp),
            /* ARC,                                 */
            //ProjectionType::Arc(arc) => arc.world_to_clip_space(xyzw),
            /* NCP,                                 */
            //ProjectionType::Ncp(ncp) => ncp.world_to_clip_space(xyzw),

            // Pseudo-cylindrical projections
            /* AIT,      Aitoff                     */
            ProjectionType::Ait(ait) => ait.world_to_clip_space(xyzw),
            // MOL,      Mollweide                  */
            ProjectionType::Mol(mol) => mol.world_to_clip_space(xyzw),
            // PAR,                                 */
            //ProjectionType::Par(par) => par.world_to_clip_space(xyzw),
            // SFL,                                 */
            //ProjectionType::Sfl(sfl) => sfl.world_to_clip_space(xyzw),

            // Cylindrical projections
            // MER,      Mercator                   */
            ProjectionType::Mer(mer) => mer.world_to_clip_space(xyzw),
            // CAR,                                 */
            //ProjectionType::Car(car) => car.world_to_clip_space(xyzw),
            // CEA,                                 */
            //ProjectionType::Cea(cea) => cea.world_to_clip_space(xyzw),
            // CYP,                                 */
            //ProjectionType::Cyp(cyp) => cyp.world_to_clip_space(xyzw),
            // Conic projections
            // COD,                                 */
            /*ProjectionType::Cod(cod) => {
                // The Cod projection is centered on (0, 45 deg)
                let rot = Rotation::from_sky_position(
                    &LonLatT::new(0.0_f64.to_angle(), (HALF_PI * 0.5).to_angle()).vector(),
                );
                cod.world_to_clip_space(&rot.rotate(&xyzw))
            }*/
            // HEALPix hybrid projection
            //ProjectionType::Hpx(hpx) => hpx.world_to_clip_space(xyzw),
        }
    }
}

use al_core::shader::UniformType;
use al_core::WebGlContext;
use web_sys::WebGlUniformLocation;
impl UniformType for ProjectionType {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        match value {
            // Zenithal projections
            /* TAN,      Gnomonic projection        */
            ProjectionType::Tan(_) => gl.uniform1i(location, 0),
            /* STG,	     Stereographic projection   */
            ProjectionType::Stg(_) => gl.uniform1i(location, 1),
            /* SIN,	     Orthographic		        */
            ProjectionType::Sin(_) => gl.uniform1i(location, 2),
            /* ZEA,	     Equal-area 		        */
            ProjectionType::Zea(_) => gl.uniform1i(location, 3),

            // Pseudo-cylindrical projections
            /* AIT,      Aitoff                     */
            ProjectionType::Ait(_) => gl.uniform1i(location, 4),
            // MOL,      Mollweide                  */
            ProjectionType::Mol(_) => gl.uniform1i(location, 5),

            // Cylindrical projections
            // MER,      Mercator                   */
            ProjectionType::Mer(_) => gl.uniform1i(location, 6),
        }
    }
}

use cgmath::Vector4;

use mapproj::CanonicalProjection;
pub trait Projection {
    /// Perform a clip to the world space deprojection
    ///
    /// # Arguments
    ///
    /// * ``pos_clip_space`` - The position in the clipping space (orthonorlized space)
    fn clip_to_world_space(&self, xy_clip: &XYClip<f64>) -> Option<XYZWWorld<f64>>;
    /// World to the clipping space deprojection
    ///
    /// # Arguments
    ///
    /// * ``pos_world_space`` - The position in the world space
    fn world_to_clip_space(&self, pos_world_space: &XYZWWorld<f64>) -> Option<XYClip<f64>>;

    /// (`alpha_p`, `delta_p`) in the WCS II paper from Mark Calabretta.
    #[inline]
    fn north_pole_world_space(&self) -> XYZWWorld<f64> {
        // This is always defined
        self.clip_to_world_space(&XYClip::new(0.0, 1.0 - 1e-5))
            .unwrap()
    }

    #[inline]
    fn south_pole_world_space(&self) -> XYZWWorld<f64> {
        // This is always defined
        self.clip_to_world_space(&XYClip::new(0.0, -1.0 + 1e-5))
            .unwrap()
    }
}

use mapproj::ProjXY;

use self::coo_space::XYScreen;
use self::coo_space::XYNDC;

use super::lonlat::LonLatT;

impl<'a, P> Projection for &'a P
where
    P: CanonicalProjection,
{
    /// Perform a clip to the world space deprojection
    ///
    /// # Arguments
    ///
    /// * ``pos_clip_space`` - The position in the clipping space (orthonorlized space)
    fn clip_to_world_space(&self, xy_clip: &XYClip<f64>) -> Option<XYZWWorld<f64>> {
        let proj_bounds = self.bounds();
        // Scale the xy_clip space so that it maps the proj definition domain of mapproj
        let xy_mapproj = {
            let x_proj_bounds = proj_bounds.x_bounds().as_ref().unwrap_or(&(-PI..=PI));

            let y_proj_bounds = proj_bounds.y_bounds().as_ref().unwrap_or(&(-PI..=PI));

            let x_len = x_proj_bounds.end() - x_proj_bounds.start();
            let y_len = y_proj_bounds.end() - y_proj_bounds.start();

            let y_mean = (y_proj_bounds.end() + y_proj_bounds.start()) * 0.5;

            let x_off = x_proj_bounds.start();
            let y_off = y_proj_bounds.start();

            ProjXY::new(
                (xy_clip.x * 0.5 + 0.5) * x_len + x_off,
                (xy_clip.y * 0.5 + 0.5) * y_len + y_off - y_mean,
            )
        };
        self.unproj(&xy_mapproj).map(|xyz_mapproj| {
            // Xmpp <-> Zal
            // -Ympp <-> Xal
            // Zmpp <-> Yal
            Vector4::new(-xyz_mapproj.y(), xyz_mapproj.z(), xyz_mapproj.x(), 1.0)
        })
    }
    /// World to the clipping space deprojection
    ///
    /// # Arguments
    ///
    /// * ``pos_world_space`` - The position in the world space
    fn world_to_clip_space(&self, pos_world_space: &XYZWWorld<f64>) -> Option<XYClip<f64>> {
        // Xmpp <-> Zal
        // -Ympp <-> Xal
        // Zmpp <-> Yal
        let xyz_mapproj = mapproj::XYZ::new_renorming_if_necessary(
            pos_world_space.z,
            -pos_world_space.x,
            pos_world_space.y,
        );

        self.proj(&xyz_mapproj).map(|xy_clip_mapproj| {
            let proj_bounds = self.bounds();
            // Scale the xy_clip space so that it maps the proj definition domain of mapproj
            let x_proj_bounds = proj_bounds.x_bounds().as_ref().unwrap_or(&(-PI..=PI));

            let y_proj_bounds = proj_bounds.y_bounds().as_ref().unwrap_or(&(-PI..=PI));

            let x_len = x_proj_bounds.end() - x_proj_bounds.start();
            let y_len = y_proj_bounds.end() - y_proj_bounds.start();

            let x_off = x_proj_bounds.start();
            let y_off = y_proj_bounds.start();

            let y_mean = (y_proj_bounds.end() + y_proj_bounds.start()) * 0.5;

            XYClip::new(
                (((xy_clip_mapproj.x() - x_off) / x_len) - 0.5) * 2.0,
                (((xy_clip_mapproj.y() - y_off + y_mean) / y_len) - 0.5) * 2.0,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn generate_maps() {
        use super::*;
        use cgmath::InnerSpace;
        use cgmath::Vector2;
        use image_decoder::{Rgb, RgbImage};

        use crate::Abort;

        fn generate_projection_map(filename: &str, projection: ProjectionType) {
            let (w, h) = (512.0, 512.0);
            let mut img = RgbImage::new(w as u32, h as u32);
            for x in 0..(w as u32) {
                for y in 0..(h as u32) {
                    let xy = Vector2::new(x, y);
                    let clip_xy = Vector2::new(
                        2.0 * ((xy.x as f64) / (w as f64)) - 1.0,
                        2.0 * ((xy.y as f64) / (h as f64)) - 1.0,
                    );
                    let rgb = if let Some(pos) = projection.clip_to_world_space(&clip_xy) {
                        let pos = pos.truncate().normalize();
                        Rgb([
                            ((pos.x * 0.5 + 0.5) * 256.0) as u8,
                            ((pos.y * 0.5 + 0.5) * 256.0) as u8,
                            ((pos.z * 0.5 + 0.5) * 256.0) as u8,
                        ])
                    } else {
                        Rgb([255, 255, 255])
                    };

                    img.put_pixel(x as u32, y as u32, rgb);
                }
            }
            img.save(filename).unwrap_abort();
        }

        // Zenithal
        generate_projection_map(
            "./../img/tan.jpg",
            ProjectionType::Tan(mapproj::zenithal::tan::Tan),
        );
        generate_projection_map(
            "./../img/stg.jpg",
            ProjectionType::Stg(mapproj::zenithal::stg::Stg),
        );
        generate_projection_map(
            "./../img/sin.jpg",
            ProjectionType::Sin(mapproj::zenithal::sin::Sin),
        );
        generate_projection_map(
            "./../img/zea.jpg",
            ProjectionType::Zea(mapproj::zenithal::zea::Zea),
        );
        /*generate_projection_map(
            "./../img/feye.png",
            ProjectionType::Feye(mapproj::zenithal::feye::Feye),
        );
        generate_projection_map(
            "./../img/arc.png",
            ProjectionType::Arc(mapproj::zenithal::arc::Arc),
        );
        generate_projection_map(
            "./../img/ncp.png",
            ProjectionType::Ncp(mapproj::zenithal::ncp::Ncp),
        );
        generate_projection_map(
            "./../img/air.png",
            ProjectionType::Air(mapproj::zenithal::air::Air::new()),
        );*/

        // Cylindrical
        generate_projection_map(
            "./../img/mer.jpg",
            ProjectionType::Mer(mapproj::cylindrical::mer::Mer),
        );
        /*generate_projection_map(
            "./../img/car.png",
            ProjectionType::Car(mapproj::cylindrical::car::Car),
        );
        generate_projection_map(
            "./../img/cea.png",
            ProjectionType::Cea(mapproj::cylindrical::cea::Cea::new()),
        );
        generate_projection_map(
            "./../img/cyp.png",
            ProjectionType::Cyp(mapproj::cylindrical::cyp::Cyp::new()),
        );*/
        // Pseudo-cylindrical
        generate_projection_map(
            "./../img/ait.jpg",
            ProjectionType::Ait(mapproj::pseudocyl::ait::Ait),
        );
        /*generate_projection_map(
            "./../img/car.png",
            ProjectionType::Par(mapproj::pseudocyl::par::Par),
        );
        generate_projection_map(
            "./../img/cea.png",
            ProjectionType::Sfl(mapproj::pseudocyl::sfl::Sfl),
        );*/
        generate_projection_map(
            "./../img/mol.jpg",
            ProjectionType::Mol(mapproj::pseudocyl::mol::Mol::new()),
        );
        // Conic
        /*generate_projection_map(
            "./../img/cod.png",
            ProjectionType::Cod(mapproj::conic::cod::Cod::new()),
        );
        // Hybrid
        generate_projection_map(
            "./../img/hpx.png",
            ProjectionType::Hpx(mapproj::hybrid::hpx::Hpx),
        );*/
    }
}
