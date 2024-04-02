use super::Renderer;
use al_core::log::console_log;
use web_sys::CanvasRenderingContext2d;

pub struct TextRenderManager {
    // The text canvas
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    color: JsValue,
    font_size: u32,
}

use cgmath::{Rad, Vector2};
use wasm_bindgen::JsValue;

use crate::camera::CameraViewPort;
use al_api::color::{ColorRGB, ColorRGBA};
use web_sys::HtmlCanvasElement;

use crate::Abort;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

impl TextRenderManager {
    /// Init the buffers, VAO and shader
    pub fn new(aladin_div: &HtmlElement) -> Result<Self, JsValue> {
        let canvas = aladin_div
            // Inside it, retrieve the canvas
            .get_elements_by_class_name("aladin-catalogCanvas")
            .get_with_index(0)
            .unwrap_abort()
            .dyn_into::<web_sys::HtmlCanvasElement>()?;
        let ctx = canvas
            .get_context("2d")
            .unwrap_abort()
            .unwrap_abort()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap_abort();

        let color = JsValue::from_str("#00ff00");
        let font_size = 15;
        Ok(Self {
            font_size,
            color,
            canvas,
            ctx,
        })
    }

    pub fn set_color(&mut self, color: &ColorRGBA) {
        let hex = al_api::color::Color::rgbaToHex(
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
            (color.a * 255.0) as u8,
        );
        self.color = JsValue::from_str(&hex);
    }

    pub fn set_font_size(&mut self, size: u32) {
        self.font_size = size;
    }

    pub fn add_label<A: Into<Rad<f32>>>(
        &mut self,
        text: &str,
        screen_pos: &Vector2<f32>,
        angle: A,
    ) -> Result<(), JsValue> {
        self.ctx.save();

        self.ctx
            .translate(screen_pos.x as f64, screen_pos.y as f64)?;

        let rot: Rad<f32> = angle.into();
        self.ctx.rotate(rot.0 as f64)?;

        self.ctx.set_text_align("center");
        self.ctx.fill_text(text, 0.0, 0.0)?;

        self.ctx.restore();

        Ok(())
    }

    pub fn draw(
        &mut self,
        _camera: &CameraViewPort,
        _color: &ColorRGBA,
        _scale: f32,
    ) -> Result<(), JsValue> {
        Ok(())
    }
}

impl Renderer for TextRenderManager {
    fn begin(&mut self) {
        self.ctx = self
            .canvas
            .get_context("2d")
            .unwrap_abort()
            .unwrap_abort()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap_abort();

        //self.clear_text_canvas();
        // Clear the Aladin Lite 2d canvas
        // This canvas is where catalogs, grid labels, Hpx grid are drawn
        /*self.ctx.clear_rect(
            0_f64,
            0_f64,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );*/

        // reset the font and color
        self.ctx
            .set_font(&format!("{}px verdana, sans-serif", self.font_size));
        self.ctx.set_fill_style(&self.color);
    }

    fn end(&mut self) {}
}
