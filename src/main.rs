//! 入口: AXP2101 上电 → CO5300 QSPI init → GUI 桌面渲染推送 → 心跳.
//!
//! M1: Squareline 风格桌面 (状态栏 + 2x2 应用网格 + 页点), 静态推一帧.
//! M3 接触摸后改为事件循环重渲染.
//! 编译: bash tools/remote_build.sh (构建机 ubuntu-hs-main, esp 工具链).

use esp_idf_sys::{
    esp_log_level_set, esp_log_write, vTaskDelay,
    esp_log_level_t_ESP_LOG_INFO, esp_log_level_t_ESP_LOG_WARN,
};

mod display;
mod gui;

fn log_info(tag: &str, msg: &str) {
    unsafe {
        esp_log_write(esp_log_level_t_ESP_LOG_INFO, tag.as_ptr() as *const _,
            b"%s\n\0".as_ptr() as *const _, msg.as_ptr() as *const _);
    }
}

#[allow(dead_code)]
fn log_warn(tag: &str, msg: &str) {
    unsafe {
        esp_log_write(esp_log_level_t_ESP_LOG_WARN, tag.as_ptr() as *const _,
            b"%s\n\0".as_ptr() as *const _, msg.as_ptr() as *const _);
    }
}

fn main() -> anyhow::Result<()> {
    // link_patches 初始化 C runtime (必须在最前面)
    esp_idf_sys::link_patches();
    unsafe {
        esp_log_level_set(b"co5300\0".as_ptr() as *const _, esp_log_level_t_ESP_LOG_INFO);
    }

    log_info("circle", "===========================================");
    log_info("circle", "  esp32-circle-led booting (M1 GUI 桌面)");
    log_info("circle", "  硬件: Waveshare ESP32-S3-Touch-AMOLED-1.75C");
    log_info("circle", "  驱动: CO5300 QSPI 40MHz + embedded-graphics");
    log_info("circle", "===========================================");

    let mut lcd = display::co5300::Co5300::init()?;
    log_info("circle", "[1] Co5300 init done");

    // M1: 渲染桌面第 0 页 (状态栏 + 2x2 网格 + 页点) 并推送
    gui::desktop::render(lcd.framebuffer, 0);
    lcd.push()?;
    log_info("circle", "[2] desktop rendered + pushed — 看屏!");

    // 心跳 (M3 接触摸后改为事件循环)
    let mut tick = 0u32;
    loop {
        unsafe { vTaskDelay(1000) };  // tick≈1s (1000Hz) 或 10s (100Hz), 仅影响日志节奏
        tick += 1;
        if tick % 5 == 0 {
            log_info("circle", "heartbeat");
        }
    }
}
