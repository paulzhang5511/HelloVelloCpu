use log::{debug, info, trace};
// 引入 Color 代替 PremulRgba8
use crate::{Color, Rect, RenderContext, VelloApp, rustlib_main};

// 可交互的方块应用结构体
#[derive(Default)]
struct InteractiveBox {
    // 方块的 X 坐标
    x: f32,
    // 方块的 Y 坐标
    y: f32,
    // 屏幕宽度
    screen_width: f32,
    // 屏幕高度
    screen_height: f32,
}

impl VelloApp for InteractiveBox {
    // 应用初始化回调
    fn on_init(&mut self) {
        // 初始化 Android 日志记录器；仅 Android 目标依赖 android_logger/log_panics，
        // 避免 host 侧 cargo check 因 target-specific dependency 缺失而失败。
        info!("[InteractiveBox::on_init] Starting application initialization");
        #[cfg(target_os = "android")]
        {
            android_logger::init_once(
                android_logger::Config::default()
                    .with_max_level(log::LevelFilter::Trace)
                    .with_tag("RustlibExample"),
            );
            debug!("[InteractiveBox::on_init] Android logger initialized with Trace level");

            // 初始化 panic 日志记录
            log_panics::init();
            debug!("[InteractiveBox::on_init] Panic logger initialized");
        }

        info!(
            "[InteractiveBox::on_init] Application initialization started. Version: 1.0.0-example"
        );

        // 设置初始坐标为 (500, 500)
        self.x = 500.0;
        self.y = 500.0;
        debug!(
            "[InteractiveBox::on_init] Initial box coordinates set to x: {}, y: {}",
            self.x, self.y
        );
        info!("[InteractiveBox::on_init] Application initialization completed successfully");
    }

    // 屏幕尺寸变化回调
    fn on_resize(&mut self, width: u32, height: u32) {
        info!("[InteractiveBox::on_resize] Surface resize event received");
        info!(
            "[InteractiveBox::on_resize] New surface dimensions: {}x{}",
            width, height
        );

        // 更新屏幕尺寸
        self.screen_width = width as f32;
        self.screen_height = height as f32;
        debug!(
            "[InteractiveBox::on_resize] Updated screen dimensions - width: {}, height: {}",
            self.screen_width, self.screen_height
        );

        // 将方块重新定位到屏幕中心
        self.x = self.screen_width / 2.0;
        self.y = self.screen_height / 2.0;
        debug!(
            "[InteractiveBox::on_resize] Box repositioned to screen center: ({}, {})",
            self.x, self.y
        );

        trace!(
            "[InteractiveBox::on_resize] Box position recalibrated to screen center: ({}, {})",
            self.x, self.y
        );
        info!("[InteractiveBox::on_resize] Surface resize handling completed");
    }

    // 触摸事件回调
    fn on_touch(&mut self, action: i32, x: f32, y: f32) {
        trace!("[InteractiveBox::on_touch] Touch event received");
        trace!(
            "[InteractiveBox::on_touch] Action: {}, Coordinates: ({:.2}, {:.2})",
            action, x, y
        );

        // 更新方块位置到触摸位置
        self.x = x;
        self.y = y;
        debug!(
            "[InteractiveBox::on_touch] Box position updated to: ({:.2}, {:.2})",
            self.x, self.y
        );

        trace!("[InteractiveBox::on_touch] Touch event handling completed");
    }

    // 绘制回调，每帧调用
    fn on_draw(&mut self, context: &mut RenderContext, dt: f64) {
        trace!("[InteractiveBox::on_draw] Frame rendering started");
        trace!("[InteractiveBox::on_draw] Delta time: {:.2} ms", dt);
        trace!(
            "[InteractiveBox::on_draw] Current box position: ({:.2}, {:.2})",
            self.x, self.y
        );
        trace!(
            "[InteractiveBox::on_draw] Screen dimensions: {:.2}x{:.2}",
            self.screen_width, self.screen_height
        );

        // 1. 绘制深灰色背景 (使用高级 Color API)
        debug!("[InteractiveBox::on_draw] Drawing deep gray background");
        let bg_color = Color::from_rgb8(30, 30, 30);
        debug!(
            "[InteractiveBox::on_draw] Background color: RGB({}, {}, {})",
            30, 30, 30
        );
        context.set_paint(bg_color); // Rust 自动隐式调用 .into() 转换为 Brush
        trace!("[InteractiveBox::on_draw] Paint color set for background");

        let bg_rect = Rect::new(
            0.0,
            0.0,
            self.screen_width as f64,
            self.screen_height as f64,
        );
        debug!(
            "[InteractiveBox::on_draw] Background rectangle: {:?}",
            bg_rect
        );
        context.fill_rect(&bg_rect);
        trace!("[InteractiveBox::on_draw] Background rectangle filled successfully");

        // 2. 计算方块坐标
        debug!("[InteractiveBox::on_draw] Calculating box coordinates");
        let box_size = 300.0;
        let half_size = box_size / 2.0;
        debug!(
            "[InteractiveBox::on_draw] Box size: {}, half size: {}",
            box_size, half_size
        );

        let target_rect = Rect::new(
            (self.x - half_size) as f64,
            (self.y - half_size) as f64,
            (self.x + half_size) as f64,
            (self.y + half_size) as f64,
        );
        debug!(
            "[InteractiveBox::on_draw] Target box rectangle: {:?}",
            target_rect
        );

        // 3. 绘制亮绿色方块 (使用高级 Color API)
        debug!("[InteractiveBox::on_draw] Drawing bright green box");
        let fg_color = Color::from_rgb8(0, 255, 128);
        debug!(
            "[InteractiveBox::on_draw] Box color: RGB({}, {}, {})",
            0, 255, 128
        );
        context.set_paint(fg_color);
        trace!("[InteractiveBox::on_draw] Paint color set for box");

        // 注意：vello_cpu 原生的 fill_rect 不直接支持圆角，如果需要圆角，
        // 可以查阅 vello_cpu 的 fill_path API，或者直接绘制直角矩形测试链路。
        context.fill_rect(&target_rect);
        trace!("[InteractiveBox::on_draw] Box rectangle filled successfully");

        trace!("[InteractiveBox::on_draw] Frame rendering completed successfully");
    }
}

// 宏定义应用入口，指定 JNI 类路径
rustlib_main!(InteractiveBox, "com/example/demo/VelloSurfaceView");
