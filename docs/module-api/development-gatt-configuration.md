# 受控开发设备的直接 GATT 安装

DEVICE-002；[ADR-051](../development/decisions/PRODUCT-ADR-051-development-gatt-installation.md)。这是专用开发身份的实际无线安装，不是生产云端认领或文件播放授权。

## 凭据与调用

`device-auth::application::Configuration::import(bytes, Role)` 只接受可信本地输入，严格检查 160 字节格式、角色、本端公私钥配对、对端公钥、主体、修订和最长 10 分钟的连接许可。对象没有 Debug、序列化或私钥导出。输入者负责清除原始字节；固件嵌入配置的 ELF、目标文件同样属于秘密产物。

原生宿主先调用 `read_development_configuration(path)`，拒绝符号链接、非普通文件、长度／格式／角色错误和非 0600 的 Unix 文件。随后 `Ble::with_development_configuration(configuration)` 接入既有 `Service<Ble>`。前端不收发密钥，安装仍由独立 `device-upload::Service` 管理。

`Ble::default()` 不读取环境或系统配对信息。没有匹配设备的可信配置时可以诊断，没有安装权限。旧绑定适配仅可通过显式 `bonded-experiment` 特性及 `experimental_bonded()` 调用，普通应用不走此分支。旧配对记录是否存在均不影响新路径。

桌面仅在 `development-device-access` 特性构建中读取 `STAGEMASTER_CONTROLLER_CONFIGURATION`，加载失败拒绝启动；普通构建保持无开发权限。生产版本需要独立凭据提供者／云端验证器，不能把此环境变量当用户认领流程。

## 无线与任务所有权

- 诊断保留 ed60 服务；应用安装为 `f889eda0-0100-4e83-968e-799ab99558fa`，eda2 连续写、eda3 通知。连接层先互相确认 Noise 身份，再收加密 SMAP 就绪回执。原生核对设备、启动、连接、会话、主体、修订及固定期限。
- 设备以单调工作代次隔离连接，Gateway 发布 LIVE_EPOCH，工作器独占 NOR；断线／过期清零，广播期间也排空旧完成。没有公开“赋予权限”写特征。
- 固定安全记录队列、固定半包期限和平台背压避免无界累积。主机接收队列最多 4 条，溢出／乱序／结束即失效；发送串行等待平台写入额度。固件第一安全片到来时冻结已协商 MTU。
- 主机连接服务是唯一通信所有者，加密保活与节目请求串行协调。普通诊断不能延长安全会话；有效保活也不能延长开发许可。过期需要重新认证，当前没有自动续签。
- 安装结果以设备持久提交回执为准；ATT 发送完成不能显示“已安装”。不可变包、已确认字节、取消意图、重新连接与丢回执对账沿用既有任务服务。失败不得降级为明文或系统自动配对。

## 可复现命令

从项目根运行，现有凭据不可覆盖或打印。首次生成仅面向已授权开发设备：

```sh
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo run -p stagemaster-device-auth --example development_credentials --features application --locked --offline -- 534d4553503332533300288485569774 "$PWD/data/DEVICE-002/development-access"
STAGEMASTER_DEVICE_CONFIGURATION="$PWD/data/DEVICE-002/development-access/device.smddev" bash tools/hardware/firmware.sh application-check
STAGEMASTER_DEVICE_CONFIGURATION="$PWD/data/DEVICE-002/development-access/device.smddev" bash tools/hardware/firmware.sh application-build
```

输出位于权限 0700 的 `target/esp32-application/`。构建不会刷机；已有当前硬件授权时仍须显式指定 ELF、串口、`partitions-storage.csv` 和 ota_0，不能用默认 `flash` 误刷只读版本。两份凭据各自 0600，新目录 0700，均被 Git 忽略。设备配置只能给本设备构建，不下发给客户端／云端。

释放其他客户端的蓝牙连接后，用与桌面相同的原生服务验收：

```sh
STAGEMASTER_CONTROLLER_CONFIGURATION="$PWD/data/DEVICE-002/development-access/controller.smddev" CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo run -p stagemaster-device-upload --example install_device --locked --offline -- "$PWD/data/DEVICE-002/venue-scenes.smpkg" install
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo run -p stagemaster-device-host --example application_denials --locked --offline -- "$PWD/data/DEVICE-002/development-access/controller.smddev"
```

`install_device` 另支持 `cancel`／`resume`／`corrupt`／`lost-commit`；后三种需要与当前有效节目不同的真实包，不能把同包快速对账算作传输故障验收。`corrupt` 在加密前改一块，检查实板拒绝并取消失败事务；`lost-commit` 在主机测试层扣留已收到的真实提交回执，再断线重新认证、查询同代结果。它模拟上层丢回执，不声称制造了真实无线丢包。

当前仅 macOS＋本台 ESP32 经过实际无线验证。跨端重用协议和 Rust 模块，不代表其他平台已经验收；未来云端归属／凭证签发按 ADR-046，商业保护范围按 ADR-101，具体加密授权方案后议，均与上述开发连接许可分离。
