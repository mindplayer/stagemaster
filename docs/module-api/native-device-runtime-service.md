# 原生设备运行服务

DEVICE-003 第五增量，依据 [ADR-131](../development/decisions/PRODUCT-ADR-131-native-device-runtime-service.md)。`Service<Ble>` 的原后台连接任务复用 [RuntimeClient](device-runtime-client.md)，安装与运行分别连接；不拥有工程、设备播放器、安装存储或物理输出。

## 调用与状态

现有 `request(Status／Scan／Connect／Cancel)` 保持。普通 Connect 仍走诊断／安装；新增 `connect_runtime` 是明确的原生 Rust 入口，不会偷偷提升安装会话。实例创建和快照读取不启动蓝牙。预期范围必须有观察权限；设备端仍须独立准入，安装凭据不因这些参数而增加权限。

```rust,ignore
let service = Service::new(Ble::with_development_configuration(trusted_config));
// 通过原扫描取得当前 epoch 和 discovered_id，完成清理后连接。
let connecting = service.connect_runtime(epoch, discovered_id, Access {
    observe: true, control: true, installation: false,
})?;
// 后台准备；调用方持续读取原连接状态和 runtime_snapshot。
let snapshot = service.runtime_snapshot(connecting.epoch)?;
if snapshot.peer.is_some() {
    let response = service.exchange_runtime(connecting.epoch, RuntimeIntent {
        operation: Operation::Status, expected_revision: 0,
    }).await?;
    // 之后目录和修改操作使用 response.observed.revision。
}
// 退出应用时有界清理；断开不会向设备发送停止或重播。
service.shutdown().await?;
```

调用方先查询状态，再显式请求控制权和选择／载入／执行等操作。连接成功只证明诊断就绪，`runtime_snapshot.peer` 才表示运行入口已完成匹配；它也不是物理发送证据。

| 方法／字段 | 职责 |
| --- | --- |
| `connect_runtime(epoch, id, expected)` | 复用本次发现句柄、物理连接与清理任务；和安装连接互斥，拒绝旧 epoch。 |
| `exchange_runtime(epoch, intent)` | 单个有界调用槽，串行发送／收取，返回完整关联回复；确认的业务失败保留在回复内。 |
| `runtime_snapshot(epoch)` | 当前准入和历史观测；故障断线后仍可读取，旧操作代次不能修改新连接。 |
| `connection_epoch` | 历史记录所属连接；后来搜索产生新操作 epoch，也不会把旧记录冒充新连接结果。 |
| `pending` | 开始发送前登记的意图；取消或失败后保留不确定性，不代表未执行。 |
| `last_response` | 上一次真实关联回复；等待新操作或断线后仍为历史值，不维持“在线”。 |

新连接清空前一连接的待确认意图／历史回复。仅搜索不丢弃这些历史记录，并保留其原连接 epoch。调用槽忙碌拒绝第二项；排队后尚未开始的调用被丢弃时可以撤回，发送开始后取消则释放连接。服务不会自动重发运行命令，也不会在重连后取得控制权。

## 时间、权限和适配

调用自入队起最多 30 秒，发送最多 2.5 秒；等待业务回复时约每 1.5 秒复用诊断与加密保活。原 4.5 秒诊断新鲜度、固定开发许可和共享客户端的安全／请求期限均有效；查询、心跳和排队不延长许可或请求。就绪在主机连接开始时间加设备给出的剩余预算处保守截止。状态查询也检查截止，避免后台调度暂停时显示过期准入。

Transport 新增运行连接、就绪、发送、待确认请求及非阻塞回复方法；默认不支持运行。实际 Ble 拥有同一个 RuntimeClient，接入原 GATT 分片／固定接收队列。适配器在发送开始后必须保留待确认请求；仍然在线且没有待确认请求的发送拒绝，才可视为发送前的本地拒绝。

### GATT 路由

诊断发现保持原服务。旧安装服务 `f889eda0-0100-4e83-968e-799ab99558fa` 在确认后主动打开安装工作器，故运行不能使用该入口：

| 运行服务／特征 | UUID | 属性 |
| --- | --- | --- |
| 服务 | `f889edb0-0100-4e83-968e-799ab99558fa` | 独立运行入口 |
| 请求 | `f889edb2-0100-4e83-968e-799ab99558fa` | 无响应写入 |
| 回复 | `f889edb3-0100-4e83-968e-799ab99558fa` | 通知 |

复用原安全分片与 SMRT v1，按服务 UUID 和特征 UUID 同时过滤；安装通知和外来特征不能消耗运行序号。缺少声明、服务、必要属性或可信身份时拒绝，不回退安装或明文，不创建系统配对。

## 验证范围

本轮原生服务测试使用真实共享客户端、安全会话、软件字节流和原安装包／ManagedWorker，核对目录到执行、断线续播与重连只读、旧 epoch、互斥、本地拒绝、业务失败、排队撤回、发送中取消、超时和历史保留。原生通知过滤另有安装／运行隔离专项；原安装和上传回归保持。

这些结果不等于实板或射频验收。现有固件尚未提供上述运行服务、生产运行队列或独立调度；桌面操作入口也尚未接入。此次未刷机、连接蓝牙或输出 DMX，完整 DEVICE-003／AUDIT-001 仍开放。
