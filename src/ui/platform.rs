//! Slint platform impl — 把 Slint 渲染绑定到 SH8601 framebuffer.
#![allow(dead_code)]

/// 占位: 实际实现时替换为自定义 WindowAdapter
pub struct CircleLedPlatform;

pub fn install() -> Result<(), slint::PlatformError> {
    // TODO: slint::platform::set_platform(Box::new(CircleLedPlatform))
    log::info!("ui::platform::install 占位 — Slint 暂未启用");
    Ok(())
}
