# 设备运行队列与独立有效性

DEVICE-003 第六增量，依据 [ADR-132](../development/decisions/PRODUCT-ADR-132-runtime-worker-queue.md)。`stagemaster-install-worker::runtime_queue` 随 `application` 提供，共用原 [运行应用入口](device-runtime-application.md) 和 [SMRT v1](device-runtime-wire.md)。本层没有平台线程、蓝牙驱动或物理输出。

## 对象归属

| 对象 | 所属任务 | 作用 |
| --- | --- | --- |
| `Gateway` | 通信任务 | 持有真实 Session，解密、校验、保活，生成单个待执行命令与不可变发送记录 |
| `Live` 最新值槽 | 平台同步设施 | 发布真实 Session 和网关期限的交集，独立于命令／完成队列；关闭时清空 |
| `Endpoint` | 原存储／运行工作器 | 跨连接保留打开代次，持有一个原 Connection；复用 ManagedWorker 执行 |
| `Command`／`Completion` 各一格队列 | 平台适配 | 传递本地 epoch／票号关联的有界工作与结果，不作为授权来源 |

不允许同时保留第二个运行 Endpoint 操作同一设备来重置历史。同一活动连接只创建一次 Gateway，连接 epoch 由平台单调产生、不能复用；耗尽后拒绝新连接。Command 只由 Gateway 构造，包含类型化 Offer／Request、票号、固定截止时间；没有从网络恢复 Grant 或 Live 的入口。

## 调用顺序

通信任务完成原安全握手及独立运行准入后，调用 `Gateway::new(session, epoch, now)`；至少需要观察权限。连接不会进入维护、取得控制权或执行节目。

1. `receive(cipher, now)` 解密一个完整记录，返回零或一个命令。平台向单格命令队列非阻塞入队，失败必须关闭连接，不能丢弃后继续显示成功。
2. `outbound(now)` 返回当前待发送密文；直到全部分片被承载接受后调用 `sent(now)`。此前重复调用返回相同字节，不再次推进密文序号。
3. `complete(completion, now)` 接收原工作器结果；检查当前权限、epoch、票号及完整关联后准备回复。旧代次／旧票号返回 false，不能完成当前请求。
4. 每个上述调用及空闲轮询后，把 `gateway.live(now)` 的完整 Option 写入独立同步槽，包含 None。错误、断开、取消和适配对象 Drop 都必须清空；单独丢弃 Gateway 无法替平台清空共享槽。

工作器始终持有原 ManagedWorker，按独立单调调度调用：

```rust,ignore
// read_live 每次重新读取平台的同步槽；不能捕获入队时的快照。
endpoint.tick(&mut worker, now(), &mut read_live)?;
let completion = endpoint.process(&mut worker, command, &mut now, &mut read_live);
// 完成队列拥塞时保留最多一份结果并继续 tick；不得阻塞推进等待通信消费。
```

平台槽应以锁／临界区原子发布整个 Live；不能分别更新 epoch、Grant 和期限而产生混合状态。工作器调用 `Live::grant(now)` 检查当前时间不早于本连接建立时间、严格早于截止点；不同 epoch 拒绝。时间下界固定为连接建立时刻，不使用每次发布时刻，避免工作器采样时间后恰好收到更新的心跳快照而误判撤销；通信侧和原 Connection 分别拒绝自身计时回退。关闭只归还本连接输入权，旧命令不能释放新连接的租约。工作开始后的撤销不保证回滚，桌面必须保留未确认意图并重连查询。

## 期限、容量与限制

`Channel::valid_until(now)` 返回当前已确认安全会话的接收保活截止时间；`Session::valid_until(now)` 再与固定应用权限取较早值。查询和发送不延期，只有有效认证接收按原协议更新接收期限。Gateway 的 Live 还包含自身协商、工作、心跳和发送期限，因此即使通信任务暂时没有再次轮询，旧发布值也不能跨过精确截止点授权。

协商总等待 5 秒，普通命令工作 30 秒，结果等待发送 5 秒，保活回复 5 秒；各阶段期限固定，重复请求和心跳不重新起算。Gateway 最多一个待完成命令、一帧回复、一份在途密文、一个待回复心跳；同一未完成请求的完整重试不产生新工作。已回复后的重试进入原 Connection 单份历史机制，返回原结果而不再次执行。同号变更／错序请求失败关闭，不扩展无限队列。

保活可在工作留置期间发送，但不能插入已经开始分片的密文。旧未消费回复不能无限占用槽。协商回复完成时按实际时间缩减剩余权限；客户端仍按自身保守起点校验，并以设备端权限为最终约束。

新层无分配器依赖；宿主布局检查约束 Gateway ≤4 KiB、Endpoint ≤2 KiB、Command ≤256 B、Completion ≤1.5 KiB。该上限是类型容量保护，不等于 ESP32 整体内存预算或真实栈峰值；固件需另外统计静态槽、已有安全会话堆、任务栈及包缓存。

真实安全会话、原安装器／包／播放器、完整 512 通道软件帧、实际载入中撤销、独立心跳与精确超时均有验证，见 [DEVICE-003](../development/tasks/DEVICE-003-remote-runtime.md)。当前固件尚未接运行服务、同步槽及持续调度，桌面页面和物理输出仍未接通；本层编译通过不能代替这些出口。
