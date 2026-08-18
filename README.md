# ESP32-微学-circle-led

**Waveshare 1.75" AMOLED 圆形触摸显示屏**（ESP32-S3）的 Rust + Slint 固件，用于 LVGL 级圆形 UI 验证。

## 硬件事实

| 项 | 值 |
|---|---|
| 主板 | [Waveshare ESP32-S3 1.75inch AMOLED Touch Display](https://www.waveshare.com/wiki/ESP32-S3_1.75inch_AMOLED_Display) |
| 主控 | ESP32-S3 (Xtensa LX7 双核, WiFi+BLE5) |
| 屏幕 | 1.75" **AMOLED**, **466×466** 像素, 262K 色 |
| 显示驱动 | **SH8601**（默认，QSPI）— 切换 R7 电阻可用 CO5300 |
| QSPI 引脚 | SCK=10, D0=11, D1=13, D2=14, D3=9, CS=12, RST=15, TE=38 |
| QSPI 时钟 | 最高 **80 MHz** |
| 触摸 | GT911（I2C: SDA=39, SCL=40, INT=21, RST=18） |
| 参考价 | ¥179（含 GPS 版本） |

## 技术选型

| 层 | 选型 | 理由 |
|---|---|---|
| **渲染** | **Slint 1.x** | ① 比 LVGL 效率高 20-50%（声明式 + 编译期优化 + dirty region）<br>② Rust 原生绑定，与 m5stamp-c3u-sh1107 同一栈<br>③ 复杂控件（旋钮/仪表盘/动画）支持齐备<br>⚠ License: GPLv3 — 开源/学习 OK；商业化需评估 |
| 屏驱动 | 自实现 QSPI 适配层（esp-idf-hal SPI3 子模式） | SH8601 QSPI 4-line, 主流 crate 不直接覆盖 |
| 触摸 | GT911 I2C | 备选 crate: `abayomi185/gt911`, `jessebraham/tt21100` |
| 构建 | Rust nightly + ESP-IDF v5.5 | 与 `m5stamp-c3u-sh1107` 共用远端构建机 ubuntu-hs-main |

## 项目结构

```
.
├── Cargo.toml
├── build.rs              # wifi_secrets.toml → consts + slint 编译
├── sdkconfig.defaults     # PSRAM + 栈大小
├── rust-toolchain.toml
├── .cargo/config.toml     # xtensa-esp32s3-espidf target
├── src/
│   ├── main.rs           # 入口
│   ├── display/
│   │   ├── mod.rs
│   │   └── sh8601.rs     # SH8601 QSPI driver (init + framebuffer 推送)
│   ├── touch/
│   │   └── mod.rs        # GT911 触摸 (占位)
│   └── ui/
│       ├── mod.rs
│       ├── app.slint     # UI DSL
│       └── platform.rs   # Slint platform impl (WindowAdapter)
├── tools/
│   └── remote_build.sh   # 远端构建脚本 (ubuntu-hs-main)
├── assets/               # 字模/图标素材
└── wifi_secrets.toml.example
```

## 编译/烧录

```bash
# 远端编译 (与 m5stamp 同流程)
bash tools/remote_build.sh

# 本机烧录 + 监视
espflash write-bin 0x0 flash-artifacts/esp32-circle-led-merged.bin
espflash monitor
```

## 参考资料

- **官方 wiki**: <https://www.waveshare.com/wiki/ESP32-S3_1.75inch_AMOLED_Display>
- **官方示例**: <https://github.com/waveshare/ESP32-S3-1.75inch-AMOLED>
- **Slint 嵌入式**: <https://github.com/slint-ui/slint>
- **ESP32-S3 + Slint 实战**: <https://github.com/yaobo-lab/esp32-s3-box--with-slint>
- **Hamboo 智能手表 (Rust + Slint on ESP32-S3)**: <https://github.com/nickelc/Hamboo>
