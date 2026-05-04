use crossbeam::channel::Receiver;
use ndk::native_window::NativeWindow;
use log::{debug, error, info, warn};

use vello_cpu::{RenderContext, RenderMode};

use crate::{VelloApp, internal::msgs::InternalMsg};

pub(crate) struct RenderThread {
    app: Box<dyn VelloApp>,
    receiver: Receiver<InternalMsg>,
    current_surface: Option<NativeWindow>,
    last_frame_time: i64,
    tight_buffer: Vec<u8>,
    tight_buffer_size: (u32, u32),
}

impl RenderThread {
    pub fn new(app: Box<dyn VelloApp>, receiver: Receiver<InternalMsg>) -> Self {
        Self {
            app,
            receiver,
            current_surface: None,
            last_frame_time: 0,
            tight_buffer: Vec::new(),
            tight_buffer_size: (0, 0),
        }
    }

    pub fn run_loop(mut self) {
        info!("RenderThread starting");
        self.app.on_init();

        while let Ok(msg) = self.receiver.recv() {
            match msg {
                InternalMsg::SurfaceCreated(window) => {
                    info!("Surface created");
                    self.current_surface = Some(window);
                }
                InternalMsg::SurfaceChanged { width, height } => {
                    info!("Surface changed: {}x{}", width, height);
                    self.app.on_resize(width, height);
                }
                InternalMsg::SurfaceDestroyed => {
                    warn!("Surface destroyed");
                    self.current_surface = None;
                }
                InternalMsg::Touch { action, x, y } => {
                    self.app.on_touch(action, x, y);
                }
                InternalMsg::DoFrame(timestamp) => {
                    let dt = if self.last_frame_time > 0 {
                        (timestamp - self.last_frame_time) as f64 / 1_000_000.0
                    } else {
                        16.666
                    };
                    self.last_frame_time = timestamp;

                    if self.current_surface.is_some() {
                        self.render_frame(dt);
                    }
                }
            }
        }

        warn!("RenderThread exit");
    }

    fn render_frame(&mut self, dt: f64) {
        if let Some(surface) = self.current_surface.as_mut() {
            match surface.lock(None) {
                Ok(mut buffer) => {
                    let width = buffer.width() as u32;
                    let height = buffer.height() as u32;
                    let stride = buffer.stride() as u32;

                    // Resize our internal buffer if needed
                    if self.tight_buffer_size != (width, height) {
                        let new_size = (width * height * 4) as usize;
                        debug!("Resizing buffer to: {}x{} ({} bytes)", width, height, new_size);
                        self.tight_buffer.resize(new_size, 0);
                        self.tight_buffer_size = (width, height);
                    }

                    // Render into our internal buffer
                    let mut ctx = RenderContext::new(width as u16, height as u16);
                    self.app.on_draw(&mut ctx, dt);
                    ctx.render_to_buffer(
                        &mut self.tight_buffer,
                        width as u16,
                        height as u16,
                        RenderMode::OptimizeSpeed,
                    );

                    // 关键：安全地复制数据，只渲染一半高度来防止崩溃
                    let copy_height = std::cmp::min(height, 1200); // 先只渲染一半高度
                    let row_bytes = (width * 4) as usize;
                    let dst_row_bytes = (stride * 4) as usize;
                    let buffer_ptr = buffer.bits() as *mut u8;

                    unsafe {
                        for y in 0..copy_height as usize {
                            let src = self.tight_buffer.as_ptr().add(y * row_bytes);
                            let dst = buffer_ptr.add(y * dst_row_bytes);
                            std::ptr::copy_nonoverlapping(src, dst, row_bytes);
                        }
                    }

                    debug!("Rendered: {}x{}", width, copy_height);
                }
                Err(e) => {
                    error!("Failed to lock buffer: {:?}", e);
                }
            }
        }
    }
}
