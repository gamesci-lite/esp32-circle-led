# CLAUDE.md

ESP32-S3 + 1.75" AMOLED（**CO5300** QSPI）+ Slint 圆形触摸显示屏的 Rust 固件项目。AI 助手与贡献者请遵循。

## ✅ 屏已点亮 — C 路线 + Rust 路线双通 (2026-08-19)

**Rust 路线（主线，已点亮）**：`src/display/co5300.rs` 移植自 C v15，复用 vendored
`esp_lcd_co5300` v2.1.0 组件（`vendor/`，经 `[package.metadata.esp-idf-sys] extra_components`
接入 + bindgen 绑定）。实测：**36.5fps 全帧滚动**（fill 13.8ms / push 13.6ms，40MHz QSPI）[MEASURED]。
⚠️ 改 `extra_components` 元数据后 cargo 不会自动重跑 esp-idf-sys 构建脚本，必须
`rm -rf target/xtensa-esp32s3-espidf/release/build/esp-idf-sys-*` 强制重建，否则绑定是旧的。

**C 路线（备用参考）**：构建机 `/root/build/circle-c/`（main.cpp v15，旧版备份 bak13/bak14）。
实测 35.0fps（fill 15.0ms / push 13.6ms）。

**黑屏根因（13 轮误诊的教训）**：esp_lcd_co5300 组件**必须**设置
`co5300_vendor_config_t.flags.use_qspi_interface = 1`，否则命令不加 QSPI 协议头
（`0x02`=写命令 / `0x32`=写显存），CO5300 全当 NOP —— 症状「API 全部成功，屏纯黑」，
QSPI 无回读，日志无法暴露。

**运行期坑**：`esp-idf-hal` 0.46.2 仍用 legacy I2C 驱动（`i2c_driver_install`），与新
`i2c_master`（driver_ng）共存即运行期 abort：`E i2c: CONFLICT! driver_ng is not allowed
to be used with this old driver`。本项目 I2C（AXP2101）用 driver_ng，故已移除 esp-idf-hal。

## Rust 路线阻塞已解 (2026-08-19 实测)

~~Rust nightly 1.95.0-nightly (2026-04) + LLVM xtensa 后端 l32r bug~~ → **根因不是 nightly 版本，
是工具链选错**：`rust-toolchain.toml` 写 `channel = "nightly"` 用了 stock 工具链（上游 LLVM xtensa
后端有 l32r bug）。构建机其实装了 espup 的 **esp-rs 分叉工具链 `esp`**（带 xtensa 补丁，LLVM 21.1.3）。

**修复**：`rust-toolchain.toml` 改 `channel = "esp"`（本仓库已改）。
**实测**：`RUSTUP_TOOLCHAIN=esp cargo build --release` 全量构建+链接通过，0 l32r 错误，ELF 905KB 产物正常
（日志 `/root/build-esp-rust.log`，EXIT=0）。m5stamp-c3u-sh1107 参照项目能用 stock nightly 是因为
ESP32-C3 是 RISC-V（上游 LLVM 成熟）；S3 是 Xtensa，必须用 esp 分叉。

**推论**：因 l32r 而移除的 `esp-idf-svc` / Slint 可以按「恢复路径」逐步加回重试。

## 铁律速查

1. **编译一律走远端构建机 ubuntu-hs-main** — 流程与 `m5stamp-c3u-sh1107` 共用 `tools/remote_build.sh`。
2. **`.cargo/config.toml` 不设 `ESP_IDF_TOOLS_INSTALL_DIR`** — 构建机走「已激活环境」模式。
3. **不给 `[patch.crates-io]` 加 git 依赖** — 构建机访问不了 github。
4. **CO5300 QSPI 必须设 `use_qspi_interface=1`**（见上方「屏已点亮」）— 缺它 = 全部命令变 NOP 的假成功。
5. **Framebuffer 必须放 PSRAM** — 466×466×RGB565 ≈ 434 KB，远超 IRAM；用 `heap_caps_malloc(MALLOC_CAP_SPIRAM)`。SPI 用 DMA 时 `max_transfer_sz` 必须分块（如 466×20×2），写 434KB 会让驱动找等量内部 RAM 反弹缓冲 → `NO_MEM`。
6. **凭据不入库** — `wifi_secrets.toml` 在 `.gitignore`，真值只放本机 + 构建机。
7. **构建机有 HTTP 代理**，rustup/curl 直连需 `unset http_proxy https_proxy HTTP_PROXY HTTPS_PROXY ALL_PROXY all_proxy`。
8. **Slint + memmap2 暂不集成** — memmap2 0.9.11 依赖 POSIX libc 常量，esp-idf musl 不兼容。等下一轮单独迭代。
9. **QSPI 只写无回读** —「API 返回成功」≠「控制器收到」。排查显示问题时优先怀疑协议头/模式，别信日志全绿。

## 硬件事实（已实测验证 2026-08-19，不要凭印象改）

| 项 | 值 |
|---|---|
| 板子 | Waveshare **ESP32-S3-Touch-AMOLED-1.75C**（触屏版）|
| 分辨率 | 466×466 RGB565，**列偏移 x_gap=6** |
| 驱动 IC | **CO5300**（⚠️ 不是 SH8601！wiki 的 SH8601 是基础版，1.75C 触屏版是 CO5300）|
| QSPI 引脚 | SDIO0-3 = **4/5/6/7**，SCLK=**38**，CS=**12**，RST=**1**，TE=**13** |
| PMU | AXP2101 @ I2C **0x34**（SDA=15, SCL=14），**显示 rail = ALDO3**（0x90 bit2 使能，0x94 电压，0x1C=3.3V）|
| QSPI 时钟 | 80 MHz（理论）/ 10 MHz（C 路线已验证）/ 40 MHz（Rust 路线当前单线）|
| 触摸 IC | GT911（I2C: SDA=39, SCL=40, INT=21, RST=18）— 待实测确认 |

## 常用命令

```bash
# 改代码 → 出固件
bash tools/remote_build.sh

# 烧录 + 监视
espflash write-bin 0x0 flash-artifacts/esp32-circle-led-merged.bin --port /dev/cu.usbmodem101
espflash monitor --port /dev/cu.usbmodem101
```

## 提交约定

- `Cargo.lock` 必须入库（bin 工程可复现）。
- `flash-artifacts/`、`target/`、`.embuild/`、`wifi_secrets.toml` 不入库。
- commit message 用英文：`feat/fix/chore(scope): 一句话`。

## 未来 l32r 修复后的恢复路径

1. `cargo add slint` 恢复 Slint（带 `default-features=false, features=["std", "unsafe-single-threaded"]`）
2. `cargo add esp-idf-svc` 恢复
3. 把 `src/ui/app.slint` + `src/ui/platform.rs` 从 git 历史 `13dfb2a` 恢复
4. main.rs 接 Slint 事件循环
5. 编译 + 烧录 + 验证完整 UI 复刻 Squareline 桌面
