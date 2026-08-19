//! 程序化图标绘制 — 近似出厂 demo 的四个应用图标 (100x100 内绘制)

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, Line, PrimitiveStyle, Rectangle, Triangle,
};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::Drawable;

use super::apps::AppId;
use super::theme;
use super::FbTarget;

/// 在图标底色块内绘制对应图标本体 (pos = 图标左上角, 100x100)
pub fn draw_icon(t: &mut FbTarget, id: AppId, pos: Point) {
    match id {
        AppId::Squareline => squareline(t, pos),
        AppId::Calculator => calculator(t, pos),
        AppId::DrawPanel => drawpanel(t, pos),
        AppId::AiChats => aichats(t, pos),
    }
}

/// Squareline: 白底 + 红色层叠菱形 logo (近似)
fn squareline(t: &mut FbTarget, pos: Point) {
    let c = pos + Point::new(50, 50);
    let style = PrimitiveStyle::with_stroke(theme::RED, 4);
    // 外层菱形 (四点连线)
    let top = c - Point::new(0, 26);
    let right = c + Point::new(22, 0);
    let bottom = c + Point::new(0, 26);
    let left = c - Point::new(22, 0);
    for (a, b) in [(top, right), (right, bottom), (bottom, left), (left, top)] {
        Line::new(a, b).into_styled(style).draw(t).ok();
    }
    // 内层小菱形填充
    Triangle::new(c - Point::new(12, 0), c + Point::new(12, 0), c - Point::new(0, 14))
        .into_styled(PrimitiveStyle::with_fill(theme::RED))
        .draw(t)
        .ok();
    Triangle::new(c - Point::new(12, 0), c + Point::new(12, 0), c + Point::new(0, 14))
        .into_styled(PrimitiveStyle::with_fill(theme::RED))
        .draw(t)
        .ok();
}

/// Calculator: 蓝紫底 + 2x2 运算格 (+ - x =)
fn calculator(t: &mut FbTarget, pos: Point) {
    let grid_pos = pos + Point::new(24, 20);
    let cell = 26i32;
    // 白色面板
    Rectangle::new(grid_pos, Size::new((cell * 2) as u32, (cell * 2) as u32))
        .into_styled(PrimitiveStyle::with_stroke(theme::FG, 2))
        .draw(t)
        .ok();
    // 十字分隔
    Line::new(grid_pos + Point::new(cell, 0), grid_pos + Point::new(cell, cell * 2))
        .into_styled(PrimitiveStyle::with_stroke(theme::FG, 2))
        .draw(t)
        .ok();
    Line::new(grid_pos + Point::new(0, cell), grid_pos + Point::new(cell * 2, cell))
        .into_styled(PrimitiveStyle::with_stroke(theme::FG, 2))
        .draw(t)
        .ok();
    // 四格符号
    let style = MonoTextStyle::new(&FONT_10X20, theme::FG);
    let glyphs = ["+", "-", "x", "="];
    for (i, g) in glyphs.iter().enumerate() {
        let cx = grid_pos.x + (i as i32 % 2) * cell + cell / 2;
        let cy = grid_pos.y + (i as i32 / 2) * cell + cell / 2 + 5;
        Text::with_alignment(g, Point::new(cx, cy), style, Alignment::Center)
            .draw(t)
            .ok();
    }
}

/// DrawPanel: 橙底 + 白色画笔 (笔杆 + 笔头 + 颜料点)
fn drawpanel(t: &mut FbTarget, pos: Point) {
    // 笔杆 (右上 → 左下 斜线)
    Line::new(pos + Point::new(66, 26), pos + Point::new(44, 52))
        .into_styled(PrimitiveStyle::with_stroke(theme::FG, 7))
        .draw(t)
        .ok();
    // 笔头 (三角)
    Triangle::new(
        pos + Point::new(44, 52),
        pos + Point::new(52, 56),
        pos + Point::new(38, 66),
    )
    .into_styled(PrimitiveStyle::with_fill(theme::FG))
    .draw(t)
    .ok();
    // 颜料点 (青绿)
    Circle::new(pos + Point::new(30, 68), 10)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::new(0, 40, 24)))
        .draw(t)
        .ok();
}

/// AIChats: 蓝底 + 白色机器人 (圆头 + 天线 + 双眼 + 肩)
fn aichats(t: &mut FbTarget, pos: Point) {
    let c = pos + Point::new(50, 46);
    // 头 (圆角矩形近似: 圆+矩形)
    Circle::new(c - Point::new(20, 20), 40)
        .into_styled(PrimitiveStyle::with_stroke(theme::FG, 3))
        .draw(t)
        .ok();
    // 天线
    Line::new(c - Point::new(0, 20), c - Point::new(0, 30))
        .into_styled(PrimitiveStyle::with_stroke(theme::FG, 3))
        .draw(t)
        .ok();
    Circle::new(c - Point::new(3, 36), 6)
        .into_styled(PrimitiveStyle::with_fill(theme::FG))
        .draw(t)
        .ok();
    // 双眼
    Circle::new(c - Point::new(12, 6), 7)
        .into_styled(PrimitiveStyle::with_fill(theme::FG))
        .draw(t)
        .ok();
    Circle::new(c + Point::new(5, 6), 7)
        .into_styled(PrimitiveStyle::with_fill(theme::FG))
        .draw(t)
        .ok();
    // 肩 (底部短粗线近似)
    Line::new(c - Point::new(18, 34) + Point::new(0, 8), c + Point::new(18, 42))
        .into_styled(PrimitiveStyle::with_stroke(theme::FG, 5))
        .draw(t)
        .ok();
}
