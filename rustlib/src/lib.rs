// src/lib.rs
pub use kurbo::{Affine, Rect, RoundedRect};
pub use peniko::Color;
pub use vello_cpu::{RenderContext, color::PremulRgba8};

pub trait VelloApp: Send + 'static {
    fn on_init(&mut self) {}
    fn on_resize(&mut self, _width: u32, _height: u32) {}
    fn on_touch(&mut self, _action: i32, _x: f32, _y: f32) {}
    fn on_draw(&mut self, context: &mut RenderContext, delta_time_ms: f64);
}

pub mod macros;

#[doc(hidden)]
#[cfg(target_os = "android")]
pub mod internal;

mod interactive_box;
