//! 应用入口注册表 — cargo feature 驱动
//!
//! 每个 app 一个 feature (Cargo.toml [features])，编译期裁剪。
//! 桌面网格按 ENTRIES 实际内容动态布局，页数 = ceil(ENTRIES/4)。

use embedded_graphics::pixelcolor::Rgb565;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppId {
    Squareline,
    Calculator,
    DrawPanel,
    AiChats,
}

pub struct AppEntry {
    pub id: AppId,
    pub name: &'static str,
    pub bg: Rgb565,
}

/// 已注册应用 (按 feature 编译期过滤, 保持照片里的顺序)
pub const ENTRIES: &[AppEntry] = &[
    #[cfg(feature = "app-squareline")]
    AppEntry { id: AppId::Squareline, name: "Squareline", bg: super::theme::FG },
    #[cfg(feature = "app-calculator")]
    AppEntry { id: AppId::Calculator, name: "Calculator", bg: super::theme::CALC_BG },
    #[cfg(feature = "app-drawpanel")]
    AppEntry { id: AppId::DrawPanel, name: "DrawPanel", bg: super::theme::DRAW_BG },
    #[cfg(feature = "app-aichats")]
    AppEntry { id: AppId::AiChats, name: "AIChats", bg: super::theme::CHAT_BG },
];

pub const PAGE_SIZE: usize = 4;

pub const fn page_count() -> usize {
    ENTRIES.len().div_ceil(PAGE_SIZE)
}
