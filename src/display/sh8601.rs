//! SH8601 SPI 驱动 — Waveshare 1.75" AMOLED (ESP32-S3).
//!
//! 硬件事实:
//! - SH8601 默认驱动
//! - SPI: SCK=GPIO10, MOSI=GPIO11, MISO=GPIO13, CS=GPIO12, RST=GPIO15
//! - 466×466 RGB565, 单线 SPI 40 MHz
//! - Init 序列移植自 espressif/esp_lcd_sh8601.c

#![allow(dead_code, unused_imports)]

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::gpio::{Output, PinDriver};
use esp_idf_sys::*;

extern crate alloc;

pub const WIDTH: u16 = 466;
pub const HEIGHT: u16 = 466;
pub const PIXEL_BYTES: usize = 2;
pub const FRAMEBUFFER_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize) * PIXEL_BYTES;

const SPI_CLOCK_HZ: i32 = 40_000_000;
const SPI_TRANS_CHUNK: usize = 4096;

pub struct Sh8601 {
    spi: spi_device_handle_t,
    _cs: PinDriver<'static, Output>,
    _rst: PinDriver<'static, Output>,
    pub framebuffer: &'static mut [u16],
}

unsafe impl Send for Sh8601 {}

impl Sh8601 {
    pub fn init(peripherals: Peripherals) -> anyhow::Result<Self> {
        log::info!("SH8601::init — 466x466 SPI 40MHz");

        // [1] SPI3 bus init — ESP-IDF 5.5 bindgen: mosi 在 union __bindgen_anon_1
        let mut bus_cfg = spi_bus_config_t::default();
        bus_cfg.__bindgen_anon_1.mosi_io_num = 11;
        bus_cfg.__bindgen_anon_2.miso_io_num = 13;
        bus_cfg.sclk_io_num = 10;
        bus_cfg.max_transfer_sz = FRAMEBUFFER_SIZE as i32;
        unsafe {
            let err = spi_bus_initialize(spi_host_device_t_SPI3_HOST, &bus_cfg, 0);
            if err != 0 {
                return Err(anyhow::anyhow!("spi_bus_initialize failed: err={}", err));
            }
        }

        // [2] SPI device init — 用 Default 后改字段
        let mut dev_cfg = spi_device_interface_config_t::default();
        dev_cfg.mode = 0;
        dev_cfg.clock_speed_hz = SPI_CLOCK_HZ;
        dev_cfg.spics_io_num = 12;
        dev_cfg.queue_size = 7;
        dev_cfg.flags = 1; // SPI_DEVICE_NO_DUMMY
        let mut spi_handle: spi_device_handle_t = core::ptr::null_mut();
        unsafe {
            let err = spi_bus_add_device(spi_host_device_t_SPI3_HOST, &dev_cfg, &mut spi_handle);
            if err != 0 {
                return Err(anyhow::anyhow!("spi_bus_add_device failed: err={}", err));
            }
        }

        // [3] 硬件 reset
        let mut rst = PinDriver::output(peripherals.pins.gpio15)?;
        rst.set_low()?;
        FreeRtos::delay_ms(10);
        rst.set_high()?;
        FreeRtos::delay_ms(150);

        // [4] SH8601 init 序列
        Self::write_cmd(spi_handle, 0x36, &[0x00])?;
        Self::write_cmd(spi_handle, 0x3A, &[0x55])?;
        Self::write_cmd(spi_handle, 0x44, &[0x00, 0xC8])?;
        Self::write_cmd(spi_handle, 0x35, &[0x00])?;
        Self::write_cmd(spi_handle, 0x53, &[0x20])?;
        FreeRtos::delay_ms(25);
        Self::write_cmd(spi_handle, 0x29, &[])?;
        FreeRtos::delay_ms(120);

        // [5] CS 由 SPI device 自动管理
        let cs = PinDriver::output(peripherals.pins.gpio12)?;

        // [6] PSRAM framebuffer 分配
        let framebuffer = Self::alloc_psram_framebuffer()?;

        log::info!("SH8601::init 完成 — framebuffer {} KB", FRAMEBUFFER_SIZE / 1024);

        Ok(Self { spi: spi_handle, _cs: cs, _rst: rst, framebuffer })
    }

    fn alloc_psram_framebuffer() -> anyhow::Result<&'static mut [u16]> {
        let ptr = unsafe {
            heap_caps_malloc(FRAMEBUFFER_SIZE, MALLOC_CAP_SPIRAM)
        } as *mut u16;
        if ptr.is_null() {
            return Err(anyhow::anyhow!("PSRAM framebuffer 分配失败 ({} bytes)", FRAMEBUFFER_SIZE));
        }
        unsafe { core::ptr::write_bytes(ptr as *mut u8, 0, FRAMEBUFFER_SIZE) };
        Ok(unsafe { core::slice::from_raw_parts_mut(ptr, WIDTH as usize * HEIGHT as usize) })
    }

    fn write_cmd(spi: spi_device_handle_t, cmd: u8, data: &[u8]) -> anyhow::Result<()> {
        let cmd_buf = [cmd; 1];
        let mut trans = spi_transaction_t::default();
        trans.length = 8; // bits
        trans.__bindgen_anon_1.tx_buffer = cmd_buf.as_ptr() as *const _;
        unsafe {
            let err = spi_device_polling_transmit(spi, &mut trans);
            if err != 0 {
                return Err(anyhow::anyhow!("spi transmit cmd failed: err={}", err));
            }
        }

        if !data.is_empty() {
            trans.length = data.len() * 8;
            trans.__bindgen_anon_1.tx_buffer = data.as_ptr() as *const _;
            unsafe {
                let err = spi_device_polling_transmit(spi, &mut trans);
                if err != 0 {
                    return Err(anyhow::anyhow!("spi transmit data failed: err={}", err));
                }
            }
        }

        Ok(())
    }

    pub fn push_framebuffer(&mut self) -> anyhow::Result<()> {
        Self::write_cmd(self.spi, 0x2A, &[0, 0, ((WIDTH - 1) >> 8) as u8, ((WIDTH - 1) & 0xFF) as u8])?;
        Self::write_cmd(self.spi, 0x2B, &[0, 0, ((HEIGHT - 1) >> 8) as u8, ((HEIGHT - 1) & 0xFF) as u8])?;
        Self::write_cmd(self.spi, 0x2C, &[])?;

        let fb_bytes = unsafe {
            core::slice::from_raw_parts(
                self.framebuffer.as_ptr() as *const u8,
                FRAMEBUFFER_SIZE,
            )
        };
        let mut trans = spi_transaction_t::default();
        for chunk in fb_bytes.chunks(SPI_TRANS_CHUNK) {
            trans.length = chunk.len() * 8;
            trans.__bindgen_anon_1.tx_buffer = chunk.as_ptr() as *const _;
            unsafe {
                let err = spi_device_polling_transmit(self.spi, &mut trans);
                if err != 0 {
                    return Err(anyhow::anyhow!("spi transmit framebuffer failed: err={}", err));
                }
            }
        }
        Ok(())
    }

    pub fn test_pattern(&mut self, rgb565: u16) -> anyhow::Result<()> {
        for px in self.framebuffer.iter_mut() { *px = rgb565; }
        self.push_framebuffer()?;
        log::info!("test_pattern(0x{:04X}) pushed", rgb565);
        Ok(())
    }

    pub fn framebuffer(&mut self) -> &mut [u16] { self.framebuffer }
    pub fn size() -> (u16, u16) { (WIDTH, HEIGHT) }
}
