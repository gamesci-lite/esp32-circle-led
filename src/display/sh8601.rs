//! SH8601 QSPI 驱动 — Waveshare 1.75" AMOLED (ESP32-S3).
//!
//! 硬件事实（实测自官方 wiki + 示例代码）：
//! - SH8601 默认驱动（R7 电阻在 SH8601 位；移到 CO5300 位则换驱动）
//! - QSPI 4-line: SCK=10, D0=11, D1=13, D2=14, D3=9, CS=12, RST=15, TE=38
//! - 466×466 RGB565
//! - 最高 QSPI 时钟 80 MHz
//!
//! 关键约束：
//! - framebuffer 必须放 PSRAM（466×466×2 ≈ 434KB，远超 IRAM 200KB）
//! - QSPI 走 SPI3 子模式（ESP32-S3 SPI0/1 留给 flash/psram）
//!
//! TODO（按顺序点亮屏）：
//!   [1] esp-alloc PSRAM 初始化 + framebuffer 分配
//!   [2] SPI3 master + QSPI driver init
//!   [3] SH8601 init 序列（sleep out / display on / col/row addr / pixel format 16bit）
//!   [4] test_pattern：全屏刷红色验证接线
//!   [5] push_framebuffer：QSPI write_memory_continue 把 buffer 推到屏
//!   [6] 与 Slint platform 集成（暴露 framebuffer 指针给 WindowAdapter）
//!
//! 参考：
//! - 官方 init 序列: https://github.com/waveshare/ESP32-S3-1.75inch-AMOLED/blob/main/examples/sh8601_test/
//! - LovyanGFX panel config: `LGFX_S3AM175`

#![allow(dead_code)]

use esp_idf_hal::delay;
use esp_idf_hal::gpio;
use esp_idf_hal::spi;

pub const WIDTH: u16 = 466;
pub const HEIGHT: u16 = 466;
/// 像素字节数 (RGB565 = 2 bytes/pixel)
pub const PIXEL_BYTES: usize = 2;
/// Framebuffer 总字节数 (~434KB)
pub const FRAMEBUFFER_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize) * PIXEL_BYTES;

pub struct Sh8601<'d> {
    _spi: spi::SpiDeviceDriver<'d, spi::SpiDriver<'d>>,
    cs: gpio::PinDriver<'d, gpio::Gpio12, gpio::Output>,
    rst: gpio::PinDriver<'d, gpio::Gpio15, gpio::Output>,
    _te: gpio::PinDriver<'d, gpio::Gpio38, gpio::Input>,
    _backlight: gpio::PinDriver<'d, gpio::Gpio47, gpio::Output>, // 视实际板子背光引脚
    framebuffer: &'static mut [u8],
}

impl<'d> Sh8601<'d> {
    /// 初始化 SH8601 + 分配 PSRAM framebuffer
    ///
    /// 当前是占位实现 — 第一阶段只构建工程, 不实际驱动屏
    pub fn init() -> anyhow::Result<Self> {
        log::info!("SH8601::init 占位 — TODO 见模块顶部");
        Ok(Self {
            _spi: unsafe { core::mem::zeroed() },  // 占位
            cs: unsafe { core::mem::zeroed() },
            rst: unsafe { core::mem::zeroed() },
            _te: unsafe { core::mem::zeroed() },
            _backlight: unsafe { core::mem::zeroed() },
            framebuffer: &mut [0u8; 0],
        })
    }

    /// 全屏刷纯色 — 第一阶段验证接线
    pub fn test_pattern(&mut self, _rgb565: u16) -> anyhow::Result<()> {
        log::info!("SH8601::test_pattern 占位");
        delay::FreeRtos::delay_ms(10);
        Ok(())
    }

    /// 把 framebuffer 整帧推到屏 (QSPI write_memory_continue)
    pub fn push_framebuffer(&mut self) -> anyhow::Result<()> {
        log::info!("SH8601::push_framebuffer 占位");
        Ok(())
    }

    /// 拿到 framebuffer 切片 — 供 Slint WindowAdapter 写入
    pub fn framebuffer(&mut self) -> &mut [u8] {
        self.framebuffer
    }
}
