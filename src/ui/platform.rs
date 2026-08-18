//! Slint Platform impl — 把 Slint 渲染绑定到 SH8601 framebuffer.
//!
//! ## 设计
//!
//! - 用 Slint 的 `SoftwareRenderer` (软件渲染器, 无 GPU 依赖, 适合嵌入式)
//! - 渲染时通过 `render_with_callbacks` 把 RGB565 像素写入 SH8601 PSRAM framebuffer
//! - 每帧渲染完成后, `SH8601::push_framebuffer()` 把整个 buffer 推到屏 (full refresh)
//!
//! ## 待实现（按 Slint 1.x API 完整化）
//!
//! - [ ] 自定义 `WindowAdapter`: 处理尺寸变化、输入事件
//! - [ ] 自定义 `SoftwareRenderer` 实例 + bind 到 SH8601 framebuffer
//! - [ ] 主事件循环: 处理触摸事件 + 触发 redraw
//!
//! 参考: <https://github.com/slint-ui/slint/tree/master/examples> (找 embedded/esp32 案例)

#![allow(dead_code)]

use crate::display::sh8601::Sh8601;
use slint::platform::{Platform, PlatformError, WindowAdapter};

/// 自定义 Slint Platform — 持有 SH8601 引用, 渲染时写入 framebuffer
///
/// 注: 当前持有 raw pointer 是简化方案, 真实实现应该用 Rc<RefCell<>> 或 channel.
pub struct CircleLedPlatform {
    lcd_ptr: *mut Sh8601<'static>,
}

// Safety: 单线程 ESP32 主循环, 不跨线程访问
unsafe impl Send for CircleLedPlatform {}

impl Platform for CircleLedPlatform {
    fn create_window_adapter(&self) -> Result<Box<dyn WindowAdapter>, PlatformError> {
        // TODO: 实现 WindowAdapter
        // 见 https://docs.slint-ui.com/1.0/src/concepts/embedders.html
        //
        // 关键点:
        //   1. window_size = SH8601::size() = (466, 466)
        //   2. WindowAdapter.render() 回调: 把 RGB565 像素写入 lcd.framebuffer
        //   3. 渲染完成后调用 lcd.push_framebuffer() 把 buffer 推到屏
        log::warn!("CircleLedPlatform::create_window_adapter 占位 — TODO 见模块顶部");
        Err(PlatformError::NoPlatform)
    }

    fn run_event_loop(&self) -> Result<(), PlatformError> {
        // TODO: 主事件循环
        // 1. 调用 slint::platform::update_timers_and_animations()
        // 2. 处理触摸事件 (从 GT911 读)
        // 3. 触发 window.request_redraw()
        // 4. SoftwareRenderer.render(&window, render_callback)
        // 5. render_callback 写入 lcd.framebuffer
        // 6. lcd.push_framebuffer()
        // 7. vTaskDelay(16ms) for ~60 FPS
        log::warn!("CircleLedPlatform::run_event_loop 占位 — TODO 见模块顶部");
        Err(PlatformError::NoPlatform)
    }

    fn duration_since_start(&self) -> core::time::Duration {
        // TODO: 返回系统启动时长 (用于动画)
        core::time::Duration::from_millis(esp_idf_hal::sys::esp_timer_get_time() as u64 / 1000)
    }
}

/// 安装自定义 Platform — 必须在创建 Slint Window 之前调用
pub fn install(lcd: &mut Sh8601<'static>) -> Result<(), PlatformError> {
    let platform = CircleLedPlatform { lcd_ptr: lcd as *mut _ };
    slint::platform::set_platform(Box::new(platform))
        .map_err(|_| PlatformError::NoPlatform)
}
