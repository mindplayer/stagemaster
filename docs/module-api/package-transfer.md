# 播放包传输接口

PLAYER-003C；[ADR-031](../development/decisions/PRODUCT-ADR-031-package-transfer.md)。`stagemaster-transfer` 已有 no_std 编码／重组、受限安装服务与主机上传协调。软件协议已在文件和 [NOR 存储](nor-package-store.md)参考路径验证；[独立运行层](device-runtime.md)另行承接。尚未接正式 GATT 特征或密码学认证，真实 Flash 写入／无线安装未验收。

## 模块调用与归属

```rust
let (installer, recovery) = Installer::open(storage, boot_id)?;
// 宿主须保留／上报 recovery 中的损坏或较早版本回退，不静默丢弃诊断。
let mut service = Service::new(installer)?;

// 可信宿主验证本设备安装权限后才可调用；不是网络请求，不是诊断握手。
service.attach(AuthorizedLink { principal, session: new_random_connection_id })?;
let mut upload = Upload::new(stable_package_reader)?; // 全包独立验证；显式安装意图
upload.connect(new_random_connection_id)?;
while let Some(request) = upload.outbound()? {
    // 适配器分片发送并重组为完整消息；真实适配不可阻塞无线／输出回调。
    let reply = service.process(request.bytes())?;
    upload.accept(reply.bytes())?;
}
let result = upload.outcome(); // Installed / Cancelled / NotStarted
```

上面省略传输适配的异步收发；完整可运行参考见 [transfer.rs](../../crates/stagemaster-transfer/examples/transfer.rs)。`Service` 独占 `Installer`，必须在其启动恢复完成、还没有内存事务时构造；不公开裸写接口。`Upload<R: ReadAt>` 持有稳定读源，只暂存一个在途消息和最多 1 KiB 源块；新增 `ReadAt for &T` 允许借用已有读源。文件参考工具为了冻结主机输入，先有界读取最多 2 MiB 为自己的快照，不代表上传器必须整包驻留 RAM。

`Service::attach` 是宿主权限边界。`principal` 和 `session` 均非零，后者每个连接随机更新；类型本身不能证明已认证，绝不能从收到的同名字段直接构造授予。服务默认拒绝收发，单连接占用；同一启动中的未完成事务只允许原主体续写、取消、校验或对账。当前没有远程强制夺取／清空接口；正式认证、物理配对和管理员接管须独立定义。设备重启丢弃未完成事务，仍需重新验证安装权限。

`detach` 只撤销连接，不取消上传、不切换／停止节目；服务的 `snapshot()` 可交给单独运行层。安装与运行、24 小时商业授权、输出确认继续分开。

## 封包 v1

采用 SMP v2 的 8 字节头：字节 0 为 `8 | OP`，查询请求／响应 OP=0/1，其余安装请求／响应 OP=2/3；字节 1=0；`2..4` 正文长度 u16 大端；`4..6` 应用组 `0x5354`；字节 6 为正文请求序号的低 8 位；字节 7 为命令。总长限制 **1,280 字节**，包括头。该组是项目内软件参考定义，尚未承诺外部工具或硬件长期兼容。

CBOR 复用 minicbor；只接收固定数量的键／确定长度容器，允许 map 键重排，拒绝重复／未知键、缺失、类型错误、尾随数据及未知版本。完整 u64 请求序号不会因头的 8 位序号回绕而重复。

请求为 `{ "v": 1, "link": bytes16, "id": uint64, "body": array }`，连接 ID 不等于安装启动 ID。命令如下：

| 命令 | 头命令号 | 正文 `body` |
| --- | --- | --- |
| 查询状态 | 0 | `[]` |
| 开始安装 | 1 | `[boot16, counter64, packageLength32, archiveDigest32]` |
| 写分块 | 2 | `[boot16, counter64, offset32, bytes]`，1–1024 字节 |
| 全包校验 | 3 | `[boot16, counter64]` |
| 持久提交 | 4 | `[boot16, counter64]` |
| 取消安装 | 5 | `[boot16, counter64]` |
| 核对待确认提交 | 6 | `[boot16, counter64]` |

包身份、偏移、连续前缀与提交保证沿用[安装接口](package-installation.md)。包摘要是 Archive 定义的摘要；不是任意文件名、UUID 或客户端声称的成功标志。

