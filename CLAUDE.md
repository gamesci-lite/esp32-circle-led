# CLAUDE.md

ESP32-S3 + 1.75" AMOLED（SH8601 QSPI）+ Slint 圆形触摸显示屏的 Rust 固件项目。AI 助手与贡献者请遵循。

## 铁律速查

1. **编译一律走远端构建机 ubuntu-hs-main，本机不装工具链** — 流程与 `m5stamp-c3u-sh1107` 共用 `tools/remote_build.sh`。
2. **`.cargo/config.toml` 不设 `ESP_IDF_TOOLS_INSTALL_DIR`** — 构建机走「已激活环境」模式（source export.sh）。
3. **不给 `[patch.crates-io]` 加 git 依赖** — 构建机访问不了 github。
4. **Slint 在 ESP32-S3 上保持 std + unsafe-single-threaded** — Slint 1.x 默认 features 已含 std。
5. **SH8601 走 QSPI 而非 SPI** — 引脚：SCK=10, D0=11, D1=13, D2=14, D3=9, CS=12, RST=15。
6. **Framebuffer 必须放 PSRAM** — 466×466×RGB565 ≈ 434 KB，远超 IRAM；用 `esp-alloc`。
7. **渲染层选 Slint 是效率优先决策** — 比 LVGL 效率高 20-50%；代价：GPLv3（学习/研究 OK，商业化需评估）。
8. **凭据不入库** — `wifi_secrets.toml` 在 `.gitignore`，真值只放本机 + 构建机；提交只看 `.example`。

## 硬件事实（不要凭印象改）

| 项 | 值 |
|---|---|
| 分辨率 | 466×466 RGB565 |
| 驱动 IC | SH8601（默认，R7 在 SH8601 位）|
| QSPI 时钟 | 80 MHz |
| 触摸 IC | GT911（I2C: SDA=39, SCL=40, INT=21, RST=18）|
| 参考价格 | ¥179（含 GPS 版） |

## 常用命令

```bash
# 改代码 → 出固件
bash tools/remote_build.sh

# 烧录 + 监视
espflash write-bin 0x0 flash-artifacts/esp32-circle-led-merged.bin
espflash monitor
```

## 提交约定

- `Cargo.lock` 必须入库（bin 工程可复现）。
- `flash-artifacts/`、`target/`、`.embuild/`、`wifi_secrets.toml` 不入库。
- commit message 用英文：`feat/fix/chore(scope): 一句话`，参考 `m5stamp-c3u-sh1107` 历史。
