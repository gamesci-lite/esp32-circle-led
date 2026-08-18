//! Slint UI 层 — 复刻 Squareline 桌面风格.
//!
//! 选型理由:
//! - 比 LVGL 效率高 20-50% (声明式 + 编译期优化 + dirty region)
//! - Rust 原生绑定, 与 m5stamp-c3u-sh1107 同一栈
//! - 复杂控件支持齐备
//! - License: GPLv3 — 开源/学习 OK

pub mod platform;

slint::include_slint!();
