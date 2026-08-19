//! 通用小部件: 状态栏 / 页码点 / 图标按钮

use embedded_graphics::geometry::Angle;
use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Arc, Circle, PrimitiveStyle, Rectangle, RoundedRectangle,
};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::Drawable;
use profont::PROFONT_24_POINT;

use super::theme;
use super::FbTarget;

/// 顶部状态栏: WiFi 扇形 + 电池 + 时间文本
pub fn status_bar(t: &mut FbTarget, time: &str, battery_pct: u8) {
    wifi_icon(t, Point::new(theme::WIFI_POS.0, theme::WIFI_POS.1));
    battery_icon(t, Point::new(theme::BATTERY_POS.0, theme::BATTERY_POS.1), battery_pct);
    let style = MonoTextStyle::new(&PROFONT_24_POINT, theme::FG);
    Text::new(time, Point::new(theme::TIME_POS.0, theme::TIME_POS.1), style)
        .draw(t)
        .ok();
}

/// WiFi 信号扇形 (3 弧 + 底点)
fn wifi_icon(t: &mut FbTarget, anchor: Point) {
    let style = PrimitiveStyle::with_stroke(theme::FG, 3);
    // 扇形开口朝上, 锚点在扇形底部中心
    for (i, d) in [18u32, 30, 42].iter().enumerate() {
        let top_left = anchor - Point::new(*d as i32 / 2, *d as i32 / 2);
        Arc::new(top_left, *d, Angle::from_degrees(225.0), Angle::from_degrees(90.0))
            .into_styled(style)
            .draw(t)
            .ok();
        let _ = i;
    }
    Circle::new(anchor - Point::new(3, 3), 6)
        .into_styled(PrimitiveStyle::with_fill(theme::FG))
        .draw(t)
        .ok();
}

/// 电池图标: 圆角外框 + 电平填充 + 凸头
fn battery_icon(t: &mut FbTarget, pos: Point, pct: u8) {
    let w = 26i32;
    let h = 14i32;
    // 外框
    RoundedRectangle::new(
        Rectangle::new(pos, Size::new(w as u32, h as u32)),
        embedded_graphics::primitives::CornerRadii::new(Size::new(3, 3)),
    )
    .into_styled(PrimitiveStyle::with_stroke(theme::FG, 1))
    .draw(t)
    .ok();
    // 电平 (内缩 2px)
    let inner_w = ((w - 4) as u32 * pct.min(100) as u32 / 100) as i32;
    if inner_w > 0 {
        Rectangle::new(pos + Point::new(2, 2), Size::new(inner_w as u32, (h - 4) as u32))
            .into_styled(PrimitiveStyle::with_fill(theme::BATTERY_GREEN))
            .draw(t)
            .ok();
    }
    // 凸头
    Rectangle::new(pos + Point::new(w + 1, 4), Size::new(3, 6))
        .into_styled(PrimitiveStyle::with_fill(theme::FG))
        .draw(t)
        .ok();
}

/// 图标按钮: 100x100 圆角底色块 (图标本体由 icons.rs 画)
pub fn icon_button(t: &mut FbTarget, pos: Point, bg: Rgb565) {
    RoundedRectangle::new(
        Rectangle::new(pos, Size::new(theme::ICON_SIZE as u32, theme::ICON_SIZE as u32)),
        embedded_graphics::primitives::CornerRadii::new(Size::new(
            theme::ICON_RADIUS as u32,
            theme::ICON_RADIUS as u32,
        )),
    )
    .into_styled(PrimitiveStyle::with_fill(bg))
    .draw(t)
    .ok();
}

/// 图标下标签 (居中)
pub fn icon_label(t: &mut FbTarget, name: &str, icon_pos: Point) {
    let style = MonoTextStyle::new(&FONT_10X20, theme::FG);
    Text::with_alignment(
        name,
        Point::new(
            icon_pos.x + theme::ICON_SIZE / 2,
            icon_pos.y + theme::LABEL_DY,
        ),
        style,
        Alignment::Center,
    )
    .draw(t)
    .ok();
}

/// 底部页码指示: 当前页 = 白色胶囊, 其余 = 灰点
pub fn page_dots(t: &mut FbTarget, pages: usize, current: usize) {
    if pages == 0 {
        return;
    }
    let capsule_w = 20i32;
    let dot_d = 8i32;
    let gap = 8i32;
    let total_w = capsule_w + (pages as i32 - 1) * (dot_d + gap);
    let mut x = (theme::SCREEN_W - total_w) / 2;
    for i in 0..pages {
        if i == current {
            RoundedRectangle::new(
                Rectangle::new(Point::new(x, theme::DOTS_Y), Size::new(capsule_w as u32, dot_d as u32)),
                embedded_graphics::primitives::CornerRadii::new(Size::new(4, 4)),
            )
            .into_styled(PrimitiveStyle::with_fill(theme::FG))
            .draw(t)
            .ok();
            x += capsule_w + gap;
        } else {
            Circle::new(Point::new(x, theme::DOTS_Y), dot_d as u32)
                .into_styled(PrimitiveStyle::with_fill(theme::GRAY))
                .draw(t)
                .ok();
            x += dot_d + gap;
        }
    }
}
