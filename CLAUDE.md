# CLAUDE.md

ESP32-S3 + 1.75" AMOLED（SH8601 QSPI）+ Slint 圆形触摸显示屏的 Rust 固件项目。AI 助手与贡献者请遵循。

## 当前阻塞状态 (2026-08-18)

**Rust nightly 1.95.0-nightly (2026-04) + LLVM xtensa 后端 l32r bug** 阻塞链接阶段：

```
dangerous relocation: l32r: misaligned literal target
```

错误源头是 LLVM xtensa 后端的 codegen 问题 —— 函数 address 没对齐到 l32r 要求的 4 字节。**与项目代码无关**。

详见 README.md「⚠️ 阻塞问题」段。

## 铁律速查

1. **编译一律走远端构建机 ubuntu-hs-main** — 流程与 `m5stamp-c3u-sh1107` 共用 `tools/remote_build.sh`。
2. **`.cargo/config.toml` 不设 `ESP_IDF_TOOLS_INSTALL_DIR`** — 构建机走「已激活环境」模式。
3. **不给 `[patch.crates-io]` 加 git 依赖** — 构建机访问不了 github。
4. **SH8601 走单线 SPI 40MHz**（当前）— QSPI 80MHz 升级留 TODO。
5. **Framebuffer 必须放 PSRAM** — 466×466×RGB565 ≈ 434 KB，远超 IRAM；用 `heap_caps_malloc(MALLOC_CAP_SPIRAM)`。
6. **凭据不入库** — `wifi_secrets.toml` 在 `.gitignore`，真值只放本机 + 构建机。
7. **构建机有 HTTP 代理**，rustup/curl 直连需 `unset http_proxy https_proxy HTTP_PROXY HTTPS_PROXY ALL_PROXY all_proxy`。
8. **Slint + memmap2 暂不集成** — memmap2 0.9.11 依赖 POSIX libc 常量，esp-idf musl 不兼容。等下一轮单独迭代。

## 硬件事实（不要凭印象改）

| 项 | 值 |
|---|---|
| 分辨率 | 466×466 RGB565 |
| 驱动 IC | SH8601（默认，R7 在 SH8601 位）|
| QSPI 时钟 | 80 MHz（理论）/ 40 MHz（当前单线）|
| 触摸 IC | GT911（I2C: SDA=39, SCL=40, INT=21, RST=18）|

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
