//! CO5300 QSPI 驱动 — Waveshare ESP32-S3-Touch-AMOLED-1.75C (2026-08-19 已实测点亮)
//!
//! 移植自已验证的 C 固件 v15 (构建机 /root/build/circle-c/main/main.cpp),
//! 驱动核心复用 vendor 的乐鑫官方组件 esp_lcd_co5300 v2.1.0 (与 C 路线同一份源码)。
//!
//! 硬件事实 (实测):
//! - CO5300 驱动 IC (⚠️ 不是 SH8601, wiki 写的是基础版)
//! - QSPI: SDIO0-3 = GPIO 4/5/6/7, SCLK=38, CS=12, RST=1 (TE=13 暂不用)
//! - PMU: AXP2101 @ I2C 0x34 (SDA=15/SCL=14), 显示 rail = ALDO3 (0x90 bit2, 0x94=0x1C→3.3V)
//! - 466×466 RGB565, 列偏移 x_gap=6 (圆屏特性)
//!
//! 命根子 (C 路线 13 轮黑屏的根因):
//!   co5300_vendor_config_t.flags.use_qspi_interface 必须 = 1,
//!   否则组件发命令不加 0x02/0x32 QSPI 协议头, CO5300 全部当 NOP —— 假成功黑屏。

#![allow(dead_code)]

use esp_idf_sys::*;
use core::ffi::c_void;

/// 毫秒级忙等延时 (替代 esp-idf-hal 的 FreeRtos::delay_ms — hal 带 legacy I2C, 见 Cargo.toml)
fn delay_ms(ms: u32) {
    unsafe { esp_rom_delay_us(ms * 1000) }
}

pub const WIDTH: usize = 466;
pub const HEIGHT: usize = 466;
pub const PIXELS: usize = WIDTH * HEIGHT;
pub const FRAMEBUFFER_SIZE: usize = PIXELS * 2;

/// QSPI 像素时钟 — C 路线 40MHz 已实测 (35fps 全帧)
const PCLK_HZ: u32 = 40_000_000;
/// 1.75C 圆屏列偏移
const X_GAP: core::ffi::c_int = 6;
/// SPI max_transfer_sz 按行带分块; 写整帧 434KB 会让驱动找等量内部 RAM 反弹缓冲 → NO_MEM
const MAX_TRANS_CHUNK: usize = WIDTH * 20 * 2;

// QSPI 引脚 (1.75C 实测)
const PIN_SDIO0: core::ffi::c_int = 4;
const PIN_SDIO1: core::ffi::c_int = 5;
const PIN_SDIO2: core::ffi::c_int = 6;
const PIN_SDIO3: core::ffi::c_int = 7;
const PIN_SCLK: core::ffi::c_int = 38;
const PIN_CS: core::ffi::c_int = 12;
const PIN_RST: core::ffi::c_int = 1;
// AXP2101 I2C 引脚
const PIN_I2C_SDA: core::ffi::c_int = 15;
const PIN_I2C_SCL: core::ffi::c_int = 14;

fn esp_check(err: esp_err_t, what: &str) -> anyhow::Result<()> {
    if err != ESP_OK as esp_err_t {
        Err(anyhow::anyhow!("{} failed: err={}", what, err))
    } else {
        Ok(())
    }
}

/// AXP2101 PMU — 显示电源 rail (ALDO3) 使能 + 回读验证
struct Axp2101 {
    _bus: i2c_master_bus_handle_t,
    dev: i2c_master_dev_handle_t,
}

