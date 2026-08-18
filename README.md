# ESP32-微学-circle-led

**Waveshare 1.75" AMOLED 圆形触摸显示屏**（ESP32-S3）的 Rust 固件，用于 LVGL 级圆形 UI 验证。

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
| 屏驱动 | esp-idf-sys FFI + 手写 init 序列（来自乐鑫 [esp_lcd_sh8601](https://github.com/espressif/esp-iot-solution/tree/master/components/display/lcd/esp_lcd_sh8601)） | 单线 SPI 40MHz 验证接线；QSPI 80MHz 升级留 TODO |
| 触摸 | GT911 I2C | 备选 crate: `abayomi185/gt911`, `jessebraham/tt21100` |
| 构建 | Rust nightly + ESP-IDF v5.5 + 远端构建机 ubuntu-hs-main | 与 `m5stamp-c3u-sh1107` 共用 |

## 项目结构

```
.
├── Cargo.toml
├── build.rs              # wifi_secrets.toml → consts
├── sdkconfig.defaults     # PSRAM + 栈大小
├── rust-toolchain.toml
├── .cargo/config.toml     # 标准 esp-idf-svc 模板
├── src/
│   ├── main.rs           # 入口: PSRAM → SH8601 → test_pattern → 心跳
│   └── display/
│       ├── mod.rs
│       └── sh8601.rs     # SH8601 SPI 驱动 (esp-idf-sys FFI)
├── tools/
│   └── remote_build.sh   # 远端构建脚本 (ubuntu-hs-main)
└── wifi_secrets.toml.example
```

## 当前状态 (2026-08-18)

### ✅ 已完成
- SH8601 单线 SPI 驱动完整实现（init 序列 + draw_bitmap + framebuffer）
- PSRAM 分配 framebuffer（434KB）
- test_pattern 全屏刷红验证路径
- main.rs 完整入口（PSRAM → SH8601 → 心跳）
- 构建机脚本 + .cargo/config.toml + sdkconfig.defaults
- 代码推送：`https://github.com/gamesci-lite/esp32-circle-led.git`

### ⚠️ 阻塞问题（**等待 nightly 修复**）

**`l32r: misaligned literal target` 链接错误** —— Rust nightly 1.95.0-nightly (2026-04) + LLVM xtensa 后端的**已知 bug**，与具体 crate 无关：

```
esp_idf_hal.d3e2fb42a88aca4d-cgu.0:
  IsrReactor::wakers 函数引用 EspError::check_and_return 函数地址
  → 目标函数地址未对齐到 l32r 要求的 4 字节边界
```

**已尝试未生效的 workaround**：
- `lto = "off"` —— 链接错误未变
- `codegen-units = 1` —— 链接错误未变
- `opt-level = 1/0` —— 链接错误未变
- 移除 esp-idf-svc —— 错误从 esp-idf-hal 触发
- 各种 `target-feature` 禁用 —— xtensa 不识别这些 feature 名
- 链接器 `--ml32r-relax` —— 不存在
- 切换到 nightly-2025-11-15 —— `xtensa-esp32s3-espidf` 是 tier 3 target，需从源码编译（30-60 min）

### 🚧 下一阶段（屏点亮后）

1. **接 Slint UI 复刻 Squareline 桌面**（代码已在 git 历史里 `13dfb2a`）
2. **GT911 触摸 driver** + 触摸事件注入 Slint
3. **QSPI 升级**（单线 40MHz 全帧 ~87ms → QSPI 80MHz 可 60FPS）
4. **Waveshare 示例图形**复刻（圆形裁剪）

## 编译验证（远端构建机）

按 CLAUDE.md 铁律，构建走专用构建机 ubuntu-hs-main：

```bash
cd ~/data0/public_work/ding/ESP32-微学-circle-led  # 本机
bash tools/remote_build.sh
```

构建机会：
1. rsync 同步代码到 `/root/build/esp32-circle-led/`
2. ESP-IDF v5.5.3 激活（`source /root/esp/esp-idf/export.sh`）
3. `cargo build --release`
4. `espflash save-image --chip esp32s3 --merge ... esp32-circle-led-merged.bin`
5. scp 把 merged.bin 拉回 `flash-artifacts/`

**编译产物**：`flash-artifacts/esp32-circle-led-merged.bin`

## 烧录 + 监视（本机）

```bash
espflash write-bin 0x0 flash-artifacts/esp32-circle-led-merged.bin --port /dev/cu.usbmodem101
espflash monitor --port /dev/cu.usbmodem101
```

预期日志：
```
[circle] ===========================================
[circle]   esp32-circle-led booting
[circle]   硬件: Waveshare ESP32-S3 1.75" AMOLED
[circle] ===========================================
[circle] [A1] PSRAM allocator: 通过 heap_caps_malloc(MALLOC_CAP_SPIRAM) 分配 framebuffer
[circle] [A2-A3] SH8601 init + PSRAM framebuffer alloc done
[circle] [A4] test_pattern: 全屏刷红 (0xF800) ...
[sh8601] test_pattern(0xF800) pushed
[circle] [A4] test_pattern pushed — 屏应亮红色
```

→ 屏**应亮全屏红色**。

## 参考资料

- **官方 wiki**: <https://www.waveshare.com/wiki/ESP32-S3_1.75inch_AMOLED_Display>
- **官方 C 驱动**: <https://github.com/espressif/esp-iot-solution/tree/master/components/display/lcd/esp_lcd_sh8601>
- **官方示例**: <https://github.com/waveshare/ESP32-S3-1.75inch-AMOLED>
- **Slint 嵌入式**: <https://github.com/slint-ui/slint>
- **ESP32-S3 + Slint 实战**: <https://github.com/yaobo-lab/esp32-s3-box--with-slint>
