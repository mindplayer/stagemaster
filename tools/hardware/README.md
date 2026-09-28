# ESP32 第一轮验证工具

PLAYER-002A 仅诊断：共享播放内核自检／负载、BLE GATT 直连及会话保活。GPIO21 固定低电平，RS485 发送关闭；没有节目上传、控制或正式 DMX 输出。

## 项目内环境

所有下载、缓存与环境均留项目 `tmp/`，不改用户全局 Rust 或 shell 配置。

- espup **0.17.1**，Apple Silicon 官方发布二进制 SHA-256 `ab0e937d659396ed2b3b0c0f74d29bdf570217f096ea88fa58b8966cf4d32cba`。
- espflash **4.6.0**，Apple Silicon ZIP SHA-256 `f39bff252a181a6e345991f603d7606cf9762550e557073c1282eada46d8c757`。
- Xtensa Rust **1.97.0.0**，实际 rustc `1.97.0-nightly (8ea53bcd7 2026-07-08)`；GCC `esp-15.2.0_20250920`。
- 固件顶层依赖精确锁定在 `apps/esp32-player/Cargo.toml`，完整依赖见独立 `Cargo.lock`。`esp-radio 1.0.0-beta.1` 与部分 HAL API 尚不稳定，当前结论限于锁定版本的诊断；不是生产认证。
- 主机测试 Python 环境 `tmp/ble-probe`：`bleak==2.1.1`、`pyserial==3.5`；Python 仅用于测试，不是新增产品运行时。

