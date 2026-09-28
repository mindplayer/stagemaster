# 主机安装消息接口

依据 [ADR-039](../development/decisions/PRODUCT-ADR-039-host-install-io.md)。位于 `stagemaster-device-host`，与[设备连接](device-connection.md)、[上传协调](package-transfer.md)配合。只负责一条安装消息的传输，不拥有工程、安装事务或设备存储，也不触发播放。

## 所有权与调用

上层任务和同一会话的连续安装序号由[主机安装任务层](host-installation-task.md)持有；下面仅展示新连接的单任务消息顺序。

一个 `Service<Transport>` 拥有一个连接任务。诊断握手、保活和安装片段顺序经过同一个适配器；上传模块不再尝试锁住适配器或另开连接。入口是原生 Rust API，未增加原始消息的 Tauri IPC。

```rust,ignore
let epoch = devices.request(Request::Status)?.epoch;
let peer = devices.installation_peer(epoch)?.ok_or(NeedsAuthentication)?;
// Upload 的 source 为只读的不可变包；业务组件维护其生命周期。
upload.connect(peer.session)?;
while let Some(frame) = upload.outbound()?.cloned() {
    let response = devices.exchange_installation(epoch, frame).await?;
    upload.accept(response.bytes())?;
}
// 从 upload.outcome() 读取安装／取消结果；不得根据片段写成功生成结果。
```

上例为正常调用顺序，完整业务组件还须处理取消意图、目标设备核对和错误后的显式重连。`exchange_installation` 错误或调用 Future 被丢弃后，调用 `Upload::disconnect`，重新识别同一设备、取得新认证会话，再调用 `Upload::connect` 查询状态。上层不能把失败解释为“未安装”，也不能改向另一设备恢复原意图。

## 权限事实

`Transport::installation_peer() -> Option<InstallationPeer>` 仅由可信原生认证适配返回。结构含设备、启动、业务会话、认证机制、片段上限和消息上限，不实现网络反序列化。适配在发放前完成认证及有界通知订阅；授权撤销立即撤掉事实，断开清理队列。

服务将事实与当前已关联的描述核对：稳定设备／启动相同、非零认证方法一致、实际声明安装能力、传输版本受支持、消息预算一致且不超过 1280 字节，片段在 1～消息上限内。业务会话必须非零。声明只用于拒绝矛盾信息，不能生成事实；中文显示标签不参与授权判断。现有原生 `Ble` 使用默认返回 `None`，未实现正式安装权限。

## 消息和时序

- 每个连接仅一个应用请求，包含排队中及执行中；第二个返回“忙”。连接代次、业务会话与请求格式在入队前检查。
- `write_installation` 只发送一个片段，失败或超时表示不确定交付；不自动重发。
- `try_installation_notification` 是非阻塞、取消安全的有界队列出队；适配必须报告溢出、断连及认证撤销，不静默丢字节。默认无权限适配返回空。
- 复用 `Assembler` 重组完整响应，严格匹配业务会话、完整请求序号、命令及启动身份；远端业务错误作为完整响应交给 `Upload`。超长、空片段、额外通知和错序关闭连接。
- 空闲约 2 秒保活；安装期间回执年龄达到 1.5 秒优先保活。一片写入最多 500 毫秒；整帧发送最多 5 秒。发送完后等工作结果最多 30 秒；第一片响应到达后，整份响应最多 5 秒。这些期限不因更多片段或保活延长。
- 保活还受距上次确认 4.5 秒的剩余预算限制；期限后完成的诊断不得重新发布在线。设备原 6 秒规则不变。
- 丢弃尚未发送的调用只清除排队请求；丢弃已开始的调用关闭连接。用户“取消安装”应由业务层设置 `Upload::request_cancel` 并继续完成当前消息，再发送事务取消；不能将“取消连接”等同事务已取消。

错误或主动断开立即清除服务中的安装权限和当前队列；清理不确定时沿用 `blocked` 规则，不能复用适配器。安装事务保留及提交核验仍由设备安装模块和上传模块负责。

## 文件职责

- `service.rs`：应用所有者、命令准入、退出清理。
- `service/state.rs`、`scan.rs`、`diagnostics.rs`：连接状态、发现、握手／保活。
- `service/connected.rs`：单任务 I/O 调度。
- `service/installation.rs`：原生消息入口、单请求占用与回复。
- `service/installation/pending.rs`：分片、重组、消息期限、响应关联。
- `installation_peer.rs`：可信事实与能力核对。

本轮未接真实安装特征、系统绑定、固件维护入口或桌面安装进度界面；软件夹具显式提供测试权限，仅在测试构建中使用。验收证据见 [主机调度验收](../development/tasks/DEVICE-002D-host-io-acceptance.md)。
