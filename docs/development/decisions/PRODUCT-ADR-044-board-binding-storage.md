# PRODUCT-ADR-044：板端绑定存储与单一 Flash 所有者

日期：2026-09-29；DEVICE-002B；基线 `400d071`，主工作区／当前会话实施。状态：本轮已实施并通过受控板级增量验收，完整业务安装继续。

## 决定

将 [ADR-042](PRODUCT-ADR-042-device-binding-vault.md) 的 VaultStore 和 [ADR-043](PRODUCT-ADR-043-binding-admission-and-link-authority.md) 接入现有第二核串行存储执行器。节目区仍为 `0x610000 + 0x402000`；在其末端之后增加 `stmbonds`，官方 data/undefined 类型，`0xA12000 + 0x21000`（132 KiB），末端 `0xA33000` 小于 16 MiB。旧分区、节目数据与双槽格式不移动。所有表项先经 SDK MD5／结构解析与全区间重叠检查，再校验两个用途的确切区域／类型／标记。

只创建一个 FlashStorage 并启用官方 multicore_auto_park，留在存储核。该核的分区句柄通过 Rc/RefCell 借用同一驱动，每次同步调用创建 SDK FlashRegion／NOR 视图并由其做范围与属性检查；不 clone 外设、不把 Rc 跨核发送、不在无线回调擦写。补外围长度溢出检查，避免上游 offset 加法先溢出。RefCell 忙时明确拒绝，不忙等。

新增凭据命令队列只携带内部请求号、结构化档案和操作，不提供无线反序列化入口；回复不含日志密钥。读取恢复可在启动时执行；初始化／提交须由现有 ManagedWorker 的有效维护许可包裹。EKV 底层是同步 NOR，本轮通过 embassy-futures 的现成 block_on 在这个专用执行器完成单一写任务；调用处不持有其他 EKV 事务，不能将其移到无线循环。

启动先只读恢复。完整空白区也不自动初始化；初始化由显式本地准备操作或编译期受控 `binding-local-test` 探针发起。后者仅用于本机已授权硬件验收，须独立构建／明确刷入，并且不包含无线节目写入口。未知／损坏／部分初始化分区原样保留，不以清零恢复掩盖故障。现有节目区保持只读，业务能力声明暂不改变。

蓝牙适配在广播前导入实际恢复的稳定随机地址／IRK／绑定，凭据变化后清除旧 RAM 绑定再导入核验结果。只接受已认证 LESC 栈事件，准入及主体查找使用 Authority；取消后晚到的持久完成只恢复档案，不恢复连接权限。实际永久身份和恢复验证通过后再开放正式安装 GATT。

本轮将上述适配接入 `bindings/link.rs`／`keys.rs`，无线会话从 `ble.rs` 拆到 `ble/session.rs`。在首次配对完成时立即撤回权限、请求物理断开并等候控制器确认，再持久提交／回读；存储失败只允许恢复权威档案，不能以内存提案继续。受保护的只读试验值默认全零，只有持久绑定重连且栈已认证时才发布；这仍是实验特征，不是正式业务授权协议。普通诊断不因尚未申请认证而在 90 秒后断线，认证待定资格届时失效，之后申请须重新连接；已认证连接则严格遵守 6 秒有效保活。

普通 `binding-readiness` 只恢复既有绑定，不自动开配对窗口、不初始化；受控 `binding-local-test` 额外允许全空白初始化和每次启动一次 90 秒／3 次尝试窗口，两镜像均没有无线节目写入。此测试入口不能代替后续实体按键与用户可见绑定／撤销管理；验收后恢复普通构建。

## 复用与验收

复用锁定 SDK 的 PartitionEntry、FlashRegion、multicore_auto_park，EKV 原子提交，以及现有 Runtime 维护许可。分区句柄只负责所有权／边界组合，不重写 SPI 驱动、事务存储或密码算法。验证分区表往返、无重叠／越界、真实 Xtensa 完整链接与预算、原节目目录保持、实际初始化／持久提交／重启只读恢复及无线保活。故障或证据不足时仍不开放安装权限，不将本地存储探针算成无线端到端交付。

接口依据本地锁定的官方 crate 源码核对：`trouble-host 0.8.0` 的 `Stack::add_bond_information/remove_bond_information/enable_privacy`、`ConnectionEvent` 和身份地址归一化；`esp-bootloader-esp-idf 0.6.0` 的 `PartitionEntry::as_flash_region`／NOR 视图。上游分别为 [Embassy trouble](https://github.com/embassy-rs/trouble) 和 [esp-rs esp-hal](https://github.com/esp-rs/esp-hal)。在线 docs.rs 页面本轮无法访问，版本准确性以已锁定源码和实际编译为准。

实际出口、首轮失败及尚未实现的用户入口见[板级验收](../tasks/DEVICE-002B-board-binding-acceptance.md)。
