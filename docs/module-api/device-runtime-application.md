# 设备运行连接的应用入口

DEVICE-003 第二增量；依据 [ADR-128](../development/decisions/PRODUCT-ADR-128-device-runtime-application.md)。`stagemaster-install-worker::operations` 由 `application` 特性提供，借用原 [ManagedWorker](maintained-install-worker.md) 和 [Runtime](device-runtime.md)，不另建播放器。本文描述已实现的类型化调用边界，网络协议、无线任务队列和桌面操作仍待接通。

## 所有者与调用顺序

存储与运行所属的串行工作器持有 `ManagedWorker`；每个已准入的安全会话只建立一个 `Connection`。同一会话不能重新构造 Connection 来重置序号。认证 Session 和设备单调时钟由可信平台适配持有，远端不能自行提供 Grant、时钟或租约。撤销后的重连须重新准入，再读取当前状态。

```rust,ignore
// clock 来自设备单调时钟；session 是实际完成认证并准入的当前 Session。
let mut connection = Connection::open(&worker, clock(), |now| {
    session.grant(now).ok()
})?;

// 在同一个运行／存储所有者内串行调用。请求身份须先与认证连接关联。
let reply = connection.process(&mut worker, request, &mut clock, |now| {
    session.grant(now).ok()
})?;

// 空闲时也查询撤销与过期；Runtime 自身另有独立推进，不能依赖收消息驱动。
connection.poll(&mut worker, clock(), |now| session.grant(now).ok())?;
// 断线／取消时显式调用；归还输入权，节目继续。
connection.close(&mut worker)?;
```

`live` 闭包每次必须从现行 Session 或等效持续校验的撤销通道取事实，不得返回长期缓存的 Grant。打开、每次处理前后与空闲轮询检查启动上下文、会话、权限修订、范围与开发期限。跨核队列的适配仍需证明授权撤销可及时到达、排队消息不会复用旧代次；上述单线程调用不是固件接线证明。

打开只验证身份，不取得控制权、不载入、不进入维护。关闭只释放自身仍持有的租约，不能释放已接管的新控制者，也不等于停止节目；`Drop` 无法借用工作器，平台必须显式 close。若平台漏掉关闭，原 Runtime 的有界租约到期仍生效，不能宣称即时归还。

## 操作与授权

| 调用 | 必要范围与行为 |
| --- | --- |
| `Status` | Observe；取得当前软件状态与目录／步骤数量 |
| `Catalog { index }`、`Step { index }` | Observe；在指定业务修订下逐条读取 |
| `Acquire`、`Renew`、`Release` | Control；使用本连接主体、原 Runtime 租约，接管必须显式 |
| `Apply(Action)` | Control；原选择、载入、执行、暂停、继续、下一步、停止等语义 |
| 进入／取消／结束维护 | Control 与 Installation；仍经原维护门和输出宿主静默确认 |

不存在远端“确认已静默”操作。网络客户端不能传入其他控制者的租约。租约时长不得超过核心上限或当前开发许可剩余时间；操作范围不代替 Runtime 的节目播放许可，也不是商业授权方案。完整范围约定见[设备操作范围](device-operation-permissions.md)。

## 请求、重试与状态

`Request` 包含会话 ID、连续非零请求序号、期望业务修订和操作；序号从 1 开始，一次处理一个请求。只有 Status 不要求当前修订。目录与步骤也校验修订，避免读取到不同版本的混合数据。

连接仅保留最后一个完整请求／回执。完全相同的最后请求重试返回历史回执，不重新载入、执行或续期。同号不同内容或跳号关闭此输入连接；错误会话的误投请求只返回身份错误，不驱逐当前控制者。业务拒绝也消费序号；修正意图须发新序号。需要新状态时发新的 Status，不能把缓存回执当实时状态。

`Reply` 包含请求、成功明细或业务失败、处理时的软件状态及目录／步骤数量。`Detail` 一次只返回状态、一条节目或一条步骤；索引等于数量返回 None，超过数量拒绝。文本按现行包的 512 字节上限完整复制，不借用以后会释放的目录；不会静默截断名称。旧回执的名称和状态始终表示原请求时点。

当前 Reply 是内部 Rust 值，包含固定容量文本和完整状态，**不是冻结的网络字节格式，也未声称可装入单个安全记录**。后续编解码必须明确版本、报文预算和必要的分页，不能直接复制结构体内存，也不能把内部槽数量当固件整体内存预算。

## 失败与验证边界

载入等存储操作前后都重查授权。若工作开始后撤销或到期，存储操作可能已经完成；后置检查拒绝交付旧结果并归还输入权，不声称回滚、不自动重试。接纳、已载入、运行实例、软件采样和物理发送仍是不同事实。

真实安装包、Noise／Session、ManagedWorker 与原 Runtime 已验证自主运行、控制接管、历史重试、失效隔离、维护及分页；完整灯值与独立 Player 对照。软件证据详见[工单](../development/tasks/DEVICE-003-remote-runtime.md#第二增量运行应用入口)。正式运行就绪／编解码、承载队列、固件独立调度、桌面入口和实际 DMX 输出仍是后续出口。
