#!/usr/bin/env bash
# 远端构建脚本 — 与 m5stamp-c3u-sh1107 共用 ubuntu-hs-main
# 用法: bash tools/remote_build.sh

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
    "${LOCAL_DIR}/" "${REMOTE_HOST}:${REMOTE_DIR}/"

echo "→ ssh ${REMOTE_HOST} cargo build --release"
ssh "${REMOTE_HOST}" "cd ${REMOTE_DIR} && source /opt/esp/esp-idf/export.sh && cargo build --release"

echo "→ scp merged.bin → flash-artifacts/"
mkdir -p "${LOCAL_DIR}/flash-artifacts"
scp "${REMOTE_HOST}:${REMOTE_DIR}/target/xtensa-esp32s3-espidf/release/esp32-circle-led-merged.bin" \
    "${LOCAL_DIR}/flash-artifacts/"

echo "✓ build done: ${LOCAL_DIR}/flash-artifacts/esp32-circle-led-merged.bin"
