# 应用权限与加密安装会话

DEVICE-002；[ADR-050](../development/decisions/PRODUCT-ADR-050-application-installation-admission.md)。实现位于 `device-auth::application` 与 `install-worker::secure`，均通过 Cargo `application` 特性启用。旧系统绑定接口不变。

## 所有权

`DevelopmentPermit` 是可信本地配置，只用于受控开发设备；没有网络解码、云端来源标签或通用密码。它绑定稳定设备编号、控制端公钥、稳定主体、非零权限修订和 1～600,000 ms 的本次连接权限。主体可在未来由云端登记，不从蓝牙定位地址推导。

`Session` 消费相互确认完成的 Noise Channel，拥有其密码状态。准入核对完整 Context 和已证明持有的公钥；错误、许可期限、安全租约或撤销关闭当前状态。`Grant` 只有只读查询，没有公开字段构造／反序列化；它仍是瞬时快照，不能替代每次检查。

`Gateway` 消费 Session，拥有 Endpoint、一个固定密文缓冲及一个待回复心跳位置，不持有无线 SDK、存储、运行时或云端客户端。可信工作器完成与无线记录由外部单一连接所有者按序投递。新代码没有新增堆队列；密码实现仍有既有有限堆分配，不能称整个流程零分配。

## 调用顺序

下面仅表达组合关系，省略无线和错误处理；没有任意 JSON 转换为授权的入口：

```rust
let permit = DevelopmentPermit::installation(
    provisioned_device_id, trusted_controller_public_key,
    stable_principal, permission_revision, 600_000,
)?;
let access = Session::admit(confirmed_noise_channel, permit, local_context, now())?;
let (mut gateway, open) = Gateway::open(access, local_worker_epoch, now())?;
publish_live_epoch(gateway.live_epoch(now()));
enqueue_once(open)?;

// 每个定时轮询、收包、完成和发送前后都重检并同步有效代次。
gateway.complete(worker_completion, now())?;
if let Some(ciphertext) = gateway.outbound(now())? {
    // 只入队一次；分片 Sender 在平台背压下逐片发送。
    send_one_bounded_record(ciphertext).await?;
    gateway.sent(now())?;
}
if let Some(command) = gateway.receive(&complete_ciphertext, now())? {
    publish_live_epoch(gateway.live_epoch(now()));
    enqueue_once(command)?;
}
```

适配器需要遵守：

- 工作代次由设备本地单调计数产生，启动内不得复用；不要截断公开连接随机数生成代次。
- 准入成功只返回 Open，尚无就绪回执。工作器先确认维护与打开成功，Gateway 再加密就绪回执；回执完整发出前不能接受业务请求。
- 每条业务明文必须是一条完整 SMP 消息，最多 1280 B；半条 SMP、错误会话、重复工作或乱序失败关闭。安全记录 MTU 分片层独立检查序号和固定 5 秒期限。
- 保活由当前安全会话认证；公开诊断不能续此权限。工作排队期间可回应加密心跳；单一发送者按序处理就绪、业务回执和心跳回复，当前密文发送完成前不重新加密。第二个尚未可回复的心跳失败关闭，队列不增长。
- `outbound` 重读返回相同密文；只有整条安全记录被底层接受后才能 `sent`。接受不等于对端已收到，更不等于已安装成功；节目结果仍以 SMP 提交回执／重连对账为准。
- 队列失败、通知失败、取消、断线和 Drop 必须关闭 Gateway 并将独立 LIVE_EPOCH 清零。Gateway 不持有原子变量，析构不能替适配器清除硬件共享状态。即使没有连接，也要排空旧工作完成，防止工作器等待旧消费者。
- 所有调度使用新鲜单调时间；发送前后重新检查。Flash 已经开始的操作可以完成，但过期结果不得赋予新连接。硬件并发期限仍须实测。

## SMAP 就绪回执

固定 112 B，小端整数，只在已经确认的加密 Message 内接收；身份核对、解密和关联全部成功才允许桌面建立安装通道。不是网络可反序列化的设备授权对象。

| 偏移 | 长度 | 含义 |
| --- | --- | --- |
| 0 | 8 | `SMAP`、版本 1、开发准入配置 1、长度 112 |
| 8／24 | 各 16 | 稳定设备号／启动随机身份 |
| 40 | 8 | 本次诊断连接随机号 |
| 48／64 | 各 16 | Noise 会话摘要前 16 B／稳定权限主体 |
| 80 | 8 | 权限修订 |
| 88 | 4 | 安装范围，当前严格为 1 |
| 92／94 | 各 2 | 应用认证编号 2／SMP 版本 1 |
| 96／98 | 各 2 | 完整消息预算 1280／保留零 |
| 100／104 | 各 4 | 安全租约 6000 ms／生成时许可剩余毫秒 |
| 108 | 4 | 保留零 |

未知版本、权限位、身份、保留字节和不合法预算拒绝。`correlate` 核对完整身份／主体／修订且剩余期限不可超出预期；这仍不替代加密认证。网络延迟会消耗回执中的剩余期限，设备判断始终权威；主机不得据接收时间给设备延期。

## 云端边界与当前完成度

未来云端拥有设备登记、公钥、归属、权限及修订；新签名验证适配应在此边界验证签发者、设备受众、控制端公钥、范围、期限和撤销状态。现有 Gateway／Endpoint／工作器不应访问云端或系统配对记录。BLE peer UUID 仍只是本机定位提示。

软件组合已覆盖真实导出包、维护门、存储恢复、取消、断线恢复、提交回执丢失及权限拒绝。后续 ADR-051 已将[专用开发配置](development-gatt-configuration.md)接入真实 GATT／桌面和 NOR，见[实板与桌面验收](../development/tasks/DEVICE-002-direct-installation-acceptance.md)。云端认领和 24 小时文件许可仍未实现；10 分钟开发连接许可不能替代文件播放规则。
