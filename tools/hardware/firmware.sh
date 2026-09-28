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
  secure-gatt-build|secure-gatt-check)
    export CARGO_TARGET_DIR="$project_root/target/esp32-secure-gatt"
    cd "$project_root/apps/esp32-player"
    if [[ "$action" == *-build ]]; then
      cargo +esp build --release --features secure-gatt-test --locked --offline
    else
      cargo +esp clippy --release --features secure-gatt-test --locked --offline -- -D warnings
    fi
    ;;
  session-build|session-check)
    export CARGO_TARGET_DIR="$project_root/target/esp32-session"
    cd "$project_root/apps/esp32-player"
    if [[ "$action" == *-build ]]; then
      cargo +esp build --release --features session-readiness --locked --offline
    else
      cargo +esp clippy --release --features session-readiness --locked --offline -- -D warnings
    fi
    ;;
  worker-test-build|worker-test-check)
    : "${STAGEMASTER_PROBE_PACKAGE:?须明确指定项目内真实导出的测试包绝对路径}"
    case "$STAGEMASTER_PROBE_PACKAGE" in
      "$project_root"/data/*) ;;
      *) printf '测试包必须位于项目 data 目录\n' >&2; exit 2 ;;
    esac
    export CARGO_TARGET_DIR="$project_root/target/esp32-worker-test"
    cd "$project_root/apps/esp32-player"
    if [[ "$action" == *-build ]]; then
      cargo +esp build --release --features worker-write-test --locked --offline
    else
      cargo +esp clippy --release --features worker-write-test --locked --offline -- -D warnings
    fi
    ;;
  installation-repair-build|installation-repair-check)
    export CARGO_TARGET_DIR="$project_root/target/esp32-installation-pair"
    cd "$project_root/apps/esp32-player"
    if [[ "$action" == *-build ]]; then
      cargo +esp build --release --features binding-repair-test --locked --offline
    else
      cargo +esp clippy --release --features binding-repair-test --locked --offline -- -D warnings
    fi
    ;;
  installation-pair-build|installation-pair-check)
    export CARGO_TARGET_DIR="$project_root/target/esp32-installation-pair"
    cd "$project_root/apps/esp32-player"
    if [[ "$action" == *-build ]]; then
      cargo +esp build --release --features installation-gatt,binding-local-test --locked --offline
    else
      cargo +esp clippy --release --features installation-gatt,binding-local-test --locked --offline -- -D warnings
    fi
    ;;
  installation-build|installation-check)
    export CARGO_TARGET_DIR="$project_root/target/esp32-installation"
    cd "$project_root/apps/esp32-player"
    if [[ "$action" == *-build ]]; then
      cargo +esp build --release --features installation-gatt --locked --offline
    else
      cargo +esp clippy --release --features installation-gatt --locked --offline -- -D warnings
    fi
    ;;
  binding-test-build|binding-test-check)
    export CARGO_TARGET_DIR="$project_root/target/esp32-binding-test"
    cd "$project_root/apps/esp32-player"
    if [[ "$action" == *-build ]]; then
      cargo +esp build --release --features binding-local-test --locked --offline
    else
      cargo +esp clippy --release --features binding-local-test --locked --offline -- -D warnings
    fi
    ;;
  storage-build|storage-check|runtime-build|runtime-check|security-build|security-check|worker-build|worker-check|binding-build|binding-check)
    profile="${action%%-*}"
    export CARGO_TARGET_DIR="$project_root/target/esp32-$profile-check"
    cd "$project_root/apps/esp32-player"
    if [[ "$action" == *-build ]]; then
      cargo +esp build --release --features "$profile-readiness" --locked --offline
    else
      cargo +esp clippy --release --features "$profile-readiness" --locked --offline -- -D warnings
    fi
    ;;
  storage-report|runtime-report)
    python3 "$project_root/tools/hardware/storage-report.py" "${action%%-*}"
    ;;
  storage-size|runtime-size)
    xtensa-esp32s3-elf-size -A "$project_root/target/esp32-${action%%-*}-check/xtensa-esp32s3-none-elf/release/stagemaster-esp32-probe"
    ;;
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
    "$project_root/tmp/esp-tools/espflash" flash --port "$2" --chip esp32s3 --flash-size 16mb \
      --partition-table "$project_root/apps/esp32-player/partitions-storage.csv" \
      --target-app-partition ota_0 --non-interactive "$firmware"
    ;;
  monitor)
    : "${2:?明确指定开发板串口}"
    "$project_root/tmp/esp-tools/espflash" monitor --port "$2" --non-interactive --no-reset --skip-update-check --elf "$firmware"
    ;;
  *) printf '用法：%s {build|check|size|storage-build|storage-check|storage-size|storage-report|runtime-build|runtime-check|runtime-size|runtime-report|secure-gatt-build|secure-gatt-check|session-build|session-check|security-build|security-check|worker-build|worker-check|worker-test-build|worker-test-check|binding-build|binding-check|binding-test-build|binding-test-check|installation-build|installation-check|installation-pair-build|installation-pair-check|installation-repair-build|installation-repair-check|flash 串口|monitor 串口}\n' "$0" >&2; exit 2 ;;
esac
