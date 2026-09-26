#!/usr/bin/env bash
set -euo pipefail
project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
action="${1:-build}"
export CARGO_HOME="$project_root/tmp/cargo-home"
export RUSTUP_HOME="$project_root/tmp/esp-rustup"
export TMPDIR="$project_root/tmp"
source "$project_root/tmp/esp-tools/export-esp.sh"
firmware="$project_root/target/esp32-player/xtensa-esp32s3-none-elf/release/stagemaster-esp32-probe"
case "$action" in
  build)
    cd "$project_root/apps/esp32-player"
    cargo +esp build --release --locked --offline
    ;;
  check)
    cd "$project_root/apps/esp32-player"
    cargo +esp clippy --release --locked --offline -- -D warnings
    ;;
  size)
    xtensa-esp32s3-elf-size -A "$firmware"
    ;;
  flash)
    : "${2:?明确指定开发板串口，例如 /dev/cu.usbmodem2101}"
    "$project_root/tmp/esp-tools/espflash" flash --port "$2" --chip esp32s3 --flash-size 16mb --non-interactive "$firmware"
    ;;
  monitor)
    : "${2:?明确指定开发板串口}"
    "$project_root/tmp/esp-tools/espflash" monitor --port "$2" --non-interactive --no-reset --skip-update-check --elf "$firmware"
    ;;
  *) printf '用法：%s {build|check|size|flash 串口|monitor 串口}\n' "$0" >&2; exit 2 ;;
esac
