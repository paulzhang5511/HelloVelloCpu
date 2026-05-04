use log::{info, debug, trace};
use rustlib::{rustlib_main, VelloApp, Scene, Color, Rect, Affine};

/// 交互式方块的业务状态结构体
#[derive(Default)]
struct InteractiveBox {
    x: f32,
    y: f32,
    screen_width: f32,
    screen_height: f32,
}

impl VelloApp for InteractiveBox {
    fn on_init(&mut self) {
        // 1. 初始化 Android Logcat 日志系统
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Trace)
                .with_tag("RustlibExample")
        );

        info!("Application initialization started. Version: 1.0.0-example");

        self.x = 500.0;
        self.y = 500.0;
        
        debug!("Initial coordinates set to x: {}, y: {}", self.x, self.y);
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        info!("Surface resized. New dimensions: {}x{}", width, height);
        
        self.screen_width = width as f32;
        self.screen_height = height as f32;
        
        self.x = self.screen_width / 2.0;
        self.y = self.screen_height / 2.0;

        trace!("Box position recalibrated to screen center: ({}, {})", self.x, self.y);
    }

    fn on_touch(&mut self, action: i32, x: f32, y: f32) {
        trace!("Touch event received. Action: {}, Coordinates: ({}, {})", action, x, y);
        self.x = x;
        self.y = y;
    }

    fn on_draw(&mut self, scene: &mut Scene, dt: f64) {
        trace!("Rendering frame. Delta time: {:.2}ms", dt);

        let bg_rect = Rect::new(0.0, 0.0, self.screen_width as f64, self.screen_height as f64);
        
        // 修复 E0599: 使用 from_rgb8 代替被废弃的 rgb8
        scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgb8(30, 30, 30),
            None,
            &bg_rect,
        );

        let box_size = 300.0;
        let half_size = box_size / 2.0;
        
        let target_rect = Rect::new(
            (self.x - half_size) as f64,
            (self.y - half_size) as f64,
            (self.x + half_size) as f64,
            (self.y + half_size) as f64,
        ).to_rounded_rect(40.0);

        // 修复 E0599: 使用 from_rgb8
        scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgb8(0, 255, 128),
            None,
            &target_rect,
        );
    }
}

rustlib_main!(InteractiveBox, "com/example/demo/VelloSurfaceView");