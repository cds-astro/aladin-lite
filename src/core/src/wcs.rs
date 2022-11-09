use fitsrs::PrimaryHeader;
use cgmath::{Matrix2, Matrix4, Matrix, Vector2, Vector4, SquareMatrix};

use al_core::image::fits::FitsBorrowed;
use crate::{Gnomonic, Orthographic};
use crate::math::projection::Projection;

// Number of axis
const NAXIS: u8 = 2;
/* Implementation of the paper from M. R. Calabretta and E. W. Greisen
   Representations of celestial coordinates in FITS */
#[derive(Debug)]
pub struct WCS2 {
    // Pixel coordinates of a coordinate reference point (r_j)
    crpix: Vector2<f64>,
    // Pixel to interm coordianates keywords
    pixel_interm_params: Pixel2IntermParams,
    // Coordinate value at reference point [deg]
    crval: Vector2<f64>,
    // Coordinate reference (radec, galactic lonlat, ...)
    ctype_coo_ref: CTYPE_COO_REF,
    // Spherical projection
    ctype_proj: CTYPE_PROJ,
    // Native longitude of celestial pole [deg]
    lonpole: f64,
    // Native latitude of celestial pole [deg]
    latpole: f64,
    // Coordinate system
    radesys: RADESYS,

    // Coordinate rotation matrix coding the native to celestial spherical coordinates
    r: Matrix4<f64>,
    // Inv of r
    r_inv: Matrix4<f64>
}

#[derive(Debug)]
enum Pixel2IntermParams {
    CDELT_PC {
        // Coordinate increment at reference point [deg] (s_i)
        cdelt: Vector2<f64>,
        // Linear transformation matrix (m_ij)
        pc: Matrix2<f64>,
        // Inv of the linear transformation matrix (m_ij)
        pc_inv: Matrix2<f64>,
    },
    CD {
        // Linear transformation matrix (s_i x m_ij)
        cd: Matrix2<f64>,
        // Inv of linear transformation matrix (s_i x m_ij)
        cd_inv: Matrix2<f64>,
    },
    CROTA
}

use fitsrs::{FITSCardValue, FITSCard};
fn has_specific_card<'a>(hdu: &'a PrimaryHeader<'a>, keyword: &'static str) -> bool {
    hdu.get(keyword).is_some()
}

fn get_card_float_value<'a>(hdu: &'a PrimaryHeader<'a>, keyword: &'static str, default: f64) -> Result<f64, &'static str> {
    match hdu.get(keyword) {
        None => Ok(default),
        Some(FITSCard::Other { value: FITSCardValue::FloatingPoint(v), .. }) => Ok(*v),
        _ => Err("Keyword does not refer to a floating point value"),
    }
}
fn get_card_str_value<'a>(hdu: &'a PrimaryHeader<'a>, keyword: &'static str) -> Result<&'a str, &'static str> {
    match hdu.get(keyword) {
        Some(FITSCard::Other { value: FITSCardValue::CharacterString(s), .. }) => Ok(s),
        _ => Err("Keyword does not refer to a string"),
    }
}

