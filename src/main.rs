//! 入口: AXP2101 上电 → CO5300 QSPI init → 四色带滚动 + 实测帧率.
//!
//! 移植自 C 固件 v15 (已实测: 35fps 全帧, 40MHz QSPI).
//! 编译: bash tools/remote_build.sh (构建机 ubuntu-hs-main, esp 工具链).

use esp_idf_sys::{
    esp_log_level_set, esp_log_write, esp_timer_get_time,
    esp_log_level_t_ESP_LOG_INFO, esp_log_level_t_ESP_LOG_WARN,
};

mod display;

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
    log_info("circle", "  esp32-circle-led booting (Rust CO5300 点亮版)");
    log_info("circle", "  硬件: Waveshare ESP32-S3-Touch-AMOLED-1.75C");
    log_info("circle", "  驱动: CO5300 QSPI 40MHz (esp_lcd_co5300 v2.1.0 vendor)");
    log_info("circle", "===========================================");

    let mut lcd = display::co5300::Co5300::init()?;
    log_info("circle", "[1] Co5300 init done, 进入全帧滚动循环");

    let mut scroll = 0usize;
    let mut frames = 0u32;
    let mut fill_us_total = 0i64;
    let mut push_us_total = 0i64;
    let mut stat_start = unsafe { esp_timer_get_time() };
    loop {
        let t0 = unsafe { esp_timer_get_time() };
        lcd.fill_bands(scroll);
        let t1 = unsafe { esp_timer_get_time() };
        lcd.push()?;
        let t2 = unsafe { esp_timer_get_time() };

        fill_us_total += t1 - t0;
        push_us_total += t2 - t1;
        frames += 1;
        scroll = (scroll + 2) % display::co5300::HEIGHT;

        if t2 - stat_start >= 5_000_000 {
            let fps = frames as f32 * 1_000_000.0 / (t2 - stat_start) as f32;
            let fill_ms = fill_us_total as f32 / 1000.0 / frames as f32;
            let push_ms = push_us_total as f32 / 1000.0 / frames as f32;
            // 无 alloc 简易格式化 (老版本 std::format! 触发 l32r 重定位的保守替代, 保留)
            let mut buf = [0u8; 96];
            let s = format_stats(&mut buf, fps, fill_ms, push_ms, frames);
            log_info("circle", s);
            frames = 0;
            fill_us_total = 0;
            push_us_total = 0;
            stat_start = t2;
        }
    }
}

/// 无 alloc 的 fps 统计行格式化
fn format_stats<'a>(buf: &'a mut [u8], fps: f32, fill_ms: f32, push_ms: f32, frames: u32) -> &'a str {
    use core::fmt::Write;
    struct BufWriter<'a> { buf: &'a mut [u8], pos: usize }
    impl<'a> Write for BufWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let b = s.as_bytes();
            let n = b.len().min(self.buf.len().saturating_sub(self.pos + 1));
            self.buf[self.pos..self.pos + n].copy_from_slice(&b[..n]);
            self.pos += n;
            Ok(())
        }
    }
    let mut w = BufWriter { buf, pos: 0 };
    let _ = write!(w, "fps={:.1} fill={:.1}ms push={:.1}ms (frames={})", fps, fill_ms, push_ms, frames);
    w.buf[w.pos] = 0;
    core::str::from_utf8(&w.buf[..w.pos]).unwrap_or("fmt err")
}
