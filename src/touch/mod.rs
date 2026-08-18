//! GT911 电容触摸驱动（I2C）.
//!
//! 引脚：SDA=39, SCL=40, INT=21, RST=18
//! I2C 地址：0x5D（INT/INT 拉低时）或 0x14（INT 高时）
//!
//! 备选 crate：
//!   - https://github.com/abayomi185/gt911
//!   - https://github.com/jessebraham/tt21100
//!   - https://github.com/DeppLearning/gt911
//!
//! TODO：
//!   [1] 选 crate（待评估: abayomi185/gt911 vs DeppLearning/gt911）
//!   [2] I2C init + reset 序列
//!   [3] 多点触摸读取（最多 5 点）
//!   [4] 与 Slint 事件循环对接（translate touch → slint::platform::WindowEvent::PointerEvent）

#![allow(dead_code)]
