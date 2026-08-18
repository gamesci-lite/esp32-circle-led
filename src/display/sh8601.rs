//! SH8601 SPI 驱动 — Waveshare 1.75" AMOLED (ESP32-S3).
//!
//! 硬件事实（实测自官方 wiki + 示例代码）：
//! - SH8601 默认驱动（R7 电阻在 SH8601 位；移到 CO5300 位换驱动）
//! - SPI 引脚: SCK=GPIO10, MOSI=GPIO11, CS=GPIO12, RST=GPIO15
//! - 466×466 RGB565
//! - 单线 SPI 时钟 40 MHz（保守；QSPI 可上 80 MHz, 待升级）
//!
//! 实现策略：
//! - 阶段 1（当前）：单线 SPI — 复用官方 SH8601 init 序列, 先验证屏接线
//! - 阶段 2（TODO）: 升级到 QSPI 4-line — 用 esp-idf-sys FFI 调 `spi_bus_initialize` QSPI 模式
//!
//! 关键约束：
//! - framebuffer 必须放 PSRAM（466×466×2 ≈ 434KB, 远超 IRAM 200KB）
//! - 坐标对齐: SH8601 要求 x/y 起始坐标偶数对齐（详见 README「Notes」）
//!
//! Init 序列来源: <https://github.com/espressif/esp-iot-solution/blob/master/components/display/lcd/esp_lcd_sh8601/esp_lcd_sh8601.c>

#![allow(dead_code)]

use core::alloc::Layout;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::{SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig};
use esp_idf_hal::units::Hertz;
use esp_idf_svc::hal::gpio::{Gpio12, Gpio15, Output, PinDriver};

/// 屏分辨率
pub const WIDTH: u16 = 466;
pub const HEIGHT: u16 = 466;
/// 像素字节数 (RGB565 = 2 bytes/pixel)
pub const PIXEL_BYTES: usize = 2;
/// Framebuffer 总字节数 (~434KB, 必须放 PSRAM)
pub const FRAMEBUFFER_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize) * PIXEL_BYTES;

/// 屏驱动实例 — 持有 SPI device driver + PSRAM framebuffer
pub struct Sh8601<'d> {
    spi: SpiDeviceDriver<'d, SpiDriver<'d>>,
    _cs: PinDriver<'d, Gpio12, Output>,
    _rst: PinDriver<'d, Gpio15, Output>,
    /// PSRAM framebuffer — Slint 渲染写入这里
    pub framebuffer: &'static mut [u16],
}

impl<'d> Sh8601<'d> {
    /// 初始化 SH8601 + 分配 PSRAM framebuffer
    pub fn init(peripherals: Peripherals) -> anyhow::Result<Self> {
        log::info!("SH8601::init — 466x466 SPI 40MHz (QSPI 待升级)");

        // [1] SPI3 bus + device init
        // 注: SPI3 在 ESP32-S3 上无内部 flash/psram 占用, 可用作 LCD
        let spi_driver = SpiDriver::new(
            peripherals.spi3,
            peripherals.pins.gpio10, // SCK
            peripherals.pins.gpio11, // MOSI (D0)
            peripherals.pins.gpio13, // MISO (D1, 单线模式不用, 接上避免浮空)
            peripherals.pins.gpio14, // D2 (单线模式不用)
            peripherals.pins.gpio9,  // D3 (单线模式不用)
            &SpiDriverConfig::new(),
        )?;
        let spi = SpiDeviceDriver::new(
            spi_driver,
            Some(peripherals.pins.gpio12), // CS
            &SpiConfig::new().baudrate(Hertz(40_000_000)),
        )?;

        // [2] 硬件 reset
        let mut rst = PinDriver::output(peripherals.pins.gpio15)?;
        rst.set_low()?;
        FreeRtos::delay_ms(10);
        rst.set_high()?;
        FreeRtos::delay_ms(150);

        // [3] SH8601 init 序列（移植自 espressif/esp_lcd_sh8601.c）
        Self::write_cmd(&spi, 0x36, &[0x00])?;          // MADCTL: RGB
        Self::write_cmd(&spi, 0x3A, &[0x55])?;          // COLMOD: 16bpp RGB565
        Self::write_cmd(&spi, 0x44, &[0x00, 0xC8])?;    // TE line (scanline 200)
        Self::write_cmd(&spi, 0x35, &[0x00])?;          // TE on
        Self::write_cmd(&spi, 0x53, &[0x20])?;          // WCTR (write control)
        FreeRtos::delay_ms(25);
        Self::write_cmd(&spi, 0x29, &[])?;               // Display ON
        FreeRtos::delay_ms(120);

        // [4] CS 由 SPI device driver 自动管理, 这里保持所有权防止 drop
        let cs = PinDriver::output(peripherals.pins.gpio12)?;

        // [5] PSRAM framebuffer 分配
        let framebuffer = Self::alloc_psram_framebuffer()?;

        log::info!("SH8601::init 完成 — framebuffer {} KB in PSRAM", FRAMEBUFFER_SIZE / 1024);

        Ok(Self { spi, _cs: cs, _rst: rst, framebuffer })
    }

