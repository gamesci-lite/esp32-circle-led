//! 主题与布局常量 (466x466 圆屏, 复刻出厂 Squareline 桌面照片)

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;

pub const SCREEN_W: i32 = 466;
pub const SCREEN_H: i32 = 466;

// 颜色
pub const BG: Rgb565 = Rgb565::BLACK;
pub const FG: Rgb565 = Rgb565::WHITE;
pub const GRAY: Rgb565 = Rgb565::new(10, 20, 10);
pub const RED: Rgb565 = Rgb565::new(26, 8, 2);        // Squareline logo 红
pub const CALC_BG: Rgb565 = Rgb565::new(11, 22, 27);  // 计算器蓝紫 #5B5BD6
pub const DRAW_BG: Rgb565 = Rgb565::new(31, 22, 0);   // 画板橙 #FF8C00
pub const CHAT_BG: Rgb565 = Rgb565::new(3, 30, 31);   // AIChats 蓝 #1E90FF
pub const BATTERY_GREEN: Rgb565 = Rgb565::new(4, 40, 4);

// 状态栏 (顶部居中簇: [wifi][电池][时间])
pub const STATUS_Y: i32 = 44;
pub const WIFI_POS: (i32, i32) = (168, 52);      // wifi 扇形锚点 (底部中心)
pub const BATTERY_POS: (i32, i32) = (200, 40);   // 电池左上角
pub const TIME_POS: (i32, i32) = (234, 62);      // 时间 baseline 起点

// 应用网格: 图标 100x100 圆角, 2x2, 标签在图标下
pub const ICON_SIZE: i32 = 100;
pub const ICON_RADIUS: i32 = 22;
pub const GRID_X0: i32 = (SCREEN_W - (ICON_SIZE * 2 + 60)) / 2;  // 103
pub const GRID_Y0: i32 = 118;
pub const GRID_GAP_X: i32 = 60;
pub const GRID_GAP_Y: i32 = 152;   // 图标 100 + 标签 ~20 + 余量
pub const LABEL_DY: i32 = 124;     // 标签 baseline 相对图标顶

// 底部页点
pub const DOTS_Y: i32 = 428;
