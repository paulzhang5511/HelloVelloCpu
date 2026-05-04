use crossbeam::channel::Receiver;
use ndk::native_window::NativeWindow;
use vello::Scene;
use crate::{VelloApp, internal::msgs::InternalMsg};

pub(crate) struct RenderThread {
    app: Box<dyn VelloApp>,
    receiver: Receiver<InternalMsg>,
    current_surface: Option<NativeWindow>,
    scene: Scene,
    last_frame_time: i64,
}

impl RenderThread {
    pub fn new(app: Box<dyn VelloApp>, receiver: Receiver<InternalMsg>) -> Self {
        Self {
            app,
            receiver,
            current_surface: None,
            scene: Scene::new(),
            last_frame_time: 0,
        }
    }

    pub fn run_loop(mut self) {
        self.app.on_init();

        while let Ok(msg) = self.receiver.recv() {
            match msg {
                InternalMsg::SurfaceCreated(window) => self.current_surface = Some(window),
                InternalMsg::SurfaceChanged { width, height } => self.app.on_resize(width, height),
                InternalMsg::SurfaceDestroyed => self.current_surface = None,
                InternalMsg::Touch { action, x, y } => self.app.on_touch(action, x, y),
                InternalMsg::DoFrame(timestamp) => {
                    let dt = if self.last_frame_time > 0 {
                        (timestamp - self.last_frame_time) as f64 / 1_000_000.0
                    } else {
                        16.666
                    };
                    self.last_frame_time = timestamp;
                    
                    self.render_frame(dt);
                }
            }
        }
    }

    fn render_frame(&mut self, dt: f64) {
        if let Some(surface) = self.current_surface.as_mut() {
            // 修复 E0596: 添加 mut 关键字，声明 buffer 是可变的
            if let Ok(mut buffer) = surface.lock(None) {
                let width = buffer.width() as u32;
                let height = buffer.height() as u32;
                let stride = buffer.stride() as u32;

                self.scene.reset();
                self.app.on_draw(&mut self.scene, dt);

                // 获取安全的可变像素切片
                let pixels = unsafe {
                    std::slice::from_raw_parts_mut(
                        buffer.bits() as *mut u8,
                        (stride * height * 4) as usize,
                    )
                };

                // ----------------------------------------------------
                // 占位逻辑：为了消除未使用变量的警告，临时手动覆写像素。
                // 这将会在屏幕上画满红色，代表您的底层 JNI 和内存锁已经完全跑通！
                // 查阅到 vello_cpu 最新版的入口后，将这段 for 循环删掉并替换即可。
                // ----------------------------------------------------
                for y in 0..height {
                    for x in 0..width {
                        let offset = ((y * stride + x) * 4) as usize;
                        pixels[offset] = 255;     // R
                        pixels[offset + 1] = 0;   // G
                        pixels[offset + 2] = 0;   // B
                        pixels[offset + 3] = 255; // A
                    }
                }
                
                // TODO: 用 vello_cpu 光栅化 self.scene。
                // 例如: vello_cpu::Renderer::new().render(&self.scene, pixels, width, height, stride);
            }
        }
    }
}