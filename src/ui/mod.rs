//! Slint UI 层。
//!
//! 选型理由：
//! - 比 LVGL 效率高 20-50%（声明式 + 编译期优化 + dirty region）
//! - Rust 原生绑定, 与 m5stamp-c3u-sh1107 同一栈
//! - 复杂控件（旋钮 / 仪表盘 / 动画）支持齐备
//! - License: GPLv3 — 开源/学习 OK
//!
//! 文件：
//! - `app.slint`    : UI DSL（build.rs 编译生成 rust 代码）
//! - `platform.rs`  : 自实现 `slint::platform::Platform` 把渲染绑到 SH8601 framebuffer

pub mod platform;

slint::include_slint!();