成功响应为 `{ "v":1, "link":bytes16, "id":uint64, "state":array }`；业务错误额外带 `"err":{"group":0x5354,"rc":uint8}`，仍返回操作后的实际状态。无效封包／未知协议／会话错误无法可靠解释，直接返回本机 `Error` 并撤销连接；此模块只实现本应用组，不是完整通用 SMP 管理服务器。

`state` 为 `[boot16, head|null, progress|null, owned, maxChunk16, maxPackage32]`：

- `head` 为 `[slot, generation64, length32, digest32]`，表示当前有效安装候选。
- `progress` 为 `[counter64, received32, phase, plannedCommit]`；计划提交同 `head` 形状，包含该事务身份对应的包长度／摘要，boot 沿用本状态的启动 ID。
- 阶段 0 接收、1 已验证、2 已提交、3 已取消、4 失败、5 待确认。只有阶段 2 的计划提交才能当作成功回执，且必须等于 `head`。
- `owned` 指该状态中的事务是否属于本连接已授权主体；不是“请求是否有网络权限”。没有事务时必须为 false。
- 当前 `maxChunk=1024`、`maxPackage=2097152` 是协议上限，不能当作实板空闲 Flash、实时预算或商业能力承诺。实际存储准备仍可拒绝。

业务错误码 1–12 依次为身份无效、事务过期、顺序、忙、范围、冲突、缺块、状态、待确认、空安装、耗尽、元数据冲突；30 存储失败、31 包被拒绝、32 事务归属不符。`RemoteError` 提供中文显示；适配日志可另记录完整本机原因，响应不传原始路径／驱动内部数据。

## 重传、取消与重连

每个连接请求 ID 从 1 起严格递增，只允许一个在途。`outbound()` 在 `accept()` 成功接纳前总是返回同一字节消息；`accept()` 必须核对连接、完整序号、命令及成功状态的变化，ATT 写回调不能替代它。旧／坏回复不推进主机。服务缓存最后请求 SHA-256 和完整响应，重复字节返回原回执；同号不同内容、跳号或过期请求撤销连接。摘要不构成认证。

`Assembler::push` 按头总长积累，`take()` 只取完整消息并开始下一条；无分配、无额外分片头。可以处理小于包头的第一片。空片、超长、无效头或一片跨越两条消息会使重组器失效，必须随连接一起丢弃。

完整请求的响应丢失，可在同一消息边界重传。发送中途不确定、半包超时、分片缺失或连接断开，必须丢弃两端半包及旧通知队列；重新授权后调用 `connect(new_id)`，主机先查询，再按设备前缀续传。不能把从头重发的消息拼进旧半包。适配器负责计时、断线检测和有界重试次数；核心不读取墙钟，不承诺无线后台常在线。设备重启换 boot，主机从零重新传；若已提交则按摘要认出，避免重复安装。

存储错误会停止自动推进；`retry()` 在没有在途消息时查询状态，**不会自动取消失败事务**。用户明确取消调用 `request_cancel()`；同一在途操作先获结果，或重连查询后再取消。待确认提交必须先对账，不能绕过。若提交已成功，结果为 `Installed`；否则才能得到 `Cancelled` 或尚未开始的 `NotStarted`。

完成／取消结果是本次意图的历史结果，重连后保持终态，不代表设备当前仍在使用它。不得用旧上传对象自动覆盖后续安装；再次安装需新建 `Upload`。安装协议本身没有运行命令；独立选择／控制由[设备运行层](device-runtime.md)承接，尚未增加控制的线缆／无线封包。

## 验证与资源

参考命令（项目根目录，最后一项可选，表示主动丢弃指定请求的应用回执并重连）：

```sh
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo run --locked --offline \
  -p stagemaster-transfer --example transfer -- data/transfer-reference path/to/show.smpkg 20 3
```

实际 614 字节桌面导出包：5 次请求、双向 1,529 字节；20 字节载荷需 80 片，244 字节载荷需 12 片。写块回执丢失并重连为 6 次／1,691 字节；提交回执丢失为 6 次／1,730 字节。三个节目各 400 帧都与源包一致。这是内存链路协议量，**不是实测蓝牙吞吐、延迟或设备播放**。

主机类型尺寸保护：单重组器 ≤1,400 字节，安装服务不含存储 ≤2,048 字节，上传器不含实际源内容 ≤2,048 字节。服务缓存、两端重组、临时编码及 Archive 校验的组合栈／堆需在固件无线任务中测量；固定单对象尺寸不等于完整 ESP32 内存验收。实际编译、故障及回归见[工单](../development/tasks/PLAYER-003C-package-transfer.md)。
