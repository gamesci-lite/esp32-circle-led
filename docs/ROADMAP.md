# Roadmap — ESP32-微学-circle-led

目标：Waveshare ESP32-S3-Touch-AMOLED-1.75C（CO5300 466×466 圆屏）做成一块
「微学」圆形智能桌面 —— 状态栏 + 应用网格 + 触摸交互，功能入口 **cargo feature 驱动**。

性能/可用性结论一律标 [MEASURED]（本机实测）/ [HYPOTHETICAL]（推断，落地前需验证）。

---

## M0 ✅ 点亮底座 (2026-08-19 完成)

| 项 | 状态 |
|---|---|
| CO5300 QSPI 驱动（C + Rust 双路线） | ✅ [MEASURED] Rust 36.5fps 全帧（fill 13.8ms / push 13.6ms @40MHz） |
| esp-rs 分叉工具链（l32r 规避） | ✅ `rust-toolchain.toml` = `esp` |
| AXP2101 PMU（ALDO3 显示 rail，回读验证） | ✅ driver_ng I2C |
| vendored esp_lcd_co5300 v2.1.0 + bindgen | ✅ `extra_components` |
| esp-idf-hal 移除（legacy I2C 冲突） | ✅ |

## M1 ✅ GUI 桌面还原 (2026-08-19 完成)

复刻出厂 Squareline 桌面（照片存档）：

- 顶部状态栏：WiFi 信号图标 + 电池图标 + 时间（先静态，M5 接真实数据）
- 2×2 应用网格：Squareline / Calculator / DrawPanel / AIChats（圆角图标 + 标签）
- 底部页码指示器（胶囊 + 圆点，随实际页数）
- 渲染：embedded-graphics 0.8（零系统依赖，m5stamp 项目已验证同工具链可编）
- **入口 feature 驱动**（见下「Feature 架构」）

验收：✅ 烧录后屏上显示完整桌面，布局与参照照片一致（图标为程序化近似）。

## M2 OTA 无线灌录（下轮，摆脱 USB 线）

现状：**无 OTA** —— 分区表单 factory，固件头 4MB vs 物理 flash 32MB。

1. 分区表 → `otadata + ota_0 + ota_1`（每槽 4MB，32MB flash 余量充足）
2. sdkconfig：`CONFIG_ESPTOOLPY_FLASHSIZE_32MB`
3. WiFi 打通（二选一，倾向 b）：
   a. 回 esp-idf-svc —— 需先把 AXP2101 驱动改 legacy I2C 与 hal 共存
   b. 纯 esp-idf-sys 裸 `esp_wifi` FFI（不引 hal/svc，保持依赖干净）[HYPOTHETICAL 工作量中]
4. `esp_https_ota`（FFI 或 esp-ota crate）+ 固件源（构建机起 http server / R2）
5. 防砖：`CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE` + 应用内自诊断后 mark valid

验收：最后一次插线烧 M2 固件，之后全部 OTA。构建机产出 app bin → 推送固件源 → 设备拉取升级。

## M3 触摸 GT911

- I2C 触摸驱动（driver_ng；GT911: SDA=39/SCL=40/INT=21/RST=18，待实测确认 [HYPOTHETICAL]）
- 触摸事件队列 → GUI 事件循环
- 点按进 app 页 / 返回桌面；左右滑切页
- DrawPanel 前置依赖

## M4 应用实体化（feature 驱动）

| App | 内容 | 依赖 |
|---|---|---|
| Calculator | 真计算器（触摸键盘） | M3 |
| DrawPanel | 触摸画板（RGB565 直写 fb） | M3 |
| AIChats | WiFi + LLM API 对话 | M2(WiFi), M3 |
| Squareline | 关于/信息页（致敬出厂 demo） | — |

## M5 状态栏真实化

- WiFi 信号强度 ↔ 图标（wifi event → RSSI）
- 电池电量 ← AXP2101 电量/电压寄存器（寄存器表已在手）
- 时间 ← SNTP 校时

## M6 性能与显示质量

- QSPI 40→80MHz [HYPOTHETICAL：信号完整性需实测，FPC 较长可能不稳]
- TE（GPIO13）同步防撕裂（esp_lcd vsync）
- fill 优化：DMA memset / 局部刷新 / 双缓冲行带
- 圆屏边缘抗锯齿

## M7 Slint 迁移（endgame，复刻 Squareline 的正式路径）

- 解 fontdb → memmap2 → espidf 缺 mmap 常量链（vendor 本地化 patch；CLAUDE.md 旧述「musl 不兼容」有误，实为 newlib）
- 复活 git 历史 `13dfb2a` 的 `src/ui/app.slint` + `platform.rs`
- SoftwareRenderer 直写 PSRAM fb（渲染管线与 M1 自绘 GUI 相同，可对照）
- 决策点：若自绘 GUI 已满足需求，Slint 可长期搁置 [HYPOTHETICAL]

---

## Feature 架构（入口 feature 驱动）

```toml
[features]
default = ["all-apps"]
all-apps = ["app-squareline", "app-calculator", "app-drawpanel", "app-aichats"]
app-squareline = []   # 关于/信息页
app-calculator = []   # 计算器 (M4)
app-drawpanel  = []   # 触摸画板 (M3 前置)
app-aichats    = []   # AI 对话 (M2 WiFi 前置)
```

- 每个 app 一个 cargo feature：`src/gui/apps.rs` 用 `#[cfg(feature = "app-xxx")]` 注册入口
- 桌面网格按启用 feature 动态生成（图标 + 标签 + 页数自适应）
- 裁剪固件：`--no-default-features --features app-calculator` 只带一个 app
- 后续系统级 feature 预留：`ota`、`touch`、`wifi`、`slint-ui`

## 依赖图

```
M0 点亮 ✅ → M1 GUI → M3 触摸 → M4 应用
         ↘ M2 OTA/WiFi ──────────↗ (AIChats 需 WiFi)
M5 状态栏真实化 ← M2(WiFi 时间/RSSI)
M6 性能（随时插空）
M7 Slint（独立支线，不阻塞主线）
```
