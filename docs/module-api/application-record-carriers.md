# 可替换的应用记录承载

[TRANSPORT-001](../development/tasks/TRANSPORT-001-record-carriers.md)／[ADR-102](../development/decisions/PRODUCT-ADR-102-record-transport-boundary.md)。`stagemaster-device-channel` 抽取既有主机应用会话；正式 BLE 路径调用共同实现，第二承载通过本机 TCP 和内存流验证。该模块不拥有设备发现、工程、安装事务、节目执行或物理输出。

## 分层与调用

1. 外部平台适配选择端点、建立连接、读取描述；端点地址和未认证描述不构成设备身份。
2. `RecordIo` 搬运完整、不可信、有界记录；GATT 负责 MTU 分片／连续序号，`StreamRecords` 负责字节流边界。两者都不授予权限。
3. `Channel<R>` 拥有现有 Noise 会话、可信配置核对、双向确认、加密就绪回执、消息和保活。安装／运行分别准备，只有匹配当前设备／启动／连接／主体／权限修订的回执才产生对应连接事实。
4. 上层 `Upload`／`transfer`／`Installer` 继续拥有安装请求、回执关联和持久提交；[RuntimeClient](device-runtime-client.md) 拥有运行请求及回复关联。载体不把“发送成功”解释为“已安装”或“已执行”。

```rust,ignore
// connected_io 已由平台建立；trusted_config 只能来自可信本机注入。
let records = StreamRecords::new(connected_io);
let mut channel = Channel::prepare(records, description, &trusted_config).await?;
let facts = channel.peer().ok_or(NeedsAuthentication)?;
upload.connect(facts.session)?;
// 单任务串行协调消息与保活；完整调用参考 tests/installation.rs。
channel.write(request.bytes()).await?;
channel.heartbeat().await?;
if let Some(response) = channel.receive()? {
    upload.accept(&response)?;
}
```

`peer()` 只在当前会话有效时返回原就绪回执，不是可跨连接保存的权限凭证；其中 `remaining_ms` 是收到时的原始预算，不能以每次查询为起点重新计时。`correlate` 可再次核对描述；无配置的旧 BLE 路径仍只诊断。原生 `Service<Ble>` 保留扫描、诊断与安装任务调度，公共设备操作未变；本次没有为字节流伪造蓝牙扫描或 20 字节诊断。

## 记录、期限和失败

`RecordIo` 的 `send` 是顺序发送，`try_receive` 是非阻塞完整记录出队，`healthy` 与 `close` 描述该次连接。接收队列最多 4 条、每记录 1～1297 字节。记录仍可能是握手或密文，不能直接传给安装器。

`StreamRecords<S>` 接受已经连接的 `AsyncRead + AsyncWrite`，不监听公网、不创建连接或自动重连。帧为 2 字节大端长度和载荷；越界长度在读取载荷前拒绝。一个持续接收任务拥有读取半部，第一字节到达后整条记录有固定 5 秒期限，收到更多字节不延期。流断开、半包、非法长度或第 5 条排队记录使排队内容全部失效。发送最多 5 秒；取消后该连接不可复用。关闭／析构终止接收任务并释放本地流与队列；实际物理设备清理由原平台负责。

共同会话的整个握手含确认与就绪回执最多 10 秒。既有 6 秒安全活性与开发凭据固定期限保持，查询、发送和明文诊断不能续期。应用发送／保活在首个等待点之前禁止复用，只有完整成功才恢复；失败、超时或丢弃 Future 后不能继续使用已推进的加密序号或接纳迟到回复。接收失败关闭通道；重连必须建立新上下文和新会话。

保活期间允许暂存一条应用响应，直到收到合法心跳回复后交还上层；完全相同的明文副本不额外占槽，第二条不同消息拒绝。完整心跳最多 5 秒，重复消息不延长期限。安装响应的请求号、事务与提交核验仍属于既有传输模块。运行使用独立 `prepare_runtime`／`runtime_peer` 和 RuntimeClient，不从安装连接或播放能力隐式升级；不新增逐帧流或生产授权。

## 验证与限制

真实本机 TCP 和使用实际 GATT 分片编解码的 20 字节软件承载，都通过同一 `Channel` 完成工程包安装、保活插入、摘要／目录和落盘逐字节校验。故障测试覆盖取消、半包期限、溢出、错误密钥、旧回执、篡改／重放、重连及固定权限期限。原 BLE 通知／连接／安装回归继续执行。

DEVICE-003 第四增量在两种承载上补齐共享运行客户端及原工作器软件验证；第五增量已补[原生设备运行服务](native-device-runtime-service.md)，共用 GATT 记录代码但独立于旧安装端点。固件运行队列和桌面操作尚未接线，具体证据与限制见[运行客户端](device-runtime-client.md)。

软件字节流通过不等于 USB 驱动、以太网设备、跨平台、射频或真实 DMX 已验收；此次抽取没有操作实板，原有 [DEVICE-002 实板证据](../development/tasks/DEVICE-002-direct-installation-acceptance.md)保留为历史基线。外层通用端点目录、网络身份、设备角色和更多控制协议仍需各自增量。当前开发配置不等于商业方案，遵循 [ADR-101](../development/decisions/PRODUCT-ADR-101-commercial-security-boundaries.md)。
