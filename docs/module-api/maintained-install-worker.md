# 受维护状态约束的安装工作器

DEVICE-002C；依据 [ADR-041](../development/decisions/PRODUCT-ADR-041-maintained-install-worker.md)。`stagemaster-install-worker::ManagedWorker<S, P>` 组合既有 Worker 与 [Runtime](device-runtime.md)，在同一个存储执行器内串行工作，不包含认证、UI、无线或物理输出驱动。

## 生命周期

```rust,ignore
let (installer, recovery) = Installer::open(storage, boot_id)?;
let mut device = ManagedWorker::new(installer, now_ms, loader_budget, policy)?;
// 输出宿主实际停发并清空旧队列；不能直接使用对端声称的 stopped 字段。
let quiet = device.quiescence_request().ok_or(NotQuiescing)?;
device.confirm_quiescent(quiet, now_ms)?;
// 认证适配产生内部 Command；无线数据本身不能构造授权事实。
let completion = device.process(command, now_ms, || live_epoch());
// 事务终结、无旧运行读源后，恢复目录；不自动选择或执行。
let state = device.finish_maintenance(now_ms)?;
```

启动身份取自 Installer，避免同一设备应用使用两个互不对应的 boot。初态 Quiescing，尚无可写窗口。`confirm_quiescent` 复用精确的 Runtime 请求／令牌，释放旧计划、目录和读源租约；旧确认、取消后的确认、时钟倒退均拒绝。

`process` 每次推进可信单调时间，随后必须在当前 `with_maintenance` 同步闭包内调用 Worker。Operation／Quiescing、失效令牌或时钟错误不执行存储；返回 `Error::Maintenance(Code)`，并撤销当前工作连接。不会隐式停止、接管或释放正在播放的节目。

工作连接撤销仍有前后双检查。已经开始的 Flash 操作可能完成，但撤销后的结果不能进入另一连接。维护不能改变这一现实或把失联误判为回滚。

## 结束维护与恢复

- Worker 的受限服务提供只读 `progress()`，不产生存储 I/O。没有事务，或事务已经 Committed／Cancelled，才允许结束维护。
- Receiving、Verified、Failed 和 Uncertain 均阻止结束。需要按原归属完成、取消或核对事务；断线不会自行释放此约束。
- `finish_maintenance` 重新打开并校验已提交快照；目录预算／读取／完整性失败保留维护，可以显式重试。空存储可以恢复为空设备。
- 成功仅绑定目录，所选节目／已载入计划／实例为空；输出帧不可用，不自动播放。旧工作连接撤销，下一个窗口需要更新的连接代次。
- `snapshot` 仅用于同执行器的只读检查；外部若保留快照，底层存储租约仍可能拒绝槽复用，不能绕过此保护。

控制租约、选择、载入、执行、停止、BeginMaintenance／CancelMaintenance、历史回执、许可策略及帧生成均委托既有 Runtime。组合层不公开可变 Worker 或 Runtime，避免绕过上述门槛；不增加第二套运行状态。

## ESP32 适配

`installation::start` 要求主任务持有 `OutputDisabled`；当前镜像 GPIO21 持续低电平，完全没有真实发送任务，因此启动后可确认静默，整个存储队列通过 ManagedWorker。播放策略始终拒绝执行；公开描述仍只有诊断能力，维护准备不授予无线安装权限。

`worker-write-test` 的本地探针额外验证：真实包安装及逐帧核对→退出维护并绑定目录→运行模式拒绝安装→明确再次请求维护→确认并释放目录。该探针的固定本地主体只在编译测试路径存在，不能进入正式 GATT 授权。

文件分工：`managed.rs` 管维护和安装生命周期；`managed/control.rs` 仅委托运行操作；固件 `installation/runtime.rs` 定义当前拒绝播放策略；`worker_probe/maintenance.rs` 仅为本地验收。物理输出实际停止确认、正式远程维护控制与无线认证仍待独立接入。

验收、资源和失败记录见[维护集成验收](../development/tasks/DEVICE-002C-maintenance-acceptance.md)。
