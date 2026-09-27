# 播放包安装事务

PLAYER-003B 已实现；[ADR-030](../development/decisions/PRODUCT-ADR-030-package-installation.md)。软件参考接口，未接正式 GATT 或 Flash；不改变既有诊断协议，也不提供远程权限。

核心 `Installer<S: Storage>` 拥有存储适配。`open(storage, boot_id)` 返回安装器和两槽启动检查报告；`head()` 是最新有效已安装候选，不代表正在播放。`transaction(counter)` 将当前启动身份与调用者递增计数绑定；计数从 1 起。同一个身份用于该安装器生命周期，断线不断开安装器；重启必须换新的身份。

```rust
let (mut installer, report) = Installer::open(storage, boot_id)?;
let transaction = installer.transaction(1);
let progress = installer.begin(transaction, package_identity)?;
// 相同已安装包可直接返回 Committed；否则从 progress.received 连续传入分块。
if progress.phase != Phase::Committed {
    // ... installer.write(transaction, offset, chunk)?;
    installer.verify(transaction)?; // 全部接收，持久同步并独立校验
}
let receipt = match installer.commit(transaction) { // 不执行节目
    Ok(receipt) => receipt,
    Err(Error::CommitUncertain(_)) => {
        installer.reconcile()?; // 同步／读取失败则仍待确认，不能启动另一事务
        installer.commit(transaction)? // 对账确认为已提交才返回成功
    }
    Err(error) => return Err(error),
};
let snapshot = installer.snapshot()?; // 独立固定读源；可跨下一次安装继续使用
let program = snapshot.load(program_index)?;
```

`Storage` 负责两槽读取／暂存／同步／提交记录／快照租约；`Installer` 唯一解释包完整性和事务状态。`prepare` 不得修改当前槽或被快照占用的槽；`commit_record` 成功必须表示载荷及记录持久化，失败允许提交点已经发生，需 `settle` 后重查。失败也允许目标槽记录可检测地损坏，但必须保持另一槽的载荷和记录完整；逻辑原子性由双槽检查和选择完成。文件适配提供更强的单记录原子改名；Flash 适配不能假定 96 字节物理原子写，需实测撕裂识别和另一槽保护。

接收连续前缀，最大单块 1 KiB；重复请求仅在整个范围已经接收、内容相同时成功。部分重叠／超前偏移不自动补洞。取消保留已安装版本；提交后不能用取消隐式回滚。最后一个终态可重复查询，开始新事务后所有旧计数失效。相同已安装摘要及长度可返回已有提交，不重复写 Flash。

固定提交记录 v1 共 96 字节：`0..8 STMINST\0`、`8..10` 版本 1、小端；`10` 槽号 0/1；`11..16` 零；`16..24` 非零代数 u64；`24..28` 包长度 u32；`28..32` 零；`32..64` 包摘要；`64..96` 为前 64 字节 SHA-256。摘要沿用包头里的 Archive 摘要（不是整文件直接 SHA）；对应载荷必须再通过 Archive 完整验证。记录的槽号必须与实际位置一致。

运行中快照须显式释放后相应旧槽才可回收；不能为腾空间偷偷让旧快照转读新包。安装服务不保存播放进度、不发 DMX，也不判断商业授权。上面省略了分块循环；可编译的实际调用见 [install.rs](../../crates/stagemaster-install-store/examples/install.rs)。

## 状态与错误

`progress()` 返回最后一个事务的身份、接收前缀、阶段和计划目标。**`Progress.commit` 在阶段为 `Committed` 之前只是计划目标，不能作为成功回执**。`head()` 和 `Installed::commit()` 才表示已完整验证的安装候选。已安装候选也不等于已获授权或正在执行。

| 阶段 | 可执行的操作／恢复 |
| --- | --- |
| `Receiving` | 连续写入、核对重复块、查询、取消；全量接收后验证 |
| `Verified` | 完整性已通过；提交前再次校验，可取消；禁止继续写块 |
| `Committed` | 重复提交返回同一回执；可开下一事务，不能取消回滚 |
| `Cancelled` | 重复取消无副作用；可用下一个计数重新安装 |
| `Failed` | 部分写入或验证失败，先取消再用下一个计数重试 |
| `Uncertain` | 只允许核对持久结果；同步／读取失败保持此状态，禁止取消和新安装 |

不完整接收、乱序、越界、重复内容冲突不改变已确认前缀。已确认区域的读取失败、写入失败或完整性验证失败会使事务失败。`verify` 对已验证／已提交事务可重复调用；`write` 只接受接收阶段。开始新事务后不再保存旧事务的回执，应以当前事务及安装摘要对账。

`Error::Code` 分为身份／过期 `Identity/Stale`、顺序／忙 `Order/Busy`、边界／冲突／缺块 `Bounds/Conflict/Incomplete`、状态／待确认 `State/Uncertain`、无安装／计数耗尽／记录冲突 `Empty/Exhausted/Metadata`；显示文本为中文。`Storage(E)` 保留适配错误；`Package` 保留全包错误；`CommitUncertain(E)` 必须走核对流程。调用者不能统一把错误转换成“请重发上一块”。

`RecoveryReport` 分槽报告 `Empty/InvalidRecord/InvalidPackage/Ready`、选择结果及代数高水位。没有记录时，即使存在完整载荷也仍是 `Empty`。有效记录对应的包损坏可回退较早完整版本；元数据读不出或相同代数冲突返回错误，不臆测缺失／成功。两槽均无有效候选时返回空安装并保留证据。

## 存储适配与预算

窄接口见 [Storage](../../crates/stagemaster-install/src/lib.rs)：容量、记录、实际长度／精确读取、准备、写入、持久同步、记录提交、待确认同步、释放与固定快照。`FileStore` 使用专用目录、单写者 OS 锁、各槽读源租约、同目录临时文件／改名和目录同步。重开只清理自己命名的暂存文件；不删除未知文件。受管文件拒绝符号链接及非普通文件，Unix 也拒绝多硬链接；这不构成对不合作外部进程或物理介质损坏的隔离保证。

参考容量每槽 2 MiB；主机可能同时保留两槽加一份暂存，峰值约 6 MiB 加记录／锁。核心单次重复核对最多 1 KiB 栈，主机类型尺寸保护为安装器不含存储 ≤384 字节、进度 ≤160 字节。Archive 校验另有既有预算，不代表 MCU 在无线栈及旧节目同时运行时已经满足内存／实时预算。

复现软件安装／重开／重放（在项目根目录执行，目录应专用于该参考工具）：

```sh
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo run --locked --offline \
  -p stagemaster-install-store --example install -- data/install-reference path/to/show.smpkg
# 省略最后的包路径：恢复检查已有提交，再用虚拟时间重放；仍可能清理中断暂存文件。
```

工具明确显示不连接设备、不发送 DMX；每个节目输出 400 帧的摘要。文件故障注入、进程异常退出、Xtensa 编译和逐帧对比见[验收记录](../development/tasks/PLAYER-003B-package-installation.md)。GATT 上传、Flash 分区与真实断电、运行切换、可信时间及商业授权仍须分别实现和验证。
