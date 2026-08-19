//! GUI 模块 — Squareline 风格圆形桌面 (M1)
//!
//! 渲染栈: embedded-graphics 0.8 直写 CO5300 PSRAM framebuffer (Rgb565)。
//! 布局复刻出厂 demo 照片: 顶部状态栏 (WiFi/电池/时间) + 2x2 应用网格 + 底部页点。
//! 应用入口由 cargo feature 驱动 (apps.rs)。

pub mod desktop;
mod apps;
mod icons;
mod theme;
mod widgets;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;

/// embedded-graphics DrawTarget — 桥接 CO5300 PSRAM framebuffer
pub struct FbTarget<'a> {
    fb: &'a mut [u16],
    w: i32,
    h: i32,
}

impl<'a> FbTarget<'a> {
    pub fn new(fb: &'a mut [u16], w: i32, h: i32) -> Self {
        Self { fb, w, h }
    }
}

impl DrawTarget for FbTarget<'_> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(p, c) in pixels {
            if p.x >= 0 && p.x < self.w && p.y >= 0 && p.y < self.h {
                self.fb[p.y as usize * self.w as usize + p.x as usize] = c.into_storage();
            }
        }
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.fb.fill(color.into_storage());
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let x0 = area.top_left.x.max(0);
        let y0 = area.top_left.y.max(0);
        let x1 = (area.top_left.x + area.size.width as i32).min(self.w);
        let y1 = (area.top_left.y + area.size.height as i32).min(self.h);
        let v = color.into_storage();
        for y in y0..y1 {
            let row = &mut self.fb[y as usize * self.w as usize + x0 as usize..y as usize * self.w as usize + x1 as usize];
            row.fill(v);
        }
        Ok(())
    }
}

impl OriginDimensions for FbTarget<'_> {
    fn size(&self) -> Size {
        Size::new(self.w as u32, self.h as u32)
    }
}
