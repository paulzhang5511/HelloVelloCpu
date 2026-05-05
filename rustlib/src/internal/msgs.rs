use ndk::native_window::NativeWindow;

// 内部消息枚举，用于 Java 层与 Rust 渲染线程之间的通信
#[derive(Debug)]
pub(crate) enum InternalMsg {
    SurfaceCreated(NativeWindow),
    SurfaceChanged { width: u32, height: u32 },
    SurfaceDestroyed,
    DoFrame(i64),
    Touch { action: i32, x: f32, y: f32 },
}
