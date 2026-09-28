# 主机节目安装任务接口

DEVICE-002D；依据 [ADR-040](../development/decisions/PRODUCT-ADR-040-host-installation-task.md)。`stagemaster-device-upload` 复用上传器、播放包及[主机消息层](host-installation-io.md)。任务不绑定工程会话、Tauri、界面、文件路径或 BLE 具体驱动；安装不触发播放。

## 调用和生命周期

```rust,ignore
let devices = Arc::new(device_host::Service::new(transport));
let tasks = device_upload::Service::new(devices.clone());
// 在阻塞工作队列校验有界、不可变的包。
let prepared = device_upload::Prepared::new(immutable_bytes)?;
let view = tasks.start(prepared, selected_epoch, expected_device_id)?;
let id = view.task.unwrap().id;
let progress = tasks.snapshot()?;
tasks.cancel(&id)?; // 保留意图，等待当前消息或重新连接后的查询
// 仅在任务需要恢复，且明确连接原设备后调用。
tasks.resume(&id, newly_authenticated_epoch)?;
tasks.shutdown().await?; // 只结束本机通信，不等于远端取消
```

`Prepared` 持有 `Arc<[u8]>`，全包严格校验后生成来源、摘要、节目数及最大参考装载预算。进行中的任务不再读取工程或生成缓存；重新编辑、关闭面板、切工作区不能改变源字节。任务由应用根服务持有，界面只订阅快照及提交意图。

`start` 要求没有未解决任务、当前连接代次／稳定设备准确、可信安装权限有效、版本和容量匹配。每次消息前重新核对权限和预算。任务 ID 随意图更新；所有操作须带当前任务 ID。一次应用仅有一个未解决任务，不能把错误设备或旧页面的操作用于新任务。

`Connection` 提供 `target(epoch)` 及异步 `exchange(epoch, frame)`。原生实现从既有连接服务读取权限；`Target` 不是 IPC 可反序列化授权。该服务独占连接的安装消息序号，不允许别的调用者并发发送原始安装消息。诊断保活仍由连接层调度。

## 状态和取消

| 状态 | 含义 |
| --- | --- |
| `Querying` | 查询权威状态、开始事务或对账 |
| `Transferring` | 发送分块；进度只按完整回执计数 |
| `Verifying` / `Committing` | 等待全包校验／持久提交 |
| `Cancelling` | 已有取消意图，尚未确认结果 |
| `Reconnect` | 传输或协议不确定，保留原包，须新认证连接 |
| `Failed` | 已知远端业务拒绝，可以显式重试同一连接 |
| `Installed` | 提交身份与原包完全一致；回执含代次／摘要／字节数 |
| `Cancelled` / `NotStarted` | 权威状态已确认取消／本意图未开始 |

只有最后三种是终态。历史完成记录不证明设备当前仍使用这个包。提交代次用十进制字符串跨 IPC，避免 JavaScript 整数精度丢失。

取消是粘性的任务意图。当前消息先完成；断线时保留，之后恢复查询。若实际已提交，仍报告已安装并说明取消未撤销结果。不能将取消按钮点击、调用超时、面板收起或断开连接当作取消成功。

`forget` 仅移除本机非运行任务。未解决任务必须在 UI 明确确认设备可能仍有事务／已提交内容；它不发送取消。退出应用对未解决任务同样提示。当前任务不跨进程持久化，也不自动重连；这些是明确的未完成能力。

## 同一连接的多个任务

请求序号属于认证会话。新的安装意图通过 `Upload::connect_at(session, next_id)` 继承已确认游标；只能在没有在途消息的正常边界读取 `next_request_id()`。发送不确定、异常退出或序号耗尽时游标为无效；即使移除任务也不能在原会话从 1 开始。新的认证会话才可以从 1 开始。

连续三个不同包在同一会话安装已经通过，持久代次为 1／2／3。协议仍严格递增，没有为修复主机错误放宽设备校验。

## 桌面接口与界面

- `installation_start(generation, token, epoch, deviceId)`：取得生成缓存不可变字节，在阻塞队列校验，再按播放包操作门→工程操作门顺序锁定，核对缓存标识及当前已应用工程，开始任务。原始认证会话及协议帧不暴露到 IPC。
- `installation_request({kind: status | cancel | resume | forget, ...})`：返回 `View { installation, destination }`。取消／恢复／移除带任务 ID；恢复另带连接代次。
- 任务与设备状态各有单调修订号，分别合并；迟到轮询不覆盖新状态。网页宿主明确拒绝设备安装。
- 工程“播放包”提供安装入口；顶部独立安装面板显示目标、已确认字节、阶段、恢复、取消和提交摘要。与设备面板互斥，关闭后后台任务继续。工程变化使旧生成结果失效，开始后任务独立运行。
- 前端等待超时不等于后台停止，随后重新读取状态；Rust 仍执行任务互斥和标识检查。

## 文件职责与验证边界

Rust 拆为 `package`、`connection`、`model`、`state`、`service`、`runner`；桌面拆为安装命令、目标摘要、退出提示和应用退出处理。界面拆为状态订阅、面板、任务操作、纯状态工具；播放包结果显示从原面板抽出，分页仍归原面板以保留上下文。

软件测试使用真实导出包、受限传输服务和文件双槽，覆盖取消、丢提交回执、错误设备、无权限、容量、并发意图、连续安装、退出和异常。组件隔离页面只在 tests 路径存在，不进入正式构建。原生当前固件仍只有诊断权限；不得把组件成功状态或文件存储测试当作真实无线安装。证据见 [D 任务层验收](../development/tasks/DEVICE-002D-task-workflow.md)。
