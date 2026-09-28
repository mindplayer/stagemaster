# DEVICE-002B：macOS 安全配对候选正向验收

日期：2026-09-29；基线 `3baa1ce`，主工作区，当前会话直接实施。结果为本次 `fix(hardware): honor native pairing read deadline` 提交。

用户明确确认当前 Mac 与 ESP32 的测试连接／配对，此前系统信任确认阻塞已解除。本轮仍禁止 RS485／DMX、eFuse 和商业授权操作，不修改用户工程。

## 实际经过与结果

1. 从当前源码重新构建、严格检查 `security-readiness`，均通过。显式沿用存储分区表，只更新 ota_0；应用镜像 590,400 B，保留已有 RWX 段告警。这个镜像没有节目写入入口，也没有绑定持久化。
2. 使用独立只读认证特征触发 macOS 原生“连接请求来自：StageMaster”。从当前板卡 USB 读取随机临时码并填入；板端实际报告 `EncryptedAuthenticated`、`bonded=true`。串口落盘时将临时码替换为 `[REDACTED]`，不记录绑定密钥。
3. 首轮主机读取失败：Bleak 2.1.1 的 CoreBluetooth 公共读方法不转发 timeout 参数，内部 delegate 默认 20 秒；外层 80 秒并不能延长它。系统配对已完成，但等待该读取的脚本先超时。此失败日志保留，不算整条探针通过。
4. 未重启板卡，重新执行现有探针，两次已认证读取、正常握手、各 10 秒保活和断线重连通过。串口分别报告 `EncryptedAuthenticated; resumed=true`，堆稳定为 41,044 B 已用／90,028 B 空闲。证明同一次启动中的内存绑定可恢复加密，不证明断电后绑定恢复。
5. 新增独立 `secure_read.py` 测试适配器，让实际 CoreBluetooth delegate 使用有界 80 秒期限，仍走原生认证和资源清理，不修改安装依赖或库源码。其他平台沿用公开读取接口。修正版的两次真实已认证读取／重连通过；未再清除系统绑定，因此首次配对超过 20 秒的完整修正版路径尚未实测。
6. 正式舞台大师窗口重新搜索后发现当前 StageMaster，选择、连接、自检及保活通过。第一段连接已观察到 33 次保活；修正版探针完成后恢复应用连接，最终又确认 22 次保活、最近往返 157 毫秒，面板保持展开。原圆弧剧场保持“已保存”；原生连接能力仍为诊断，未授予节目安装权限。先前面板记录的旧连接标识与当前启动的标识不同；搜索刷新后可用，不能据此认定此前所有连接失败的根因。

## 改动与证据

- 产品 Rust、工程格式、持久化格式及现有界面未改变；本轮只修正测试客户端的真实超时边界。
- 新适配器 26 行，现有探针仍保持小文件。私有 delegate 调用仅用于锁定 Bleak 2.1.1 的候选实验；升级该依赖时须重新核验，不将它作为正式跨端传输接口。
- 构建／检查：`logs/device-002b-pairing-resume-{build,check,flash}.log`。
- 首轮失败与脱敏串口：`logs/device-002b-pairing-resume-{probe,serial}.log`。
- 旧脚本正向恢复：`logs/device-002b-pairing-reconnect-probe.log`；修正版：`logs/device-002b-pairing-bounded-probe.log`。两个成功日志各含两条 `authenticated_read_pass`、两条 `authenticated_heartbeat_pass` 和最终 `PASS`。
- Python 语法和差异检查通过；没有重复不相关的 Rust 全工作区或 UI 测试。上一轮 347 Rust／最终 19 定向结果不改记为本轮结果。

## 当前边界和后续

板卡保留本轮只读安全候选固件，内存绑定在当前启动有效，舞台大师显示诊断连接。此次没有操作节目区；也未重新读取 A／B 槽，不把上一轮的槽摘要当作本轮重验。

当前已证实 macOS 正向 LE Secure Connections 认证及同启动重连可行。错误码／取消／晚到结果、绑定持久化和撤销、物理准入、连接级业务会话及正式 GATT 安装仍待完成；配对成功不代表完整 DEVICE-002 或生产加密／离线授权已经交付。后续基于 [ADR-036](../decisions/PRODUCT-ADR-036-authenticated-device-session.md) 收敛这些生命周期边界，再接已有维护工作器与桌面任务。
