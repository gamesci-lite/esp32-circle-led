//! 桌面页 — 状态栏 + 2x2 应用网格 + 页码点 (复刻出厂 Squareline 桌面)

use embedded_graphics::prelude::*;

use super::apps;
use super::theme;
use super::widgets;
use super::icons;
use super::FbTarget;

/// 渲染桌面第 `page` 页到 framebuffer
pub fn render(fb: &mut [u16], page: usize) {
    let mut t = FbTarget::new(fb, theme::SCREEN_W, theme::SCREEN_H);
    let _ = t.clear(theme::BG);

    // 顶部状态栏: WiFi + 电池(70% 静态, M5 接真实数据) + 时间(静态, M5 SNTP)
    widgets::status_bar(&mut t, "00:00", 70);

    // 2x2 应用网格 (feature 驱动, 见 apps::ENTRIES)
    let start = page * apps::PAGE_SIZE;
    for (slot, entry) in apps::ENTRIES.iter().skip(start).take(apps::PAGE_SIZE).enumerate() {
        let col = slot as i32 % 2;
        let row = slot as i32 / 2;
        let pos = Point::new(
            theme::GRID_X0 + col * (theme::ICON_SIZE + theme::GRID_GAP_X),
            theme::GRID_Y0 + row * theme::GRID_GAP_Y,
        );
        widgets::icon_button(&mut t, pos, entry.bg);
        icons::draw_icon(&mut t, entry.id, pos);
        widgets::icon_label(&mut t, entry.name, pos);
    }

    // 底部页码点
    widgets::page_dots(&mut t, apps::page_count(), page);
}
