use crossbeam::channel::Receiver;
use ndk::native_window::NativeWindow;
use log::{debug, error, info, trace, warn};

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
        debug!("[RenderThread::new] Initializing RenderThread struct");
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
        info!("[RenderThread::run_loop] Starting render loop");
        
        self.app.on_init();
        
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                InternalMsg::SurfaceCreated(window) => {
                    info!("[RenderThread::SurfaceCreated] NativeWindow attached");
                    self.current_surface = Some(window);
                }
                InternalMsg::SurfaceChanged { width, height } => {
                    info!("[RenderThread::SurfaceChanged] Dimensions: {}x{}", width, height);
                    self.app.on_resize(width, height);
                }
                InternalMsg::SurfaceDestroyed => {
                    warn!("[RenderThread::SurfaceDestroyed] Surface released");
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
        
        warn!("[RenderThread::run_loop] Exiting render loop");
    }

    fn render_frame(&mut self, dt: f64) {
        if let Some(surface) = self.current_surface.as_mut() {
            match surface.lock(None) {
                Ok(mut buffer) => {
                    let width = buffer.width() as u32;
                    let height = buffer.height() as u32;
                    let stride = buffer.stride() as u32;
                    
                    trace!("[RenderThread::render_frame] Buffer: {}x{} (stride: {})", 
                          width, height, stride);
                    
                    if self.tight_buffer_size != (width, height) {
                        let new_size = (width * height * 4) as usize;
                        debug!("[RenderThread::render_frame] Resizing tight buffer: {} bytes", 
                              new_size);
                        self.tight_buffer.resize(new_size, 0);
                        self.tight_buffer_size = (width, height);
                    }
                    
                    let mut ctx = RenderContext::new(width as u16, height as u16);
                    self.app.on_draw(&mut ctx, dt);
                    ctx.render_to_buffer(&mut self.tight_buffer, 
                                        width as u16, 
                                        height as u16, 
                                        RenderMode::OptimizeSpeed);
                    
                    let row_bytes = (width * 4) as usize;
                    let dst_stride = (stride * 4) as usize;
                    let buffer_ptr = buffer.bits();
                    
                    if !buffer_ptr.is_null() {
                        for y in 0..height as usize {
                            let src_idx = y * row_bytes;
                            let dst_idx = y * dst_stride;
                            
                            unsafe {
                                let src_ptr = self.tight_buffer.as_ptr().add(src_idx);
                                let dst_ptr = (buffer_ptr as *mut u8).add(dst_idx);
                                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, row_bytes);
                            }
                        }
                        trace!("[RenderThread::render_frame] Completed {} row copies", height);
                    } else {
                        error!("[RenderThread::render_frame] Buffer pointer is NULL!");
                    }
                }
                Err(e) => {
                    error!("[RenderThread::render_frame] Failed to lock buffer: {:?}", e);
                }
            }
        }
    }
}
