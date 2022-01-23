#[derive(Clone)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Color {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Color {
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }
}

fn to_hex_char(color: f32) -> String {
    let r = (color * 255_f32) as u8;

    let res = format!("{:x}", r);
    if res.len() == 1 {
        format!("0{}", res)
    } else {
        res
    }
}

impl From<&Color> for String {
    fn from(color: &Color) -> Self {
        let red = to_hex_char(color.red);
        let green = to_hex_char(color.green);
        let blue = to_hex_char(color.blue);

        let color = format!("#{}{}{}", red, green, blue);
        color
    }
}
use al_core::{shader::UniformType, WebGlContext};
use web_sys::WebGlUniformLocation;
impl UniformType for Color {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform4f(location, value.red, value.green, value.blue, value.alpha);
    }
}
impl<'a> UniformType for &'a Color {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform4f(location, value.red, value.green, value.blue, value.alpha);
    }
}