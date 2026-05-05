pub mod msgs;
pub mod render_thread;

use crossbeam::channel::{Sender, unbounded};
use jni::JNIEnv;
use jni::NativeMethod;
use jni::objects::{JClass, JObject};
use log::warn;
use ndk::hardware_buffer_format::HardwareBufferFormat;
use ndk::native_window::NativeWindow;
use std::ffi::c_void;
use std::sync::OnceLock;
use std::thread;

use crate::VelloApp;
use msgs::InternalMsg;
use render_thread::RenderThread;

pub static RENDER_TX: OnceLock<Sender<InternalMsg>> = OnceLock::new();

pub fn register_natives<T: VelloApp + Default + 'static>(env: &mut JNIEnv, class_path: &str) {
    let class = env.find_class(class_path).expect("找不到指定的 Java 类");

    let methods = [
        NativeMethod {
            name: "nativeInit".into(),
            sig: "()V".into(),
            fn_ptr: native_init::<T> as *mut c_void,
        },
        NativeMethod {
            name: "nativeSurfaceCreated".into(),
            sig: "(Landroid/view/Surface;)V".into(),
            fn_ptr: native_surface_created as *mut c_void,
        },
        NativeMethod {
            name: "nativeSurfaceChanged".into(),
            sig: "(II)V".into(),
            fn_ptr: native_surface_changed as *mut c_void,
        },
        NativeMethod {
            name: "nativeSurfaceDestroyed".into(),
            sig: "()V".into(),
            fn_ptr: native_surface_destroyed as *mut c_void,
        },
        NativeMethod {
            name: "nativeDoFrame".into(),
            sig: "(J)V".into(),
            fn_ptr: native_do_frame as *mut c_void,
        },
        NativeMethod {
            name: "nativeTouchEvent".into(),
            sig: "(IFF)V".into(),
            fn_ptr: native_touch_event as *mut c_void,
        },
    ];

    env.register_native_methods(class, &methods)
        .expect("JNI 动态注册失败");
}

extern "system" fn native_init<T: VelloApp + Default + 'static>(_env: JNIEnv, _class: JClass) {
    RENDER_TX.get_or_init(|| {
        let (tx, rx) = unbounded();
        let app = Box::new(T::default());

        thread::Builder::new()
            .name("VelloRenderThread".into())
            .spawn(move || {
                RenderThread::new(app, rx).run_loop();
            })
            .expect("无法启动渲染线程");

        tx
    });
}

// 修复警告: 移除了多余的 `mut` (从 `mut env` 改为 `env`)
extern "system" fn native_surface_created(env: JNIEnv, _class: JClass, surface: JObject) {
    if let Some(tx) = RENDER_TX.get() {
        let window = unsafe {
            // 注意这里如果是 jni 高版本可能需要借用，如果报错请改为 env.get_native_interface()
            NativeWindow::from_surface(env.get_native_interface(), surface.as_raw())
                .expect("转换 NativeWindow 失败")
        };

        // vello_cpu renders RGBA8888 bytes. Force a 4-byte window format before the
        // render thread locks the buffer so Rust never writes 4 bytes per pixel into
        // a smaller RGB565 surface. Width/height 0 keeps the platform surface size.
        if let Err(e) =
            window.set_buffers_geometry(0, 0, Some(HardwareBufferFormat::R8G8B8A8_UNORM))
        {
            warn!("Failed to request RGBA8888 NativeWindow format: {:?}", e);
        }

        let _ = tx.send(InternalMsg::SurfaceCreated(window));
    }
}

extern "system" fn native_surface_changed(
    _env: JNIEnv,
    _class: JClass,
    width: jni::sys::jint,
    height: jni::sys::jint,
) {
    if width <= 0 || height <= 0 {
        return;
    }

    if let Some(tx) = RENDER_TX.get() {
        let _ = tx.send(InternalMsg::SurfaceChanged {
            width: width as u32,
            height: height as u32,
        });
    }
}

extern "system" fn native_surface_destroyed(_env: JNIEnv, _class: JClass) {
    if let Some(tx) = RENDER_TX.get() {
        let _ = tx.send(InternalMsg::SurfaceDestroyed);
    }
}

extern "system" fn native_do_frame(_env: JNIEnv, _class: JClass, frame_time: jni::sys::jlong) {
    if let Some(tx) = RENDER_TX.get() {
        let _ = tx.send(InternalMsg::DoFrame(frame_time));
    }
}

extern "system" fn native_touch_event(
    _env: JNIEnv,
    _class: JClass,
    action: jni::sys::jint,
    x: jni::sys::jfloat,
    y: jni::sys::jfloat,
) {
    if let Some(tx) = RENDER_TX.get() {
        let _ = tx.send(InternalMsg::Touch { action, x, y });
    }
}
