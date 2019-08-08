use crate::graphics::GraphicsApi;
use rask_game_engine::math::Mat3;
use webhogg_wasm_shared::error::ClientError;

pub struct Render<T> {
    graphics: T,
    frame_nr: u64,
}

impl<T: GraphicsApi> Render<T> {
    pub fn new(canvas: web_sys::OffscreenCanvas) -> Result<Self, ClientError> {
        T::new(canvas).map(|api| Self {
            graphics: api,
            frame_nr: 0,
        })
    }

    pub fn render(&mut self) -> Result<(), ClientError> {
        self.graphics
            .ok()
            .map_err(|e| ClientError::WebGlError(format!("WebGl2 error: {}", e)))?;
        self.graphics.clear(&[0.8, 0.05, 0.55])?;
        let w = f32::sin((self.frame_nr as f32) * 0.01) * 0.5;
        let h = f32::cos((self.frame_nr as f32) * 0.01) * 0.5;
        self.graphics.draw_rect(&Mat3::scaling(w, h))?;
        self.frame_nr += 1;
        Ok(())
    }
}