impl Axp2101 {
    fn init() -> anyhow::Result<Self> {
        let mut bus_cfg = i2c_master_bus_config_t::default();
        bus_cfg.i2c_port = 0; // I2C_NUM_0
        bus_cfg.sda_io_num = PIN_I2C_SDA;
        bus_cfg.scl_io_num = PIN_I2C_SCL;
        bus_cfg.__bindgen_anon_1.clk_source = soc_periph_i2c_clk_src_t_I2C_CLK_SRC_DEFAULT;
        bus_cfg.glitch_ignore_cnt = 7;
        bus_cfg.flags.set_enable_internal_pullup(1);
        let mut bus: i2c_master_bus_handle_t = core::ptr::null_mut();
        esp_check(unsafe { i2c_new_master_bus(&bus_cfg, &mut bus) }, "i2c_new_master_bus")?;

        let mut dev_cfg = i2c_device_config_t::default();
        dev_cfg.dev_addr_length = 0; // I2C_ADDR_BIT_LEN_7
        dev_cfg.device_address = 0x34;
        dev_cfg.scl_speed_hz = 400_000;
        let mut dev: i2c_master_dev_handle_t = core::ptr::null_mut();
        esp_check(unsafe { i2c_master_bus_add_device(bus, &dev_cfg, &mut dev) }, "i2c_master_bus_add_device")?;

        let axp = Self { _bus: bus, dev };
        let chip_id = axp.read_reg(0x03)?;
        log::info!("AXP@0x34 chip_id=0x{:02x} (期望 0x4a)", chip_id);

        // enable 所有 LDO rail 兜底 (显示 rail = ALDO3, vthinkxie power.cpp 确认)
        axp.write_reg(0x90, 0x01 | 0x02 | 0x04 | 0x08 | 0x10 | 0x20 | 0x80)?;
        axp.write_reg(0x94, 0x1C)?; // ALDO3 = 3.3V
        // ★ 回读验证 (C 路线第 13 轮只写不读的教训)
        let ldo_ctrl = axp.read_reg(0x90)?;
        let aldo3_v = axp.read_reg(0x94)?;
        log::info!("AXP rails on, 回读 0x90=0x{:02x} (期望含 bit2) 0x94=0x{:02x} (期望 0x1c)", ldo_ctrl, aldo3_v);
        if ldo_ctrl & 0x04 == 0 || aldo3_v != 0x1C {
            return Err(anyhow::anyhow!("AXP2101 rails 未生效: 0x90=0x{:02x} 0x94=0x{:02x}", ldo_ctrl, aldo3_v));
        }
        Ok(axp)
    }

    fn write_reg(&self, reg: u8, val: u8) -> anyhow::Result<()> {
        let buf = [reg, val];
        esp_check(unsafe { i2c_master_transmit(self.dev, buf.as_ptr(), 2, -1) }, "axp write_reg")
    }

    fn read_reg(&self, reg: u8) -> anyhow::Result<u8> {
        let mut val = 0u8;
        esp_check(unsafe { i2c_master_transmit_receive(self.dev, &reg, 1, &mut val, 1, -1) }, "axp read_reg")?;
        Ok(val)
    }
}

pub struct Co5300 {
    panel: esp_lcd_panel_handle_t,
    _axp: Axp2101,
    pub framebuffer: &'static mut [u16],
}

unsafe impl Send for Co5300 {}

