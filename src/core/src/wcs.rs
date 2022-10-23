use fitsrs::PrimaryHeader;
use cgmath::{Matrix2, Vector2, Vector4};

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

        let pc11 = get_card_float_value(hdu, "PC1_1", 1.0)?;
        let pc12 = get_card_float_value(hdu, "PC1_2", 0.0)?;
        let pc21 = get_card_float_value(hdu, "PC2_1", 0.0)?;
        let pc22 = get_card_float_value(hdu, "PC2_2", 1.0)?;

        let cdelt1 = get_card_float_value(hdu, "CDELT1", 1.0)?;
        let cdelt2 = get_card_float_value(hdu, "CDELT2", 1.0)?;

        let crval1 = get_card_float_value(hdu, "CRVAL1", 0.0)?;
        let crval2 = get_card_float_value(hdu, "CRVAL2", 0.0)?;

        let lonpole = get_card_float_value(hdu, "LONPOLE", 0.0)?;
        let latpole = get_card_float_value(hdu, "LATPOLE", 0.0)?;

        let ctype1 = get_card_str_value(hdu, "CTYPE1")?;
        let ctype2 = get_card_str_value(hdu, "CTYPE2")?;

        let ctype_coo_ref = match (&ctype1[0..4], &ctype2[0..4]) {
            ("RA--", "DEC-") => Ok(CTYPE_COO_REF::RA_DEC),
            ("GLON", "GLAT") => Ok(CTYPE_COO_REF::G_LON_LAT),
            ("ELON", "ELAT") => Ok(CTYPE_COO_REF::E_LON_LAT),
            ("RLON", "RLAT") => Ok(CTYPE_COO_REF::R_LON_LAT),
            ("HLON", "HLAT") => Ok(CTYPE_COO_REF::H_LON_LAT),
            _ => Err("CTYPE first 4-character not recognized")
        }?;

        let ctype_proj = match &ctype1[5..8] {
            // zenithal/azimuthal perspective
            "AZP" => Ok(CTYPE_PROJ::AZP),
            // slant zenithal perspective
            "SZP" => Ok(CTYPE_PROJ::SZP),
            // Gnomonic
            "TAN" => Ok(CTYPE_PROJ::TAN),
            // Mollweide’s projection
            "MOL" => Ok(CTYPE_PROJ::MOL),
            // Hammer-Aitoff
            "AIT" => Ok(CTYPE_PROJ::AIT),
            _ => Err("CTYPE last 3-character not recognized")
        }?;

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

        Ok(WCS2 {
            crpix: Vector2::new(crpix1, crpix2),
            crval: Vector2::new(crval1, crval2),
            cdelt: Vector2::new(cdelt1, cdelt2),
            pc: Matrix2::new(pc11, pc21, pc12, pc22),
            lonpole,
            latpole,
            ctype_coo_ref,
            ctype_proj,
            radesys
        })
    }

    pub fn pixel_to_intermediate_world_coordinates(&self, p: &Vector2<f64>) -> Vector2<f64> {
        let p_off = p - self.crpix;
        let p_rot = self.pc * p_off;

        let p_s = Vector2::new(
            p_rot.x * self.cdelt.x,
            p_rot.y * self.cdelt.y,
        );

        p_s
    }

    pub fn intermediate_world_to_pixel_coordinates(&self, p: &Vector2<f64>) -> Vector2<f64> {
        let p_off = p - self.crpix;
        let p_rot = self.pc * p_off;

        let p_s = Vector2::new(
            p_rot.x * self.cdelt.x,
            p_rot.y * self.cdelt.y,
        );

        p_s
    }

    pub fn interm_world_to_celestial_spherical_coordinates(&self, x: &Vector2<f64>) -> Option<Vector4<f64>> {
        match self.ctype_proj {
            // Zenithal/azimuthal perspective
            CTYPE_PROJ::AZP => unimplemented!(),
            // Slant zenithal perspective
            CTYPE_PROJ::SZP => unimplemented!(),
            // Gnomonic
            CTYPE_PROJ::TAN => {
                Gnomonic.clip_to_world_space(x)
            },
            // Orthographic
            CTYPE_PROJ::SIN => {
                Orthographic.clip_to_world_space(x)
            },
            // Mollweide’s projection
            CTYPE_PROJ::MOL => unimplemented!(),
            // Hammer-Aitoff
            CTYPE_PROJ::AIT => unimplemented!(),
        }
    }

    pub fn pixel_to_celestial_spherical_coordinates(&self, p: &Vector2<f64>) -> Option<Vector4<f64>> {
        let x = self.pixel_to_intermediate_world_coordinates(p);
        self.interm_world_to_celestial_spherical_coordinates(&x)
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