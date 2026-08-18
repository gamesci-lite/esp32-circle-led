//! 入口: PSRAM init → SH8601 init → test_pattern 验证屏接线.
//!
//! 编译验证: 远端构建机 ubuntu-hs-main (本机无工具链).

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_sys::{
    esp_log_level_set, esp_log_write,
    esp_log_level_t_ESP_LOG_INFO, esp_log_level_t_ESP_LOG_WARN,
};

mod display;

fn log_info(tag: &str, msg: &str) {
    unsafe {
        esp_log_write(esp_log_level_t_ESP_LOG_INFO, tag.as_ptr() as *const _,
            b"%s\n\0".as_ptr() as *const _, msg.as_ptr() as *const _);
    }
}

fn log_warn(tag: &str, msg: &str) {
    unsafe {
        esp_log_write(esp_log_level_t_ESP_LOG_WARN, tag.as_ptr() as *const _,
            b"%s\n\0".as_ptr() as *const _, msg.as_ptr() as *const _);
    }
}

fn main() -> anyhow::Result<()> {
    // link_patches 初始化 C runtime (必须在最前面)
    esp_idf_sys::link_patches();
    // 让 ESP_LOG 输出到 UART (默认 INFO 级别)
    unsafe {
        esp_log_level_set(b"sh8601\0".as_ptr() as *const _, esp_log_level_t_ESP_LOG_INFO);
    }

    log_info("circle", "===========================================");
    log_info("circle", "  esp32-circle-led booting");
    log_info("circle", "  硬件: Waveshare ESP32-S3 1.75\" AMOLED");
    log_info("circle", "  阶段: 点亮屏验证 (test_pattern)");
    log_info("circle", "===========================================");

    // [A1] PSRAM allocator — 通过 heap_caps_malloc(MALLOC_CAP_SPIRAM) 分配 framebuffer,
    //     PSRAM heap 在 sdkconfig 中通过 CONFIG_SPIRAM=y 自动启用.
    log_info("circle", "[A1] PSRAM allocator: 通过 heap_caps_malloc(MALLOC_CAP_SPIRAM) 分配 framebuffer");

    // [A2-A3] SH8601 init + PSRAM framebuffer alloc
    let peripherals = Peripherals::take()?;
    let mut lcd = display::sh8601::Sh8601::init(peripherals)?;
    log_info("circle", "[A2-A3] SH8601 init + PSRAM framebuffer alloc done");

    // [A4] test_pattern: 全屏刷红 (0xF800) 验证屏接线
    log_info("circle", "[A4] test_pattern: 全屏刷红 (0xF800) ...");
    lcd.test_pattern(0xF800)?;
    log_info("circle", "[A4] test_pattern pushed — 屏应亮红色");

    // 心跳, 不退出
    let mut tick = 0u32;
    loop {
        FreeRtos::delay_ms(1000);
        tick += 1;
        if tick % 5 == 0 {
            // 格式化消息用 std::format! 会触发 iDisplay fmt 重定位 — 简单字符串避免
            log_info("circle", "heartbeat tick");
            log_warn("circle", "fb[0]=FFFF");
        }
    }
}
