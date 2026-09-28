# 应用层安全会话

DEVICE-002B；实现 `crates/stagemaster-device-session`；依据 [ADR-047](../development/decisions/PRODUCT-ADR-047-noise-application-session.md)。这是已实现的协议核心，尚未接到正式 GATT／云端授权。实测边界见[验收](../development/tasks/DEVICE-002B-session-acceptance.md)。

## 职责与依赖

独立 no_std＋alloc Rust 模块，仅依赖选定的 Snow 密码原语、subtle、zeroize。安全随机源由宿主注入；BLE、文件、云端、安装任务和播放核心都在模块外。控制端必须预先取得可信设备公钥；广播里的公钥不能直接成为信任依据。

[安全记录字节通道](secure-record-channel.md)已作为独立无堆适配实现并完成软件组合测试；双方生产依赖仍分离，仅由测试组合，不代表正式 GATT 已接通。

后续[免配对 GATT 实板实验](../development/tasks/DEVICE-002-secure-gatt-acceptance.md)已完成真实握手、保活、消息与故障拒绝。独立的[应用权限及安装适配](application-installation-session.md)已接入软件组合验证；实际凭据配置、正式 GATT／桌面接线和云端仍待完成。无线实验使用 USB 固定的启动期公钥，不代表云端认领。

`SecretKey` 持有本端秘密，不提供 Debug／Serialize／Clone；密钥存储与生产配置另由凭据适配负责。`Context` 绑定稳定设备号、启动身份和连接随机号。`PeerProof` 只能由完成相互确认的通道取得，表示当前对端持有相应密钥，**不是安装授权或账号归属**；保存它的副本不会冻结有效期。

## 调用形状

以下省略 I/O 和错误分支，真实调用必须在单一连接任务内按顺序执行：

```rust
let mut handshake = Handshake::initiate(
    context, &local_key, trusted_device_key, platform_entropy, now_ms,
)?;
let length = handshake.write(&mut handshake_buffer, now_ms)?;
// 适配层发送完整消息并重组设备响应；不暴露半包给协议核心。
handshake.read(&complete_response, later_ms)?;
let mut channel = handshake.finish(later_ms)?;
let length = channel.confirmation(&mut cipher_buffer, later_ms)?;
// 发送加密确认，收到设备确认回复。
channel.confirm(&complete_confirmation, next_ms)?;
let proof = channel.peer(next_ms)?.ok_or(NotConfirmed)?;
// 独立授权适配核对 proof、公钥对应的有效资格、目标和权限。
// 未通过之前，禁止调用安装／控制业务入口。
let length = channel.seal(Kind::Message, &request, &mut cipher_buffer, next_ms)?;
let record = channel.open(&response, &mut plaintext_buffer, after_ms)?;
```

设备端使用 `Handshake::respond`，读取首包、发送响应、`finish`，先 `confirm` 再 `confirmation`。发送侧必须确保输出完整消息进入有界发送路径；发送失败立即丢弃通道，不能继续用已推进的 nonce。重连创建全新上下文与状态，不从旧对象恢复计数。

## 缓冲、时间和失败

| 项目 | 契约 |
| --- | --- |
| 握手 | 固定 IK 两消息 96／48 B；无业务载荷 |
| 双向确认 | 加密 33 B 角色＋握手摘要，密文 49 B |
| 业务 | 类型 1 B＋载荷 1～1280 B；密文最多 1297 B |
| 保活／回复 | 分别独立类型，载荷必须为空，密文 17 B |
| 握手期限 | 创建后固定 10 秒，包含双向确认，碎片／诊断不续期 |
| 活跃期限 | 最后有效加密入站记录后 6 秒；发送、轮询不续期 |
| 时间源 | 宿主单调毫秒；倒退、溢出、到达期限即关闭 |

主循环即使无数据也必须轮询期限；握手阶段的无数据超时由外层定时器销毁对象，不能只在收到下一包时才清理连接。业务通过 `Channel::poll`／`peer` 检查；授权适配另检查凭证有效期和撤销状态。心跳是链路存活证明，不能延长云端凭证或文件播放许可。

任一顺序、类型、长度、重放、认证、时间或随机源错误都关闭当前状态；无降级明文路径。解密错误清空整个调用方明文缓冲。成功返回的载荷借用该缓冲，由调用方完成处理后清理；取消、断线或队列故障清理重组、待发、权限和旧完成。此层不拥有任务看门狗和重连退避。

ESP32 本地双端测试验证堆恢复和有限分配；不是零分配实现。1280 B 明文／密文工作缓冲及 Snow 内部对象也占栈／堆，实际 GATT 任务栈高水位仍待测。认证编号 2 用于独立 SMAP 加密就绪回执，后续 ADR-051 的专用 `application-gatt` 镜像已开放并实测安装；默认诊断构建不开放。旧 LESC 回执保持原编号。实际接线见[开发 GATT 配置](development-gatt-configuration.md)。
