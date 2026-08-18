//! 入口: PSRAM init → SH8601 init → Slint platform install → App run.
//!
//! 编译验证: 远端构建机 ubuntu-hs-main (本机无工具链).
//!
//! ## 阶段状态
//!   - [x] A1: PSRAM allocator init
//!   - [x] A2: SH8601 driver (单线 SPI 40MHz, 待升级 QSPI)
//!   - [x] A3: PSRAM framebuffer alloc (434KB)
//!   - [x] A4: test_pattern (全屏刷红验证接线)
//!   - [~] B1: Slint Platform 自定义 (接口完整, 渲染循环待实现)
//!   - [x] B2: app.slint UI 复刻 (状态栏 + 2x2 网格 + 页码)
//!   - [x] B3: 4 个应用图标占位 (程序化绘制, 待替换 PNG)
//!   - [~] B4: main.rs 接入 (链路完整, Slint 渲染循环在 platform.rs)

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::link_patches;

mod display;
mod touch;
mod ui;

#[esp_idf_svc::main]
fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    log::info!("===========================================");
    log::info!("  esp32-circle-led booting");
    log::info!("  硬件: Waveshare ESP32-S3 1.75\" AMOLED");
    log::info!("  渲染: Slint 1.x (声明式 + dirty region)");
    log::info!("===========================================");

    // [A1] PSRAM allocator
    esp_alloc::psram_allocator::init();
    log::info!("[A1] PSRAM allocator initialized");

    // [A2-A4] SH8601 init
    let peripherals = Peripherals::take()?;
    let mut lcd = display::sh8601::Sh8601::init(peripherals)?;
    log::info!("[A2-A3] SH8601 init + PSRAM framebuffer alloc done");

    // [A4] test_pattern: 全屏刷红 (0xF800)
    log::info!("[A4] test_pattern: 全屏刷红 (0xF800) 验证屏接线...");
    lcd.test_pattern(0xF800)?;
    FreeRtos::delay_ms(2000);

    // [B1] Slint platform install
    log::info!("[B1] Slint platform install...");
    if let Err(e) = ui::platform::install(&mut lcd) {
        log::warn!("Slint platform install 失败 ({:?}) — UI 暂不显示, 仅保留屏驱动验证", e);
        log::warn!("保持 test_pattern 红色屏, 5 秒后退出");
        FreeRtos::delay_ms(5000);
        return Ok(());
    }

    // [B2] App 实例 + show
    let app = ui::App::new()?;
    app.show()?;
    log::info!("[B2] App shown");

    // [B4] 主事件循环
    log::info!("[B4] 主事件循环 — 60 FPS");
    // TODO: Slint 自定义 Platform::run_event_loop() 实现后,
    // 这里直接调用 ui::platform::run_event_loop(&app) 即可.
    // 当前占位: 保持屏亮, 不退出.
    loop {
        FreeRtos::delay_ms(1000);
        log::info!("heartbeat — UI 渲染循环待 platform.rs 完整实现");
    }

    // Ok(())
}
