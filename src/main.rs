//! 入口：初始化 ESP-IDF + SH8601 + Slint，跑 UI 循环。
//!
//! 阶段：
//!   1. link_patches() + EspLogger
//!   2. PSRAM allocator (esp-alloc)
//!   3. SH8601 init + QSPI driver
//!   4. Slint platform install (自定义 WindowAdapter → SH8601 framebuffer)
//!   5. App::run() 事件循环

use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::link_patches;

mod display;
mod touch;
mod ui;

#[esp_idf_svc::main]
fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    log::info!("esp32-circle-led booting");

    // 1. PSRAM allocator
    // esp_alloc::psram_allocator::init();

    // 2. SH8601 init (QSPI 4-line)
    // let mut lcd = display::sh8601::Sh8601::init()?;
    // lcd.test_pattern()?; // 全屏刷红验证

    // 3. GT911 触摸 (可选, 第一阶段只点亮屏)
    // let touch = touch::Gt911::init()?;

    // 4. Slint platform install
    // ui::platform::install()?;
    // 5. App::run()

    log::info!("boot done — TODO: 接入 Slint 循环");
    Ok(())
}