    /// 从 PSRAM 分配 framebuffer (~434KB)
    fn alloc_psram_framebuffer() -> anyhow::Result<&'static mut [u16]> {
        let layout = Layout::from_size_align(FRAMEBUFFER_SIZE, 16)
            .map_err(|e| anyhow::anyhow!("layout 错误: {:?}", e))?;

        // Global allocator 已由 esp_alloc::psram_allocator::init 启用 PSRAM heap
        let ptr = unsafe { alloc::alloc::Global.alloc(layout) } as *mut u16;
        if ptr.is_null() {
            return Err(anyhow::anyhow!("PSRAM framebuffer 分配失败 ({} bytes)", FRAMEBUFFER_SIZE));
        }
        unsafe { core::ptr::write_bytes(ptr as *mut u8, 0, FRAMEBUFFER_SIZE) };
        Ok(unsafe { core::slice::from_raw_parts_mut(ptr, WIDTH as usize * HEIGHT as usize) })
    }

    /// 发送 SH8601 命令（单线 SPI）
    fn write_cmd(spi: &SpiDeviceDriver<'_, SpiDriver<'_>>, cmd: u8, data: &[u8]) -> anyhow::Result<()> {
        // 注: 借用检查器要求 spi 是 &mut, 这里 & 是因为 SpiDeviceDriver 内部自己处理 CS
        spi.write(&[cmd])?;
        if !data.is_empty() {
            spi.write(data)?;
        }
        Ok(())
    }

    /// 设置 column address window (CASET 0x2A)
    fn set_col_addr(&mut self, x0: u16, x1: u16) -> anyhow::Result<()> {
        Self::write_cmd(&self.spi, 0x2A, &[
            (x0 >> 8) as u8,
            (x0 & 0xFF) as u8,
            ((x1 - 1) >> 8) as u8,
            ((x1 - 1) & 0xFF) as u8,
        ])
    }

    /// 设置 row address window (RASET 0x2B)
    fn set_row_addr(&mut self, y0: u16, y1: u16) -> anyhow::Result<()> {
        Self::write_cmd(&self.spi, 0x2B, &[
            (y0 >> 8) as u8,
            (y0 & 0xFF) as u8,
            ((y1 - 1) >> 8) as u8,
            ((y1 - 1) & 0xFF) as u8,
        ])
    }

    /// 推送 framebuffer 全屏到屏（draw_bitmap）
    pub fn push_framebuffer(&mut self) -> anyhow::Result<()> {
        self.set_col_addr(0, WIDTH)?;
        self.set_row_addr(0, HEIGHT)?;
        // 写 RAMWR (0x2C) + 全帧像素
        self.spi.write(&[0x2C])?;
        let fb_bytes = unsafe {
            core::slice::from_raw_parts(
                self.framebuffer.as_ptr() as *const u8,
                FRAMEBUFFER_SIZE,
            )
        };
        self.spi.write(fb_bytes)?;
        Ok(())
    }

    /// 全屏刷纯色 (RGB565) — 验证屏接线
    pub fn test_pattern(&mut self, rgb565: u16) -> anyhow::Result<()> {
        for px in self.framebuffer.iter_mut() {
            *px = rgb565;
        }
        self.push_framebuffer()?;
        log::info!("test_pattern(0x{:04X}) pushed", rgb565);
        Ok(())
    }

    /// 拿到 framebuffer 切片 — 供 Slint WindowAdapter 写入
    pub fn framebuffer(&mut self) -> &mut [u16] {
        self.framebuffer
    }

    /// 屏尺寸常量
    pub fn size() -> (u16, u16) {
        (WIDTH, HEIGHT)
    }
}

// ============================================================================
// 待升级到 QSPI 的说明
// ============================================================================
//
// 单线 SPI 40MHz 推送全帧 ~434KB 需要约 87ms（很慢，FPS < 12）。
// 升级到 QSPI 4-line 80MHz 后 ~ 11ms（FPS > 60）。
//
// 升级路径：
//   1. 创建 src/idf_component.yml 引入 espressif/esp_lcd_sh8601 (官方 C 组件)
//   2. 改用 esp_idf_sys::spi_bus_initialize + esp_lcd_new_panel_io_spi (QSPI 模式)
//   3. 移除本文件的 SPI 单线 init 序列, 改用 esp_lcd_panel_init
//   4. draw_bitmap 改用 esp_lcd_panel_draw_bitmap
//
// 当前 PR 阶段：单线 SPI 跑通, 验证 init 序列 + 接线正确。
// ============================================================================
