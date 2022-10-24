use fitsrs::PrimaryHeader;
use cgmath::{Matrix2, Matrix4, Matrix, Vector2, Vector4};

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
    // Coordinate increment at reference point [deg] (s_i)
    cdelt: Vector2<f64>,
    // Linear transformation matrix (m_ij)
    pc: Matrix2<f64>,
    // Inv of the linear transformation matrix (m_ij)
    pc_inv: Matrix2<f64>,
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

use fitsrs::{FITSCardValue, FITSCard};
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

impl WCS2 {
    pub fn new<'a>(fits: &FitsBorrowed<'a>) -> Result<Self, &'static str> {
        let hdu = fits.get_header();
        let crpix1 = get_card_float_value(hdu, "CRPIX1", 0.0)?;
        let crpix2 = get_card_float_value(hdu, "CRPIX2", 0.0)?;

        let cdelt1 = get_card_float_value(hdu, "CDELT1", 1.0)?;
        let cdelt2 = get_card_float_value(hdu, "CDELT2", 1.0)?;

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

        // Linear transformation matrix
        let pc11 = get_card_float_value(hdu, "PC1_1", 1.0)?;
        let pc12 = get_card_float_value(hdu, "PC1_2", 0.0)?;
        let pc21 = get_card_float_value(hdu, "PC2_1", 0.0)?;
        let pc22 = get_card_float_value(hdu, "PC2_2", 1.0)?;

        let pc = Matrix2::new(pc11, pc21, pc12, pc22);
        let pc_inv = pc.transpose();
        // Native to celestial coordinates matrix
        let (alpha_p, delta_p, phi_p) = if ctype_proj.is_zenithal() {
            let alpha_p = crval1.to_radians();
            let delta_p = crval2.to_radians();

            Ok((crval1, crval2, lonpole.to_radians()))
        } else {
            Err("cylindrical alpha_p, delta_p, phi_p not yet implemented. See p1080 of the reference paper")
        }?;

        let (s_ap, c_ap) = alpha_p.sin_cos();
        let (s_dp, c_dp) = delta_p.sin_cos();
        let (s_pp, c_pp) = phi_p.sin_cos();

        let r11 = -s_ap * s_pp - c_ap * c_pp * s_dp;
        let r12 = c_ap * s_pp - s_ap * c_pp * s_dp;
        let r13 = c_pp * c_dp;
        
        let r21 = s_ap * c_pp - c_ap * s_pp * s_dp;
        let r22 = -c_ap * c_pp - s_ap * s_pp * s_dp;
        let r23 = s_pp * c_dp;

        let r31 = c_ap * c_dp;
        let r32 = s_ap * c_dp;
        let r33 = s_dp;

        let r = Matrix4::new(
            r11, r21, r31, 0.0,
            r12, r22, r32, 0.0,
            r13, r23, r33, 0.0,
            0.0, 0.0, 0.0, 1.0
        );
        use cgmath::SquareMatrix;
        //let r = Matrix4::identity();
        let r_inv = r.transpose();
        Ok(WCS2 {
            crpix: Vector2::new(crpix1, crpix2),
            crval: Vector2::new(crval1, crval2),
            cdelt: Vector2::new(cdelt1, cdelt2),
            pc,
            pc_inv,
            r,
            r_inv,
            lonpole,
            latpole,
            ctype_coo_ref,
            ctype_proj,
            radesys
        })
    }

    /* Pixel <=> projection plane coordinates transformation */
    fn pixel_to_interm_world_coordinates(&self, p: &Vector2<f64>) -> Vector2<f64> {
        let p_off = p - self.crpix;
        let p_rot = self.pc * p_off;

        Vector2::new(
            (p_rot.x * self.cdelt.x).to_radians(),
            (p_rot.y * self.cdelt.y).to_radians(),
        )
    }

    fn interm_world_to_pixel_coordinates(&self, x: &Vector2<f64>) -> Vector2<f64> {
        let p_rot = Vector2::new(
            x.x.to_degrees() / self.cdelt.x,
            x.y.to_degrees() / self.cdelt.y,
        );

        let p_off = self.pc_inv * p_rot;

        p_off + self.crpix
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
        let p = self.native_spherical_to_interm_world_coordinates(
            &self.celestial_to_native_spherical_coordinates(xyz)
        )?
        .map(|x| self.interm_world_to_pixel_coordinates(&x));

        Ok(p)
    }

    pub fn deproj(&self, p: &Vector2<f64>) -> Result<Option<Vector4<f64>>, &'static str> {
        let xyz = self.interm_world_to_native_spherical_coordinates(
            &self.pixel_to_interm_world_coordinates(p)
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
            if !($x - $y < $d || $y - $x < $d) { panic!(); }
        }
    }

    #[test]
    fn identity() {
        let mut f = File::open("../../examples/cutout-CDS_P_HST_PHAT_F475W.fits").unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();

        let fits = FitsBorrowed::new(&buf).unwrap();

        let wcs = WCS2::new(&fits).unwrap();
        let p = Vector2::new(1500.0, 1500.0);
        let xyz = wcs.deproj(&p).unwrap().unwrap();
        let p_prim = wcs.proj(&xyz).unwrap().unwrap();

        assert_delta!(p.x, p_prim.x, 1e-6);
        assert_delta!(p.y, p_prim.y, 1e-6);
    }

    #[test]
    fn deproj() {
        let mut f = File::open("../../examples/cutout-CDS_P_HST_PHAT_F475W.fits").unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();

        let fits = FitsBorrowed::new(&buf).unwrap();

        let wcs = WCS2::new(&fits).unwrap();
        let p = Vector2::new(1500.0, 1500.0);
        let lonlat = wcs.deproj(&p).unwrap().unwrap().lonlat();

        use crate::math::lonlat::LonLat;
        println!("{:?} {:?}", wcs, lonlat);
    }
 }