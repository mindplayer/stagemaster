# 安装工作模块

DEVICE-002C；依据 [ADR-037](../development/decisions/PRODUCT-ADR-037-installation-worker.md)。`crates/stagemaster-install-worker` 提供同步、无调度器依赖的 Rust 边界，具体存储 I/O 仍由既有 `Storage` 完成。它不解析工程、不持有蓝牙句柄、不负责认证或生产许可。

无线回调到工作器之间的半包／单请求／回执分片与固定期限由独立 [Endpoint 字节通道](installation-byte-channel.md) 承接。它和工作器位于同一 crate，但各自拥有状态、没有互相调用 I/O；适配器负责连接两端及队列外撤销。

## 调用与所有权

```rust,ignore
let installer = Installer::open(store, boot_id)?.0;
let mut worker = Worker::new(installer)?; // 留在同一工作任务，store 不要求 Send

// 认证适配器分配单调不回绕的非零 epoch；对端不能指定。
// live_epoch 的撤销必须独立于有界消息队列，断线即更新。
let opened = worker.process(Command::Open { epoch, link: authorized_link }, live_epoch);
let completed = worker.process(Command::Frame { epoch, frame }, live_epoch);
worker.observe(live_epoch()); // 空闲时也处理撤销
```

`AuthorizedLink` 必须来自已验证的主体及当前业务连接。非零值、蓝牙名称、设备描述或诊断握手都不是授权依据；这个内部队列没有字节反序列化入口。安装适配器只能传入已完成有界重组的 `Frame`，不能把未校验的碎片直接拼进安装状态。

工作模块独占 `Service<Storage>`。读源仍按既有存储租约规则返回；`snapshot()` 是同一所有者内供运行模块调用的可信接口，不是任意网络读取能力。Flash、`Rc`、`RefCell`、旧包快照不能跨核传递。

## 断线、错误与队列

- 开始处理前读取最新存活代号；过期 `Open`／`Frame` 返回 `Obsolete`，不执行其存储操作，也不破坏当前新连接。
- `Open` 消耗代号，即使授权形状错误也不允许重用；协议错误撤销活动连接，再次授权须使用新代号。
- 当前连接没有完成 `Open` 时返回 `NotOpen`。普通帧永远不自行建立权限。
- 开始的同步请求可能在断线后完成，完成后再次核对代号；过期回执返回 `Obsolete`。安装事务和已提交结果不会伪装回滚。
- 无线发送器在通知前再次检查代号，丢弃迟到回执；持续消费／清理旧回执，不能让断开的接收方永久堵住有界队列。
- 重连须重新认证、建立新代号，通过现有传输层查询权威状态。丢失“提交成功”回执后请求取消，若实际已提交，应报告已安装。
- 传输层的事务归属、错误码、重复请求和取消／对账规则不变；本模块不引入第二份安装状态。

## ESP32 候选适配与验收界限

`apps/esp32-player/src/installation.rs` 在第二核创建官方分区驱动及存储，32 KiB 独立静态栈，容量各为 1 的请求／回执通道。`measured_nor.rs` 记录实际读／写／擦时间及次数，调用官方多核停放；物理写／擦后给其他任务调度机会。栈采样仅记录调用 NOR 前的当前深度，不包含更深 ROM 调用，不能当作完整高水位。

`worker-readiness` 只读；`worker-write-test` 是显式单独编译的本地存储试验，测试包路径须指定到项目 data 内。测试主程序始终持有 GPIO21 低电平，未加载现场播放器、未连接 GATT 安装入口。测试中的固定主体是内部夹具，绝不能用于无线认证，也不对设备描述宣称安装能力。

软件四项保护测试已验证旧队列、错误连接、准备中断线的归属保留，以及查询／开始／写入／校验／提交各边界丢回执后的重连对账。实板已完成真实小包／舞台场景包的独立本地安装、Flash 重放逐帧对照、同包重启零擦写及分配器峰值测量；另有压力轮 BLE 断开，不能认定无线稳定性出口通过。详见 [实板验收](../development/tasks/DEVICE-002C-worker-acceptance.md)，不能将本地受控测试替代已认证 GATT／桌面安装。