从 [espup 官方发布](https://github.com/esp-rs/espup/releases/tag/v0.17.1) 和 [espflash 官方发布](https://github.com/esp-rs/espflash/releases/tag/v4.6.0) 下载对应文件，核对摘要后将可执行文件放到 `tmp/esp-tools/`。首次工具链安装（项目根目录）：

```sh
CARGO_HOME="$PWD/tmp/cargo-home" RUSTUP_HOME="$PWD/tmp/esp-rustup" TMPDIR="$PWD/tmp" \
  ./tmp/esp-tools/espup install --targets esp32s3 --toolchain-version 1.97.0.0 \
  --crosstool-toolchain-version 15.2.0_20250920 --name esp \
  --export-file "$PWD/tmp/esp-tools/export-esp.sh"
```

首次缺依赖时，在 `apps/esp32-player` 中用同样的项目绝对环境变量执行 `cargo +esp fetch --locked`，之后离线构建。不修改全局默认工具链。`espup` 的下载仍依赖官方发布可访问。

## 复现

```sh
bash tools/hardware/firmware.sh build
bash tools/hardware/firmware.sh check
bash tools/hardware/firmware.sh size
```

刷写需要当前设备测试授权，显式指定串口；命令会写诊断固件及默认引导／分区，不备份旧固件，不支持用户节目持久安装，也不烧写 eFuse。本轮用户已授权且明确旧固件不保留。

```sh
bash tools/hardware/firmware.sh flash /dev/cu.usbmodem2101
bash tools/hardware/firmware.sh monitor /dev/cu.usbmodem2101
```

GATT 验收脚本只识别唯一匹配的诊断服务；没有匹配或有多个即失败。系统蓝牙权限由 macOS 管理，不绕过权限。脚本需要板卡已启动并广播，串口监听不是蓝牙测试的前置条件。

```sh
UV_CACHE_DIR="$PWD/tmp/uv-cache" uv venv tmp/ble-probe
UV_CACHE_DIR="$PWD/tmp/uv-cache" uv pip install --python tmp/ble-probe/bin/python 'bleak==2.1.1' 'pyserial==3.5'
TMPDIR="$PWD/tmp" tmp/ble-probe/bin/python -u tools/hardware/gatt_probe.py
```

测试服务读写／通知一致性、错误版本／会话、重复／乱序、长度拒绝、30 秒心跳、无有效心跳约 6 秒断开、三次重连和本地内核持续推进。`PASS` 不代表 DMX 电气、真实灯具或长期稳定性通过。

固件 ELF 的 `.rotext_dummy` 与 `.text` 共段会触发工具链 RWX 段告警；保留告警和 `readelf` 记录，没有用编译开关掩盖。该裸机链接属性不等于运行时权限隔离；生产内存保护须单独审查。

## 只读设备身份（DEVICE-002A）

默认诊断固件现为 0.2.0。新增只读 96 字节身份／能力特征，仍不接受节目安装或控制；格式见[设备描述](../../docs/module-api/device-description.md)。两次运行之间显式重启，用独立编解码核对稳定设备身份／新启动身份；脚本每次检查三次重连与 24 次保活。

```sh
PYTHONDONTWRITEBYTECODE=1 TMPDIR="$PWD/tmp" tmp/ble-probe/bin/python tools/hardware/device_description_probe.py --output data/DEVICE-002/identity-before.json
tmp/esp-tools/espflash reset --port /dev/cu.usbmodem2101 --chip esp32s3 --non-interactive --skip-update-check
PYTHONDONTWRITEBYTECODE=1 TMPDIR="$PWD/tmp" tmp/ble-probe/bin/python tools/hardware/device_description_probe.py --previous data/DEVICE-002/identity-before.json --output data/DEVICE-002/identity-after.json
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo run -p stagemaster-device-host --example inspect_device --locked --offline
```

最后一条复用产品原生宿主，验证实际描述读取／保活／断开清除。运行这些工具前释放应用当前蓝牙连接；不要并发运行两个客户端。同一 Rust target 目录的构建／测试也应串行，避免独立 Cargo 工作区构建干扰。

## 安全连接候选（DEVICE-002B）

`security-readiness` 仅加入要求 LE 已认证加密的只读探针；不包含安装、播放或输出。独立输出目录 `target/esp32-security-check/`。候选和实板发现见 [ADR-036](../../docs/development/decisions/PRODUCT-ADR-036-authenticated-device-session.md)。默认刷写命令仍只刷默认诊断构建，不会自动选择这个实验镜像。

```sh
bash tools/hardware/firmware.sh security-build
bash tools/hardware/firmware.sh security-check
```

受控刷入候选镜像后，`admission_probe.py` 不配对，只验首握手前固定期限和握手后正常过期。`secure_link_probe.py` 才会触发系统验证码流程，须已有当前设备配对确认，并由 USB 本地读取动态验证码；不会选固定密码或跳过失败。两者均使用项目 Python 环境及 `PYTHONDONTWRITEBYTECODE=1`，避免缓存写到源码目录。绑定目前仅在设备 RAM，持久化／撤销／业务权限和正向跨端验收未完成。

## NOR 存储准备（PLAYER-003D）

新增 `storage-readiness` 可选特性，使用官方分区解析／NOR 区域和独立包存储，只读检查、无安装连接。构建目录 `target/esp32-storage-check/` 与默认诊断镜像隔离；没有存储刷机命令。当前板卡尚未安装 `partitions-storage.csv`，不能直接在默认旧分区启用节目写入。

```sh
bash tools/hardware/firmware.sh storage-build
bash tools/hardware/firmware.sh storage-check
bash tools/hardware/firmware.sh storage-size
bash tools/hardware/firmware.sh storage-report
```

报告直接读取 Xtensa ELF 类型大小和入口栈帧，不是硬件动态峰值。详细[预算／限制](../../docs/module-api/nor-package-store.md)和[验收](../../docs/development/tasks/PLAYER-003D-flash-store.md)。本地分区和镜像验证命令不会访问设备：

```sh
mkdir -p data/player-003d
TMPDIR="$PWD/tmp" tmp/esp-tools/espflash partition-table --skip-update-check --to-binary apps/esp32-player/partitions-storage.csv --output data/player-003d/partitions.bin
TMPDIR="$PWD/tmp" tmp/esp-tools/espflash partition-table --skip-update-check --to-csv data/player-003d/partitions.bin --output data/player-003d/partitions-roundtrip.csv
TMPDIR="$PWD/tmp" tmp/esp-tools/espflash save-image --chip esp32s3 --flash-size 16mb --skip-update-check --partition-table apps/esp32-player/partitions-storage.csv --target-app-partition ota_0 target/esp32-storage-check/xtensa-esp32s3-none-elf/release/stagemaster-esp32-probe data/player-003d/storage-readiness.bin
```

## 设备运行准备（PLAYER-003E）

新增 `runtime-readiness` 特性，在存储检查基础上只读恢复 Runtime 的包绑定，真实策略拒绝播放，不建立控制连接。保留完整运行请求／装载／帧生成函数进行代码生成和链接；类型检查不能替代这个构建。

```sh
bash tools/hardware/firmware.sh runtime-build
bash tools/hardware/firmware.sh runtime-check
bash tools/hardware/firmware.sh runtime-size
bash tools/hardware/firmware.sh runtime-report
```

目录 `target/esp32-runtime-check/` 与诊断／存储检查产物分别保留。没有运行镜像刷写命令；具体[运行契约和预算](../../docs/module-api/device-runtime.md)。报告脚本仅调用 binutils 读取 ELF，不接触板卡。安装维护需实际输出静默确认；当前软件检查不证明无线、看门狗和任务栈动态峰值。
