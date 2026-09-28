# 设备绑定档案与持久存储

DEVICE-002B；依据 [ADR-042](../development/decisions/PRODUCT-ADR-042-device-binding-vault.md)。当前实现是 `stagemaster-device-auth` 的绑定记录和可选 `persistence` 层；不是完整连接认证服务，尚未接 ESP32 蓝牙事件或正式无线安装。

## 数据与调用

`LocalIdentity` 持有固定随机静态地址和本机 IRK；`Binding` 持有随机主体 ID、对端身份地址、LESC LTK 与可选 IRK。最多四项，新增／更新／撤销生成递增修订的 `Vault` 提案。地址按蓝牙字节序保存；瞬时 RPA 不能作为持久身份地址。可信适配器先确认已认证且完成绑定，再把栈结果转换成 Binding；不能从无线 JSON 自报字段生成权限。

```rust,ignore
let mut store = VaultStore::new(partition_nor, wear_level_seed)?;
store.recover().await?; // 失败保持不可用，禁止自动格式化
let proposal = store.current().ok_or(Code::Unavailable)?.enroll(binding)?;
// 宿主先撤回相关实时权限，进入有效维护窗口，再串行执行实际写任务。
store.commit(&proposal).await?;
// 新档案持久化完成；之后仍须核对当前连接的加密和密钥持有证明。

let proposal = store.current().ok_or(Code::Unavailable)?.revoke(principal)?;
store.commit(&proposal).await?;
// 撤销成功后还须移除协议栈中的绑定并断开对应连接。
```

首次本地配备才显式调用 `initialize(LocalIdentity)`；只接受完整空白区。任何部分初始化、未知格式或损坏均保留原字节并拒绝初始化。修复／清除凭据须由后续明确的本地维护操作承接，不作为连接错误的自动重试。

## 存储和生命周期

- `Vault` 是严格 384 字节格式，检查版本、保留字节、零密钥、地址类别、重复主体／身份、数量和修订；Secret 和完整编码输出的 Debug 均脱敏，不派生网络反序列化入口。
- `persistence` 是可选特性，复用锁定 EKV 1.0.0 的原子事务。固定专用区域 132 KiB，其中一页为版本／配置标识；NOR 读写对齐最多 4 字节，擦除页必须 4 KiB。只接受已经限制容量的分区驱动，不接受“整个芯片加一个猜测偏移”。
- 配置标识在挂载前检查，编译期校验 EKV 配置；新／旧格式互不自动迁移。读操作不格式化，不修改用户节目区。
- `current()` 只返回成功验证的持久档案。过期提案在 I/O 前拒绝且不污染现有状态；开始实际变更即撤回缓存，失败后保持 None；只有显式 `recover()` 成功才恢复可用性。调用方不能保留旧克隆继续授权。
- 适配器记录底层 I/O 错误，避免 EKV 探测元数据时跳过不可读页后暴露可能更旧的绑定。取消等待不能转换成“确认撤销”；成功以提交后的完整回读为准。
- 当前适配器单一所有者，底层同步 NOR；异步签名复用 EKV，但不把同步 Flash 调用变成非阻塞操作。硬件端必须接现有存储核，不能在无线回调或输出循环调用。

## 验证与未接项

完整证据见[绑定存储验收](../development/tasks/DEVICE-002B-vault-acceptance.md)。真实板卡仍运行上一轮只读安全候选；本轮没有修改分区表或刷机。Xtensa 库级编译不是完整固件链接、实板断电、绑定恢复或峰值预算的替代。下一步是物理准入／取消和连接级状态，再组合真实绑定存储及 GATT；不能由已存档的绑定直接产生 `AuthorizedLink`。
