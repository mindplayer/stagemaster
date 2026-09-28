# 安全记录字节通道

DEVICE-002；`stagemaster-device-link::secure`，依据 [ADR-048](../development/decisions/PRODUCT-ADR-048-bounded-secure-record-framing.md)。此模块已实现无堆收发与故障保护；[实验 GATT](../development/tasks/DEVICE-002-secure-gatt-acceptance.md)已实板通过，正式授权与安装适配仍待实施。

`Sender`／`Receiver` 各属于一条物理连接的单一方向，独立持有序号、固定缓冲与期限，不依赖加密库、蓝牙 SDK、云端或安装模块。传递的是完整不透明握手／密文，不能凭 `Record` 授予权限。

```rust
let mut tx = Sender::new(gatt_value_bytes, now_ms)?; // 20～244 B，含 4 B 序号
let mut rx = Receiver::new(gatt_value_bytes, now_ms)?;
tx.queue(&complete_noise_message, now_ms)?;
while let Some(packet) = tx.fragment(now_ms)? {
    transport.write(packet.bytes()).await?;
    tx.sent(current_ms)?; // I/O 成功后确认，失败／取消则关闭整条连接
}
if let Some(record) = rx.push(&notification, current_ms)? {
    // 按当前阶段交给 Handshake::read、Channel::confirm 或 Channel::open。
    // 通过认证后再核对业务权限，不能直接当安装请求。
}
tx.poll(current_ms)?;
rx.poll(current_ms)?;
```

上面为省略错误清理的调用形状。真实连接任务必须定时轮询，即使没有收到新数据；任一错误使方向永久失效，适配器同时清理另一方向、Noise 状态、权限、旧完成并断开。取消不能沿用旧实例。连续调用 `fragment` 返回同一待发片段，重复调用 `sent`、未读取片段就确认、已有待发时再次 `queue` 都关闭该方向。

首部是版本 1／保留 0／消息长度 u16 大端，随后 1～1297 B 消息；每片附现有 Serial u32 小端序号，从 1 开始跨记录连续。每方向最多一条 1301 B 缓冲。发送从入队、接收从首片起固定 5 秒，不接受续期。完整记录之间可以空闲，活跃连接由上层握手／加密保活约束。

收发完成只表示结构完整或传输提交，并不表示消息已认证、工作器已完成或节目已安装。MTU／片段序号／首部都是不可信输入；认证依据始终是[独立安全会话](device-session.md)和授权适配。真实安装依旧通过 Endpoint、ManagedWorker、存储封印与摘要回执核验。

软件组合验证包含 20／244 B 两种预算下的完整 Noise 握手、确认、每种 256 轮最大消息＋加密回复及合法分片内的认证损坏拒绝。此为软件字节通道验证，不是无线链路吞吐或实板栈使用量证明。
