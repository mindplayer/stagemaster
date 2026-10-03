# 共享设备运行客户端

DEVICE-003 第四增量，依据 [ADR-130](../development/decisions/PRODUCT-ADR-130-runtime-client-channel.md)。`stagemaster-device-channel::runtime::RuntimeClient<R>` 在既有 `RecordIo`／安全 `Channel` 上调用 [SMRT v1](device-runtime-wire.md)，设备端仍复用原 [ManagedWorker 运行入口](device-runtime-application.md)。不拥有发现、物理连接、播放器或后台调度。

## 准备与调用

平台先建立记录承载、取得当前描述；可信配置由本机受控入口提供。描述必须显式包含 `RUNTIME_APPLICATION`，单有播放能力不能推导运行端点。`Channel::prepare_runtime` 完成共同安全握手，再核对独立运行就绪；安装的 `peer()` 和运行的 `runtime_peer()` 分开，不能交叉使用。

```rust,ignore
let channel = Channel::prepare_runtime(records, description, &trusted_config, expected_access).await?;
let mut client = RuntimeClient::new(channel)?;
client.send(Operation::Status, 0).await?;
// 宿主串行轮询；没有回复时继续处理自身调度，并按既有周期保活。
if let Some(response) = client.receive()? {
    // 关联已通过；仍须检查业务 result，保留 observation.revision。
    accept_response(response);
}
// 宿主独立安排；并非每次 receive 后都发送心跳。
client.heartbeat().await?;
```

`expected_access` 是可信的范围预期，不能令设备授予权限；服务端仍须显式 scoped 准入。旧 SMDV v1 配置不升级，旧安装端点和回执不能用于运行。`RuntimeClient::new` 只接受尚未发送应用请求、没有待取消息的有效运行 Channel，防止重置同一会话请求序号。

| 方法 | 语义 |
| --- | --- |
| `send(operation, expected_revision)` | 发送一项明确意图；返回请求标识不等于执行成功。目录／步骤和修改采用已观察修订；状态读取不要求当前修订。 |
| `receive()` | 非阻塞地消费至多一条消息，严格核对完整请求及启动标识；业务失败作为正常回复返回。 |
| `retry_pending()` | 仅在原连接有效且存在待确认请求时显式重发完整原请求；不创建新意图或延长原期限。 |
| `peer()` | 当前可用运行连接的就绪事实；剩余开发许可是原始观测，不能重新起算。 |
| `pending()`／`last_response()` | 分别保留未确认意图和最近一份历史回复；都不能证明设备当前在线、已经执行或正在物理发送。 |
| `heartbeat()`／`close()` | 保活或关闭当前输入；不取得控制权，不自动停止设备节目。 |

一次只保留一个待确认请求和一份历史回复。忙碌、缺少本地范围或无法编码的新请求不消耗序号；真正发送前先记录意图，发送失败或取消后仍能展示“不确定是否执行”。关闭或重连后不能自动重放它；新会话先查询实际状态，再由显式操作取得控制权。

## 期限与重复回复

握手总期限 10 秒，安全活性期限 6 秒，完整心跳往返最多 5 秒；等待业务回复最多 30 秒，从首次发送开始计算。重试和心跳均不延长业务期限或开发许可。客户端不创建后台计时线程，宿主负责持续调用；期限到达后 `peer()` 不再返回有效连接，后续操作关闭旧会话。

心跳期间最多暂存一份不同的应用明文；完全相同的回复副本不增加槽，不能延长总等待。收取时最近一份完整历史回复的重复副本被忽略，不用于完成新请求；其他无对应请求、错误关联或格式异常关闭连接。密文依旧使用严格的新序号；此规则不允许重放密文，也不承诺在任意丢失密文后继续原连接。

## 验证与产品边界

真实本机 TCP 和 20 字节 GATT 软件分片经过同一客户端、安全会话、真实安装包与原工作器，覆盖目录／选择／载入／执行／暂停／继续／下一步／停止。断线后原实例继续，并将完整 512 通道软件帧与独立 Player 对照；重连只读查询不抢权。显式重试测试在回复加密发送前留置结果，验证历史回执和控制权不重执行；错误／取消、权限拒绝、迟到回复及固定期限另有专项。

第四增量 155 项相关 Rust 回归通过（含 16 项新增），全工作区全部目标严格 Clippy（application）及 Xtensa application-gatt 检查通过。测试服务端是软件验证工具；第五增量已接[原生设备服务](native-device-runtime-service.md)，生产无线队列、固件独立调度及桌面操作尚待接线。本次未刷机、连接实板或发送 DMX，不能据此显示“设备已出光”。
