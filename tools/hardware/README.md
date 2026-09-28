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

刷写需要当前设备测试授权，显式指定串口；命令会写诊断固件及引导，并显式采用 `partitions-storage.csv`／ota_0，避免将已配置节目存储的设备改回旧 factory 全占用分区。此命令不备份旧固件、不自动发送节目，也不烧写 eFuse。本轮用户已授权且明确旧固件不保留。

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

macOS 正向配对及同启动重连已实测，见[验收](../../docs/development/tasks/DEVICE-002B-pairing-acceptance.md)。Bleak 2.1.1 公共 CoreBluetooth 读取不转发 timeout，内部默认 20 秒；实验通过独立 `secure_read.py` 使用同一原生 delegate 的 80 秒有界读取。升级依赖须核对该私有实验适配；正式产品不依赖它。临时配对码只在 USB 本地显示，持久日志必须脱敏；不要输出绑定密钥。

## NOR 存储准备（PLAYER-003D）

新增 `storage-readiness` 可选特性，使用官方分区解析／NOR 区域和独立包存储，只读检查、无安装连接。构建目录 `target/esp32-storage-check/` 与默认诊断镜像隔离；没有存储刷机命令。DEVICE-002 已受控安装此分区表并改用 SDK 支持的 data／undefined 类型；旧 0x40 类型会被明确拒绝，不能直接在旧表启用节目写入。决定见 [ADR-037](../../docs/development/decisions/PRODUCT-ADR-037-installation-worker.md)。

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

## 双核安装工作任务（DEVICE-002C）

`worker-readiness` 在第二核只读恢复存储；`worker-write-test` 是独立、显式的本地擦写验收镜像。两者都没有无线安装权限，固定测试主体不能当设备认证。Flash 驱动采用官方多核停放；GPIO21 全程保持低电平，没有任何现场输出。

```sh
bash tools/hardware/firmware.sh worker-build
bash tools/hardware/firmware.sh worker-check
STAGEMASTER_PROBE_PACKAGE="$PWD/data/PLAYER-003A/final.smpkg" bash tools/hardware/firmware.sh worker-test-build
STAGEMASTER_PROBE_PACKAGE="$PWD/data/PLAYER-003A/final.smpkg" bash tools/hardware/firmware.sh worker-test-check
```

编译命令不刷机。已有相应测试授权时，明确刷入 `target/esp32-worker-test/xtensa-esp32s3-none-elf/release/stagemaster-esp32-probe`，同时指定 `--partition-table apps/esp32-player/partitions-storage.csv --target-app-partition ota_0`；必须在启动后立即运行下列监听，以覆盖约 45 秒后的本地测试。测试镜像会在该时刻将编译时嵌入的包写入专用备用槽，不能当成只读镜像。

```sh
PYTHONDONTWRITEBYTECODE=1 tmp/ble-probe/bin/python tools/hardware/worker_probe.py \
  --port /dev/cu.usbmodem2101 --output data/DEVICE-002/worker-check.json --expect-write
```

首次新包使用 `--expect-write`；同一镜像重启复测改用 `--expect-existing`，断言持久恢复且实际擦写计数全为零。恢复只读 `worker-readiness` 后使用 `--expect-readonly-recovery`，额外断言未运行本地写探针。脚本持续测量诊断 BLE 保活、内核推进、结束时堆回收，失败后仍观察串口 10 秒并保留原失败及数据，不自动重连伪装通过。测试采用 reset／刷写后重启，不等同物理拔电。

可用 `stagemaster-install-store` 的 `install` 示例生成电脑参考帧摘要，向探针传 `--reference-replay logs/对应参考.log`，核对固件从已安装 Flash 读取的每个节目各 400 帧。所有帧仅在 RAM 编码和计算摘要，未连接 DMX 输出驱动。固件中的共享堆即时采样会受另一核无线临时分配影响，不能拿两个瞬时值的差异断言泄漏；主机仍检查结束时占用返回基线。

`crates/stagemaster-project/examples/export_package.rs` 复用正式电脑编译器，输入工程、显式节目选择 JSON、新包路径；拒绝覆盖旧文件，无设备访问。可用于生成真实工程规模的验证包，不把填充随机数据当合法节目。

工作器对操作耗时、物理 NOR 次数／耗时、调用前栈深度和分配器历史峰值做汇总；避免每块输出完整事务状态。SDK 串口打印本身使用临界区，过多日志会干扰无线时序。成功及失败的实测结果、当前限制见 [DEVICE-002C 验收记录](../../docs/development/tasks/DEVICE-002C-worker-acceptance.md)。

当前本地探针的状态查询使用 20 字节片段、安装使用 244 字节片段，均通过正式 `Endpoint` 和实际跨核工作队列；监听脚本增加 `--expect-byte-channel` 可强制检查这两条路径及目标类型尺寸。这里只在板内传递节目片段，同时另测诊断 BLE；不能把它计作节目已通过蓝牙下发。正式认证与 GATT 接入仍是独立出口，见 [字节通道接口](../../docs/module-api/installation-byte-channel.md)。

## 运行维护集成（DEVICE-002C）

当前 worker-readiness／worker-write-test 均使用 ManagedWorker，通过 Runtime 的有效维护窗口执行每个存储命令。只读构建仍不写包；本地写测试增加真实目录绑定、运行态拒写和重新进入维护的断言。使用 `worker_probe.py --expect-maintenance-cycle` 验证这条额外路径；该选项应与明确构建／刷入的 worker-write-test 及原有包逐帧参考一起使用，不能在只读固件上假定会发生。机制与本轮数据见[维护验收](../../docs/development/tasks/DEVICE-002C-maintenance-acceptance.md)。GATT 正式安装仍须完成认证，GPIO21 始终禁用。
