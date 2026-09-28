# 安装字节通道接口

DEVICE-002C；依据 [ADR-038](../development/decisions/PRODUCT-ADR-038-installation-byte-channel.md)，实现位于 `stagemaster-install-worker::Endpoint`。它不持有设备句柄、Flash、工程或密钥；只是既有分片重组、工作器请求及回执之间的有界适配。

## 调用方式

```rust,ignore
// grant 已由独立认证适配确认；epoch 由连接所有者单调分配。
let (mut endpoint, open) = Endpoint::open(epoch, grant, payload_bytes, now_ms)?;
publish_live_epoch(endpoint.live_epoch());
enqueue_or_close(open); // 有界队列，不可无限等待无线回调

// 队列中的任何完成都要消费；旧 epoch 返回 false，不进入新发送队列。
let accepted = endpoint.complete(completion, now_ms);
publish_live_epoch(endpoint.live_epoch()); // 错误须先撤销，再传播错误／断开
accepted?;

// 每个已认证、有序传入的片段。请求完整后才返回一条工作命令。
let received = endpoint.receive(fragment, now_ms);
publish_live_epoch(endpoint.live_epoch());
if let Some(command) = received? { enqueue_or_close(command); }

// 已有完整回复才返回 Some；底层确认成功前重复调用返回同一片段。
match endpoint.fragment(now_ms) {
    Ok(Some(bytes)) => {
        if let Err(error) = send_with_deadline(bytes).await {
            endpoint.close();
            publish_live_epoch(endpoint.live_epoch());
            return Err(error.into());
        }
        let sent = endpoint.sent(fresh_now_ms);
        publish_live_epoch(endpoint.live_epoch());
        sent?; // 最后一片后才能接收下一条请求
    }
    Ok(None) => {},
    Err(error) => {
        publish_live_epoch(endpoint.live_epoch());
        return Err(error.into());
    }
}
```

示例中的队列／发送／撤销由适配器实现，不是新增的公共 API。底层发送失败、无法入队、连接断开、订阅失效、诊断过期、认证撤销、控制路径取消或宿主销毁时都须 `close()` 并立即清除工作器的存活代次；仍持续消费旧完成。不能仅丢弃 Future 而保留可写授权。

## 约束

- `open` 仅检查内部授权参数的形状，不做密码验证；名称、MAC、描述、公开随机数、诊断握手或构造非零字节均不能作为授权。
- 每连接指定 1–1280 字节片段上限，GATT 适配还须根据真实协商 MTU 减去协议开销和特征上限；1 字节用于验证头部可拆分，不是建议的 BLE 配置。
- 接收完整请求后处于 `Working`，完整回执处于 `Sending`。这两阶段再收到新片段会关闭连接，不偷偷排入第二个请求。一次传输只有一个应用请求在途。
- 请求会话和回执的会话／完整序号／命令必须匹配；具体幂等、事务所有权及取消／提交结果仍由 `Service`／`Upload` 维护。
- `fragment()` 不推进发送；`sent()` 只确认当前片段已被底层接受，不意味着用户电脑已经收到，更不代表安装成功。后续丢包或失联仍需重连查询权威状态。
- `poll()` 由适配器定时调用。打开、半包、整份回执各 5 秒；工作器完成 30 秒。重复片段／轮询／发送不续期；时间倒退和期限计算溢出关闭。空闲接收依赖外层保活，不能以端点仍存在显示设备在线。
- 关闭、超时或任何同代次错误后不可重新打开原对象；新认证连接创建新端点和新代次。异代次完成直接丢弃，不破坏当前端点。
- 该层没有播放器或输出权限。安装仍须静默维护，不因第二核承接就支持边播边写。

## 验证入口

`crates/stagemaster-install-worker/tests/endpoint.rs` 使用真实导出包、安装工作器和 FileStore，在 1／20／244／1280 字节片段下持久安装并重开，另覆盖背压、无打开回执、错误会话／完成、固定期限、时钟异常，以及提交回执半包丢失后取消应报告已安装。

ESP32 的 `worker_probe` 本地测试入口已复用该端点和实际双核队列，20 字节状态查询／244 字节安装请求与回执均经过同一规则；`worker_probe.py --expect-byte-channel` 检查真实目标端点大小与分片计数。这是本地字节通道与 Flash 验收；同时另测诊断 BLE 保活，**不能表述为节目已经通过蓝牙传输**。正式 GATT 入口仍须认证适配与实板验收。