// (Y-X'-Y'') Euler angles matrix
// See: https://en.wikipedia.org/wiki/Euler_angles#Rotation_matrix
fn create_yxy_euler_rotation_matrix(alpha: f64, delta: f64, phi: f64) -> Matrix4<f64> {
    let (s1, c1) = phi.sin_cos();
    let (s2, c2) = delta.sin_cos();
    let (s3, c3) = alpha.sin_cos();

    let r11 = c1*c3 - c2*s1*s3;
    let r12 = s1*s2;
    let r13 = c1*s3 + c2*c3*s1;

    let r21 = s2*s3;
    let r22 = c2;
    let r23 = -c3*s2;

    let r31 = -c3*s1 - c1*c2*s3;
    let r32 = c1*s2;
    let r33 = c1*c2*c3 - s1*s3;

    Matrix4::new(
        r11, r21, r31, 0.0,
        r12, r22, r32, 0.0,
        r13, r23, r33, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

impl WCS2 {
    pub fn new<'a>(fits: &FitsBorrowed<'a>) -> Result<Self, &'static str> {
        let hdu = fits.get_header();
        let crpix1 = get_card_float_value(hdu, "CRPIX1", 0.0)?;
        let crpix2 = get_card_float_value(hdu, "CRPIX2", 0.0)?;

        let pixel_interm_params = (if has_specific_card(hdu, "CDELT1") {
            let cdelt1 = get_card_float_value(hdu, "CDELT1", 1.0)?;
            let cdelt2 = get_card_float_value(hdu, "CDELT2", 1.0)?;

            // Linear transformation matrix (m_ij)
            let pc11 = get_card_float_value(hdu, "PC1_1", 1.0)?;
            let pc12 = get_card_float_value(hdu, "PC1_2", 0.0)?;
            let pc21 = get_card_float_value(hdu, "PC2_1", 0.0)?;
            let pc22 = get_card_float_value(hdu, "PC2_2", 1.0)?;

            let pc = Matrix2::new(pc11, pc21, pc12, pc22);
            let pc_inv = pc.transpose();

            Ok(Pixel2IntermParams::CDELT_PC {
                cdelt: Vector2::new(cdelt1.to_radians(), cdelt2.to_radians()),
                pc: pc,
                pc_inv: pc_inv
            })
        } else if has_specific_card(hdu, "CD1_1") {
            // Linear transformation matrix (m_ij * s_i)
            let cd11 = get_card_float_value(hdu, "CD1_1", 1.0)?;
            let cd12 = get_card_float_value(hdu, "CD1_2", 0.0)?;
            let cd21 = get_card_float_value(hdu, "CD2_1", 0.0)?;
            let cd22 = get_card_float_value(hdu, "CD2_2", 1.0)?;

            let cd = Matrix2::new(cd11, cd21, cd12, cd22);
            let cd_inv = cd.transpose();

            Ok(Pixel2IntermParams::CD {
                cd,
                cd_inv
            })
        } else if has_specific_card(hdu, "CROTA") {
            Err("CROTA not implemented")
        } else {
            // Default case
            let pc = Matrix2::identity();

            Ok(Pixel2IntermParams::CDELT_PC {
                cdelt: Vector2::new(1.0_f64.to_radians(), 1.0_f64.to_radians()),
                pc_inv: pc.clone(),
                pc
            })
        })?;
        

        let crval1 = get_card_float_value(hdu, "CRVAL1", 0.0)?;
        let crval2 = get_card_float_value(hdu, "CRVAL2", 0.0)?;

        let ctype1 = get_card_str_value(hdu, "CTYPE1")?;
        let ctype2 = get_card_str_value(hdu, "CTYPE2")?;

        let ctype_proj = match &ctype1[5..8] {
            /* Zenithal projections */ 
            // zenithal/azimuthal perspective
            "AZP" => Ok(CTYPE_PROJ::AZP),
            // slant zenithal perspective
            "SZP" => Ok(CTYPE_PROJ::SZP),
            // Gnomonic
            "TAN" => Ok(CTYPE_PROJ::TAN),
            // Orthographic
            "SIN" => Ok(CTYPE_PROJ::SIN),

            /* Cylindrical projections */ 
            // Mollweide’s projection
            "MOL" => Ok(CTYPE_PROJ::MOL),
            // Hammer-Aitoff
            "AIT" => Ok(CTYPE_PROJ::AIT),
            _ => Err("CTYPE last 3-character not recognized")
        }?;

        let ctype_coo_ref = match (&ctype1[0..4], &ctype2[0..4]) {
            ("RA--", "DEC-") => Ok(CTYPE_COO_REF::RA_DEC),
            ("GLON", "GLAT") => Ok(CTYPE_COO_REF::G_LON_LAT),
            ("ELON", "ELAT") => Ok(CTYPE_COO_REF::E_LON_LAT),
            ("RLON", "RLAT") => Ok(CTYPE_COO_REF::R_LON_LAT),
            ("HLON", "HLAT") => Ok(CTYPE_COO_REF::H_LON_LAT),
            _ => Err("CTYPE first 4-character not recognized")
        }?;

        let default_lonpole = if ctype_proj.is_zenithal() {
            // For zenithal projection, as theta_0, the latitude of the fiducial point
            // corresponds to the native point, then lonpole = 0, unless delta_0 equals 90.0
            if crval2 == 90.0 {
                0.0
            } else {
                180.0
            }
        } else if ctype_proj.is_cylindrical() {
            if crval2 >= 0.0 {
                0.0
            } else {
                180.0
            }
        } else {
            return Err("Other from cylindrical and zenithal projections not yet implemented!");
        };

        let lonpole = get_card_float_value(hdu, "LONPOLE", default_lonpole)?;
        let latpole = get_card_float_value(hdu, "LATPOLE", 0.0)?;

        let radesys = match get_card_str_value(hdu, "RADESYS")? {
            // International Celestial Reference System
            "ICRS" => Ok(RADESYS::ICRS),
            // Mean place, new (IAU 1984) system
            "FK5" => Ok(RADESYS::FK5),
            // Mean place, old (Nessell-Newcomb) system
            "FK4" => Ok(RADESYS::FK4),
            // Mean place, old system without e-terms
            "FK4_NO_E" => Ok(RADESYS::FK4_NO_E),
            // Geocentric apparent place, IAU 1984 system
            "GAPPT" => Ok(RADESYS::GAPPT),
            _ => Err("Reference system not recognized")
        }?;

        // Native to celestial coordinates matrix
        let (alpha_p, delta_p, phi_p) = if ctype_proj.is_zenithal() {
            Ok((crval1.to_radians(), crval2.to_radians(), lonpole.to_radians()))
        } else {
            Err("cylindrical alpha_p, delta_p, phi_p not yet implemented. See p1080 of the reference paper")
        }?;

        // (Y-X'-Y'') Euler angles matrix
        let r = create_yxy_euler_rotation_matrix(-alpha_p, delta_p, -phi_p + crate::math::PI);
        let r_inv = r.transpose();

        Ok(WCS2 {
            crpix: Vector2::new(crpix1, crpix2),
            crval: Vector2::new(crval1.to_radians(), crval2.to_radians()),
            pixel_interm_params,
            r,
            r_inv,
            lonpole: lonpole.to_radians(),
            latpole: latpole.to_radians(),
            ctype_coo_ref,
            ctype_proj,
            radesys
        })
    }

    /* Pixel <=> projection plane coordinates transformation */
    fn pixel_to_interm_world_coordinates(&self, p: &Vector2<f64>) -> Result<Vector2<f64>, &'static str> {
        let p_off = p - self.crpix;

        match &self.pixel_interm_params {
            Pixel2IntermParams::CDELT_PC { pc, cdelt, .. } => {
                let p_rot = pc * p_off;

                Ok(Vector2::new(
                    -p_rot.x * cdelt.x,
                    p_rot.y * cdelt.y,
                ))
            },
            Pixel2IntermParams::CD { cd, .. } => {
                let p_rot = cd * p_off;

                Ok(Vector2::new(
                    -p_rot.x,
                    p_rot.y,
                ))
            },
            _ => Err("CROTA not implemented")
        }
    }

    fn interm_world_to_pixel_coordinates(&self, x: &Vector2<f64>) -> Result<Vector2<f64>, &'static str> {
        let p_off = (match &self.pixel_interm_params {
            Pixel2IntermParams::CDELT_PC { pc_inv, cdelt, .. } => {
                let p_rot = Vector2::new(
                    -x.x / cdelt.x,
                    x.y / cdelt.y,
                );
        
                Ok(pc_inv * p_rot)
            },
            Pixel2IntermParams::CD { cd_inv, .. } => {
                let p_rot = Vector2::new(
                    -x.x ,
                    x.y,
                );
        
                Ok(cd_inv * p_rot)
            },
            _ => Err("CROTA not implemented")
        })?;

        Ok(p_off + self.crpix)
    }

    /* Projection plane <=> native spherical coordinates transformation */
    fn interm_world_to_native_spherical_coordinates(&self, x: &Vector2<f64>) -> Result<Option<Vector4<f64>>, &'static str> {
        match self.ctype_proj {
            // Zenithal/azimuthal perspective
            CTYPE_PROJ::AZP => Err("AZP not implemented yet!"),
            // Slant zenithal perspective
            CTYPE_PROJ::SZP => Err("SZP not implemented yet!"),
            // Gnomonic
            CTYPE_PROJ::TAN => {
                Ok(Gnomonic.clip_to_world_space(x))
            },
            // Orthographic
            CTYPE_PROJ::SIN => {
                Ok(Orthographic.clip_to_world_space(x))
            },
            // Mollweide’s projection
            CTYPE_PROJ::MOL => Err("MOL not implemented yet!"),
            // Hammer-Aitoff
            CTYPE_PROJ::AIT => Err("AIT not implemented yet!"),
        }
    }

    fn native_spherical_to_interm_world_coordinates(&self, xyz: &Vector4<f64>) -> Result<Option<Vector2<f64>>, &'static str> {
        match self.ctype_proj {
            // Zenithal/azimuthal perspective
            CTYPE_PROJ::AZP => Err("AZP not implemented yet!"),
            // Slant zenithal perspective
            CTYPE_PROJ::SZP => Err("SZP not implemented yet!"),
            // Gnomonic
            CTYPE_PROJ::TAN => {
                Ok(Gnomonic.world_to_clip_space(xyz))
            },
            // Orthographic
            CTYPE_PROJ::SIN => {
                Ok(Orthographic.world_to_clip_space(xyz))
            },
            // Mollweide’s projection
            CTYPE_PROJ::MOL => Err("MOL not implemented yet!"),
            // Hammer-Aitoff
            CTYPE_PROJ::AIT => Err("AIT not implemented yet!"),
        }
    }

    /* Native <=> celestial spherical coordinates transformation */
    fn native_to_celestial_spherical_coordinates(&self, xyz: &Vector4<f64>) -> Vector4<f64> {
        self.r_inv * xyz
    }

    fn celestial_to_native_spherical_coordinates(&self, xyz: &Vector4<f64>) -> Vector4<f64> {
        self.r * xyz
    }

    pub fn proj(&self, xyz: &Vector4<f64>) -> Result<Option<Vector2<f64>>, &'static str> {
        let x = self.native_spherical_to_interm_world_coordinates(
            &self.celestial_to_native_spherical_coordinates(xyz)
        )?;

        if let Some(x) = x {
            Ok(Some(self.interm_world_to_pixel_coordinates(&x)?))
        } else {
            Ok(None)
        }
    }

    pub fn deproj(&self, p: &Vector2<f64>) -> Result<Option<Vector4<f64>>, &'static str> {
        let xyz = self.interm_world_to_native_spherical_coordinates(
            &self.pixel_to_interm_world_coordinates(p)?
        )?
        .map(|xyz| self.native_to_celestial_spherical_coordinates(&xyz));

        Ok(xyz)
    }
}

