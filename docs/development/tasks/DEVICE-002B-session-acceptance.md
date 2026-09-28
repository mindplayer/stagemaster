# DEVICE-002B：应用层安全协议核心与板内资源验证

日期：2026-09-29；基线 `050cb82`；主工作区、当前 Astra 单一负责人。结果为本次 `feat(device): add bounded Noise session and board resource probe` 提交。父 DEVICE-002 保持完整目标，未完成无线认证安装。

## 变更与审查

按 [ADR-046](../decisions/PRODUCT-ADR-046-cloud-owned-direct-gatt.md)／[ADR-047](../decisions/PRODUCT-ADR-047-noise-application-session.md)，新增独立 `stagemaster-device-session`。采用锁定 Snow 0.10.0 的 Noise IK，而非自行实现密码算法；注入随机源、有界消息、相互确认、连接上下文、防重放、时间和关闭语义。身份持有证明与云端权限明确分开。接口见[模块契约](../../module-api/device-session.md)。

源代码审查补拒绝低阶输入的全零 DH 结果；审查确认期限时发现设备收到确认后可能提前延长握手期限，已修为发送确认回复仍检查原 10 秒上限和最后入站 6 秒上限，并加入两个边界回归。生产代码按密钥、握手、确认、记录拆分，新增产品源码最大 146 行；未向原页面／宿主大文件堆功能。

## 软件与构建

- 最终全工作区 385 项 Rust 测试、严格 Clippy 通过；新模块 11 项包含双方角色分别与独立 noise-protocol／noise-rust-crypto 实现互操作、错误密钥／上下文／低阶输入、首握手重放、认证前业务、乱序／重复／损坏、缓冲清理、时间倒退／期限／延迟确认及随机源失败。独立实现仅为开发依赖，不进入固件。
- Xtensa `session-readiness` 和只读 `worker-readiness` 两种实际 release 链接／严格 Clippy 通过；原有裸机 RWX 段告警保留。根／固件格式和差异检查通过。
- 最终日志：`logs/device-002-session-workspace-{tests,clippy}-final.log`、`device-002-session-{tests,clippy}-final.log`、`device-002-session-firmware-check-fixed.log`、`device-002-session-firmware-build.log`、`device-002-session-readonly-{check,build}.log`。
- 初始 zeroize 锁版本冲突、prologue 常量长度错误导致互操作测试 panic、Clippy 诊断和 esp-hal 时间 API 编译错误均已修复；原失败日志保留，未删测试或放宽检查。

## ESP32 实测

授权范围内刷入 584,672 B 本地安全协议探针；不提供安装 GATT、不保存密钥、不写节目、不启用系统绑定。板端使用 RF 开启后的硬件随机源生成两个临时身份，实际完成双方握手／确认、128 轮最大载荷加解密和加密保活回复、损坏认证标签拒绝／清空缓冲。

| 指标 | 本次实测 |
| --- | --- |
| 双方密钥生成、握手、确认与检查 | 499,972 μs |
| 128 轮 1280 B 消息＋保活回复 | 217,123 μs |
| 起始／释放后的堆占用 | 均 35,724 B |
| 握手时／通道时堆占用 | 36,828／35,852 B |
| 累计堆峰值 | 36,836 B，比起始多 1,112 B |
| Handshake／Channel 对象大小 | 800／256 B |

这是同一 ESP32 内双方计算测量，不是 BLE 吞吐、无线往返、满负载并发或任务栈高水位；不能据此承诺真实传输性能。测后诊断堆 41,044 B 稳定，内核持续推进。日志 `logs/device-002-session-{flash,board}.log`。

## 恢复与现有数据

已恢复 556,320 B `worker-readiness` 只读镜像，启动不会重复执行安全协议压力测试，不依赖系统配对。真实 NOR 恢复选择 A 第 9 代、106,825 B／27 场景、摘要 `2fafb9eff721cfe3e2b8eaa64399a428be6d39bdb45325035a588973dc0295db`；B 为 Empty（上轮中断后没有有效提交，不能声称 B 第 8 代仍完整）。工作器只读状态查询通过，计数 `writes=0 erases=0`。USB 观察启动实报 USB_UART_CHIP_RESET，不称为拔电验收。日志 `logs/device-002-session-restore-readonly.log`、`device-002-session-readonly-board.log`。

原舞台大师 PID 44624 重新发现设备 locator `539ef19d-0d98-5743-db54-9e8cb0095e41`，应用内直接选择／连接成功，10 次保活、最近往返 157 ms，无系统配对操作。原圆弧工程未修改；板端日志确认 GPIO17 连接绿灯输出，GPIO21 保持低。用户此前确认过同实现指示灯实物，本轮未要求再次确认。系统已有历史配对条目未再次删除，新固件不使用其绑定恢复。

## 未完成项与后续

仍需版本化 GATT 密文分片、双端安全通道、独立可信凭据／授权适配、真实无线安装及桌面进度／取消／恢复验收。旧 LESC 安装 27,648 B 后断开的根因未因换协议自动消除。云端服务、生产认领／可信时间／24 小时文件许可尚未实现；本增量不签发安装权限、不修改设备声明或持久格式。先完成可测的直连链，再按既有云端边界接入，不要求每次重连联网。完整 goal 保持 active。