impl Co5300 {
    pub fn init() -> anyhow::Result<Self> {
        log::info!("Co5300::init — 466x466 QSPI {}MHz", PCLK_HZ / 1_000_000);

        // [0] AXP2101: 显示 rail ALDO3 上电 (带读回验证)
        let axp = Axp2101::init()?;

        // [1] PSRAM framebuffer (434KB 远超 IRAM)
        let framebuffer = Self::alloc_psram_framebuffer()?;

        // [2] SPI3 bus: 四线 QSPI, DMA 分块
        let mut bus_cfg = spi_bus_config_t::default();
        bus_cfg.__bindgen_anon_1.mosi_io_num = PIN_SDIO0;
        bus_cfg.__bindgen_anon_2.miso_io_num = PIN_SDIO1;
        bus_cfg.__bindgen_anon_3.quadwp_io_num = PIN_SDIO2;
        bus_cfg.__bindgen_anon_4.quadhd_io_num = PIN_SDIO3;
        bus_cfg.sclk_io_num = PIN_SCLK;
        bus_cfg.max_transfer_sz = MAX_TRANS_CHUNK as core::ffi::c_int;
        esp_check(unsafe {
            spi_bus_initialize(spi_host_device_t_SPI3_HOST, &bus_cfg, spi_common_dma_t_SPI_DMA_CH_AUTO)
        }, "spi_bus_initialize")?;

        // [3] esp_lcd panel io: 32-bit 命令 + quad 数据
        let mut io_cfg = esp_lcd_panel_io_spi_config_t::default();
        io_cfg.cs_gpio_num = PIN_CS;
        io_cfg.dc_gpio_num = -1;
        io_cfg.spi_mode = 0;
        io_cfg.pclk_hz = PCLK_HZ;
        io_cfg.trans_queue_depth = 10;
        io_cfg.lcd_cmd_bits = 32;
        io_cfg.lcd_param_bits = 8;
        io_cfg.flags.set_quad_mode(1);
        let mut io: esp_lcd_panel_io_handle_t = core::ptr::null_mut();
        esp_check(unsafe { esp_lcd_new_panel_io_spi(spi_host_device_t_SPI3_HOST as i32, &io_cfg, &mut io) },
                  "esp_lcd_new_panel_io_spi")?;

        // [4] CO5300 panel — ★★★ use_qspi_interface = 1 (命根子, 见模块头注释)
        let mut vendor_config: co5300_vendor_config_t = unsafe { core::mem::zeroed() };
        vendor_config.flags.set_use_qspi_interface(1);
        let mut panel_cfg = esp_lcd_panel_dev_config_t::default();
        panel_cfg.reset_gpio_num = PIN_RST;
        panel_cfg.__bindgen_anon_1.rgb_ele_order = lcd_rgb_element_order_t_LCD_RGB_ELEMENT_ORDER_BGR;
        panel_cfg.bits_per_pixel = 16;
        panel_cfg.vendor_config = &mut vendor_config as *mut _ as *mut c_void;
        let mut panel: esp_lcd_panel_handle_t = core::ptr::null_mut();
        esp_check(unsafe { esp_lcd_new_panel_co5300(io, &panel_cfg, &mut panel) },
                  "esp_lcd_new_panel_co5300")?;

        // [5] 硬件 reset (200ms) + init (组件默认 init 表) + 列偏移 + DISPON
        esp_check(unsafe { gpio_set_level(PIN_RST as gpio_num_t, 0) }, "rst low")?;
        delay_ms(200);
        esp_check(unsafe { gpio_set_level(PIN_RST as gpio_num_t, 1) }, "rst high")?;
        delay_ms(200);
        esp_check(unsafe { esp_lcd_panel_init(panel) }, "esp_lcd_panel_init")?;
        esp_check(unsafe { esp_lcd_panel_set_gap(panel, X_GAP, 0) }, "esp_lcd_panel_set_gap")?;
        esp_check(unsafe { esp_lcd_panel_disp_on_off(panel, true) }, "esp_lcd_panel_disp_on_off")?;

        log::info!("Co5300::init 完成 — fb {} KB, x_gap={}", FRAMEBUFFER_SIZE / 1024, X_GAP);
        Ok(Self { panel, _axp: axp, framebuffer })
    }

    fn alloc_psram_framebuffer() -> anyhow::Result<&'static mut [u16]> {
        let ptr = unsafe {
            heap_caps_malloc(FRAMEBUFFER_SIZE, MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT)
        } as *mut u16;
        if ptr.is_null() {
            return Err(anyhow::anyhow!("PSRAM framebuffer 分配失败 ({} bytes)", FRAMEBUFFER_SIZE));
        }
        unsafe { core::ptr::write_bytes(ptr as *mut u8, 0, FRAMEBUFFER_SIZE) };
        Ok(unsafe { core::slice::from_raw_parts_mut(ptr, PIXELS) })
    }

    /// 全帧推送 (QSPI DMA, 40MHz 实测 ~13.6ms API 时间)
    pub fn push(&self) -> anyhow::Result<()> {
        esp_check(unsafe {
            esp_lcd_panel_draw_bitmap(
                self.panel,
                0,
                0,
                WIDTH as core::ffi::c_int,
                HEIGHT as core::ffi::c_int,
                self.framebuffer.as_ptr() as *const c_void,
            )
        }, "esp_lcd_panel_draw_bitmap")
    }

    /// 红/绿/蓝/白 四色横带 (验证点亮 + 区分 BGR/RGB)
    pub fn fill_bands(&mut self, scroll: usize) {
        const BANDS: [u16; 4] = [0xF800, 0x07E0, 0x001F, 0xFFFF];
        for y in 0..HEIGHT {
            let c = BANDS[((y + scroll) % HEIGHT) * 4 / HEIGHT];
            let row = &mut self.framebuffer[y * WIDTH..(y + 1) * WIDTH];
            row.fill(c);
        }
    }
}
