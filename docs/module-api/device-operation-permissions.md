# 设备操作范围

DEVICE-003／[ADR-127](../development/decisions/PRODUCT-ADR-127-device-operation-permissions.md)。实现沿用 `stagemaster-device-auth::application`，无新增运行依赖或字节格式。它决定本次连接可以做哪些操作，不代表设备已经实现相应能力，也不替代 Runtime 的控制租约和节目播放许可。

## 可信宿主调用

```rust,ignore
// 来自可信本地配置，不能将网络声明的范围直接当授权。
let permissions = Permissions::only(Scope::Observe).with(Scope::Control);
let permit = DevelopmentPermit::scoped(
    device_id, controller_key, principal, revision, duration_ms, permissions,
)?;
let mut session = Session::admit(confirmed_channel, permit, context, now_ms)?;

// 每次业务派发读取新鲜单调时间，先检查整个连接，再检查该操作。
let grant = session.require(Scope::Control, now_ms)?;
// 后续运行网关须用该主体向既有 Runtime 取得控制权，不能跳过租约／修订。
```

| 范围 | 对应责任 | 不隐含的权力 |
| --- | --- | --- |
| Installation | 已有安装 Gateway 打开及每次工作 | 观察运行内容、接管或播放 |
| Observe | 后续运行状态／目录只读入口 | 写安装存储、取得控制租约 |
| Control | 后续控制租约和运行操作 | 安装、读取全部内容、绕过播放许可 |

Permissions 只能由显式 Scope 构造、组合；没有任意位图解码或反序列化实现。每个范围相互独立；需要既查看又控制的客户端必须显式具有两个范围。

`Session::grant` 只给出当前已认证身份和范围快照，不代表某个具体操作已授权。`Session::require` 先执行同一 grant 活性／固定开发期限检查，再匹配范围；不足则拒绝并关闭此会话。查询或保活不能延长权限，也不能从旧 Grant 恢复会话。网络请求在队列中等待时，平台仍须在实际处理前后重检并撤销过期工作代次，不能把快照长久保存为授权凭据。

## 兼容和当前接线

`DevelopmentPermit::installation` 和 SMDV v1 配置始终只含 Installation，现有 SMAP v1 回执固定安装位保持。安装 Gateway 在打开与 poll 时调用 require；Observe／Control 或两者组合都不能产生安装 Open 命令。即使可信宿主显式提供全部范围，旧安装回执仍只公告安装入口。

当前正式设备和主机连接依然只有安装协议；新增范围不会使其自动支持远程播放。第二增量已提供[类型化运行连接入口](device-runtime-application.md)，正式目录消息／固件控制队列／桌面播放按钮仍未接线，没有把已安装包伪报为运行或物理输出。具体生产身份、签名授权与商业化规则后议；既有开发连接期限不是商业使用限时。
