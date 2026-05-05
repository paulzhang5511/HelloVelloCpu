use crossbeam::channel::Receiver;
use log::{debug, error, info, warn};
use ndk::native_window::NativeWindow;

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

                    if width == 0 || height == 0 {
                        warn!("Skipping render for empty buffer: {}x{}", width, height);
                        return;
                    }

                    if width > u16::MAX as u32 || height > u16::MAX as u32 {
                        error!(
                            "Buffer dimensions exceed vello_cpu u16 limits: {}x{}",
                            width, height
                        );
                        return;
                    }

                    if stride < width {
                        error!(
                            "Native buffer stride {} is smaller than width {}",
                            stride, width
                        );
                        return;
                    }

                    let Some(row_bytes) = (width as usize).checked_mul(4) else {
                        error!("Row byte count overflow for width {}", width);
                        return;
                    };
                    let Some(dst_row_bytes) = (stride as usize).checked_mul(4) else {
                        error!("Destination row byte count overflow for stride {}", stride);
                        return;
                    };
                    let Some(new_size) = row_bytes.checked_mul(height as usize) else {
                        error!(
                            "Render buffer size overflow for dimensions {}x{}",
                            width, height
                        );
                        return;
                    };

                    // Resize our internal buffer if needed
                    if self.tight_buffer_size != (width, height) {
                        debug!(
                            "Resizing buffer to: {}x{} ({} bytes)",
                            width, height, new_size
                        );
                        self.tight_buffer.resize(new_size, 0);
                        self.tight_buffer_size = (width, height);
                    }

                    // Render into our internal buffer
                    let render_width = width as u16;
                    let render_height = height as u16;
                    let mut ctx = RenderContext::new(render_width, render_height);
                    self.app.on_draw(&mut ctx, dt);
                    ctx.render_to_buffer(
                        &mut self.tight_buffer,
                        render_width,
                        render_height,
                        RenderMode::OptimizeSpeed,
                    );

                    // 关键：逐行复制完整可见高度，并显式校验 stride/尺寸以防止访问越界。
                    let buffer_ptr = buffer.bits() as *mut u8;

                    if buffer_ptr.is_null() {
                        error!("Buffer pointer is null!");
                        return;
                    }

                    unsafe {
                        for y in 0..height as usize {
                            let src = self.tight_buffer.as_ptr().add(y * row_bytes);
                            let dst = buffer_ptr.add(y * dst_row_bytes);
                            std::ptr::copy_nonoverlapping(src, dst, row_bytes);
                        }
                    }

                    debug!("Rendered: {}x{}", width, height);
                }
                Err(e) => {
                    error!("Failed to lock buffer: {:?}", e);
                }
            }
        }
    }
}
