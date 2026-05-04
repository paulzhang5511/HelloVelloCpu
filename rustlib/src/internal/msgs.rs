use ndk::native_window::NativeWindow;

pub(crate) enum InternalMsg {
    SurfaceCreated(NativeWindow),
    SurfaceChanged { width: u32, height: u32 },
    SurfaceDestroyed,
    DoFrame(i64),
    Touch { action: i32, x: f32, y: f32 },
}