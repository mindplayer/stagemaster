# 设备绑定准入和实时权限

DEVICE-002B；模块 `stagemaster_device_auth::authority`；依据 [ADR-043](../development/decisions/PRODUCT-ADR-043-binding-admission-and-link-authority.md)。当前为经过软件／存储组合验证的核心，尚未接真实蓝牙栈和固件。

## 宿主调用顺序

```rust,ignore
let mut auth = Authority::new(store.current().ok_or(Code::Unavailable)?.clone(), now_ms);
let connection = auth.connect(device_random_128_bits, now_ms)?;
// 只从当前实体连接的协议栈事件构造 evidence，不从无线请求反序列化。
let grant = auth.resumed(connection, &stack_evidence, now_ms)?;
// 每次分派前，重新采样当前链路认证等级并调用 grant；不要缓存旧授权。
let grant = auth.grant(connection, actual_stack_security, now_ms)?;
// 适配为已有 AuthorizedLink，再经 Endpoint／工作器队列安装。
// 成功通过原诊断 Session 的消息，才允许续约：
auth.heartbeat(connection, actual_stack_security, now_ms)?;
```

`Evidence` 包括栈已确认的 LESC 认证等级、是否绑定、恢复／新配对来源和对端密钥。`Peer` 不含自报主体；恢复必须与持久档案的地址／LTK／可选 IRK 一致，再返回存档主体。键值 Debug 脱敏。密钥持有与地址解析由栈完成，这些类型不另实现密码认证。

`Connection` 为内部递增代次；`Grant` 提供主体、业务随机数和代次，不能持久化。适配器把失联／取消／超时／降级同步到工作器 LIVE_EPOCH，撤回半包及通知，再断开实体链路。`poll()` 空闲时也要执行；返回需关闭的连接，计时错误则无条件关闭当前实体连接。

## 新增绑定和撤销

本地物理操作才调用 `open_pairing()`；90 秒／三次，打开已存在的窗口返回忙，断开和失败不重置次数。`begin_pairing()` 在显示验证码前消费一次额度；未准入时拒绝、撤回权限并断开。`set_bondable(false)` 不足以阻止配对，不能替代本层准入。

当前尝试完成后 `paired()` 只返回 `Vault` 提案，同时暂停档案与连接权限。适配层先断开、撤销工作器权限，再在已有维护／Flash 执行器调用 `store.commit()`，之后只用 `store.current()` 的实际核验结果恢复 Authority。新连接必须重新走密钥恢复证明，写入完成本身不赋予旧连接权限。新主体由设备 RNG 生成；已有同一对端更换密钥仍保留其主体。

`revoke()` 同样先暂停权限再返回提案；持久成功后移除栈 RAM 中的绑定。错误／超时后的结果未知，维持不可用直到明确恢复；可能恢复完整旧或新档案，不把错误当撤销成功。`restore()` 拒绝身份更换、倒退到最后确认修订之前及覆盖仍可用的快照，不复活旧连接。任何外部凭据 I/O 前必须 `suspend()`；本层无法感知调用方偷偷修改存储或持有旧档案副本。

## 期限与未完成项

连接等待证明的绝对上限 90 秒，准入尝试还受全局窗口限制；认证后 6 秒保活。查询权限、弱请求及重复结果都不续期，恰好到期亦拒绝。旧代次结果不影响新连接，代次不回绕；计时倒退关闭连接与准入。

板级物理按钮、稳定广播身份、凭据分区与串行执行器、栈事件清理及真实无线安装仍待接入。生产许可与输出权限分属其他模块；不能据此宣称已实现商业授权或 DMX。
