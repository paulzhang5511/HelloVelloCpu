// 修复 E0432: 去掉了不存在的 SceneBuilder
pub use vello::{self, Scene}; 
pub use vello::kurbo::{Affine, Rect};
pub use vello::peniko::Color;

pub trait VelloApp: Send + 'static {
    fn on_init(&mut self) {}
    // 修复警告: 未使用的参数加上下划线前缀
    fn on_resize(&mut self, _width: u32, _height: u32) {}
    fn on_touch(&mut self, _action: i32, _x: f32, _y: f32) {}
    fn on_draw(&mut self, scene: &mut Scene, delta_time_ms: f64);
}

pub mod macros;

#[doc(hidden)]
#[cfg(target_os = "android")]
pub mod internal;
