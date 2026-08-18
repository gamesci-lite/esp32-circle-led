#!/usr/bin/env bash
# 远端构建脚本 — ubuntu-hs-main
# 用法: bash tools/remote_build.sh
#
# 与 m5stamp-c3u-sh1107 共用构建机 + 同样的环境 (ESP-IDF v5.5, xtensa toolchain esp-14.2)
# 但 target 不同: m5stamp 是 ESP32-C3 (RISC-V), 本项目是 ESP32-S3 (Xtensa)

set -euo pipefail

REMOTE_HOST="ubuntu-hs-main"
REMOTE_DIR="/root/build/esp32-circle-led"
LOCAL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "→ rsync ${LOCAL_DIR} → ${REMOTE_HOST}:${REMOTE_DIR}"
rsync -az --delete \
    --exclude target \
    --exclude .embuild \
    --exclude .git \
    --exclude flash-artifacts \
    --exclude wifi_secrets.toml \
    --exclude Cargo.lock.bak \
    "${LOCAL_DIR}/" "${REMOTE_HOST}:${REMOTE_DIR}/"

echo "→ ssh ${REMOTE_HOST} bash /root/circle-build.sh"
ssh "${REMOTE_HOST}" "bash /root/circle-build.sh"

echo "→ scp merged.bin → flash-artifacts/"
mkdir -p "${LOCAL_DIR}/flash-artifacts"
scp "${REMOTE_HOST}:${REMOTE_DIR}/esp32-circle-led-merged.bin" \
    "${LOCAL_DIR}/flash-artifacts/"

echo "✓ build done: ${LOCAL_DIR}/flash-artifacts/esp32-circle-led-merged.bin"