#[derive(Debug)]
enum CTYPE_COO_REF {
    RA_DEC,
    G_LON_LAT,
    E_LON_LAT,
    R_LON_LAT,
    H_LON_LAT,
}

#[derive(Debug)]
enum CTYPE_PROJ {
    // zenithal/azimuthal perspective
    AZP,
    // slant zenithal perspective
    SZP,
    // Orthographic
    SIN,
    // Gnomonic
    TAN,
    // Mollweide’s projection
    MOL,
    // Hammer-Aitoff
    AIT,
}

impl CTYPE_PROJ {
    fn is_zenithal(&self) -> bool {
        match self {
            CTYPE_PROJ::AZP | CTYPE_PROJ::SZP | CTYPE_PROJ::SIN | CTYPE_PROJ::TAN => true,
            _ => false
        }
    }

    fn is_cylindrical(&self) -> bool {
        match self {
            CTYPE_PROJ::MOL | CTYPE_PROJ::AIT => true,
            _ => false
        }
    }
}

#[derive(Debug)]
enum RADESYS {
    // International Celestial Reference System
    ICRS,
    // Mean place, new (IAU 1984) system
    FK5,
    // Mean place, old (Nessell-Newcomb) system
    FK4,
    // Mean place, old system without e-terms
    FK4_NO_E,
    // Geocentric apparent place, IAU 1984 system
    GAPPT,
}

 mod tests {
    use std::fs::File;
    use std::io::Read;

    use cgmath::Vector2;
    use crate::wcs::WCS2;
    use al_core::image::fits::FitsBorrowed;

    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            if ($x - $y).abs() > $d { assert!(false); }
        }
    }

    #[test]
    fn proj_deproj_round_trip() {
        let mut f = File::open("../../examples/cutout-CDS_P_HST_PHAT_F475W.fits").unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();

        let fits = FitsBorrowed::new(&buf).unwrap();

        let wcs = WCS2::new(&fits).unwrap();
        let p = Vector2::new(0.0, 0.0);
        let xyz = wcs.deproj(&p).unwrap().unwrap();
        let p_prim = wcs.proj(&xyz).unwrap().unwrap();

        assert_delta!(p.x, p_prim.x, 1e-6);
        assert_delta!(p.y, p_prim.y, 1e-6);
    }

    use crate::math::angle::Angle;
    #[test]
    fn deproj() {
        let mut f = File::open("../../examples/cutout-CDS_P_HST_PHAT_F475W.fits").unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();

        let fits = FitsBorrowed::new(&buf).unwrap();

        let wcs = WCS2::new(&fits).unwrap();
        let p = Vector2::new(1500.0, 1500.0);
        let xyz = wcs.deproj(&p).unwrap().unwrap();

        let (Angle(lon), Angle(lat)) = crate::math::lonlat::xyzw_to_radec(&xyz);

        assert_delta!(lon, wcs.crval.x, 1e-6);
        assert_delta!(lat, wcs.crval.y, 1e-6);
    }

    use crate::LonLatT;
    use crate::ArcDeg;
    #[test]
    fn proj() {
        let mut f = File::open("../../examples/cutout-CDS_P_HST_PHAT_F475W.fits").unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();

        let fits = FitsBorrowed::new(&buf).unwrap();

        let wcs = WCS2::new(&fits).unwrap();
        let xyz = LonLatT::new(Angle(wcs.crval.x), Angle(wcs.crval.y)).vector();
        let p = dbg!(wcs.proj(&xyz).unwrap().unwrap());

        assert_delta!(p.x, 1500.0, 1e-6);
        assert_delta!(p.y, 1500.0, 1e-6);
    }
}
