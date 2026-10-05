# 工程恢复接口

RECOVERY-001；[ADR-027](../development/decisions/PRODUCT-ADR-027-project-recovery.md)。已实现本地工程恢复；工程 JSON 格式不变。

## 模块与调用

`stagemaster-project-store` 独立于 Tauri、播放、云端和硬件：

```rust
let store = RecoveryStore::open(&dedicated_directory)?;
let mut session = store.begin()?; // 此编辑会话全程保有操作系统文件锁
session.checkpoint(&document, source_path_label, unix_time_ms)?;
let catalog = store.list(unix_time_ms)?;
// 用户选中 inactive 条目之后：
let mut candidate = store.claim(&entry.id, &entry.token)?;
let recovered = candidate.document.clone();
// 调用者把 recovered 作为无保存目标的副本；建立新检查点或手动保存成功后：
candidate.discard()?;
// 已保存或用户明确放弃当前会话编辑时：
session.clear()?;
```

`RecoveryStore::open` 只接受宿主提供的专用目录；库不会读写 `source_file` 指向的文件。记录名为会话 UUID，不接受任意路径。每个记录最多为工程限制 8 MiB 加 64 KiB 封装；每个会话最多一份、目录生成上限 64 份。未清理的有效、损坏及非普通记录均计入容量，不静默淘汰。

`list` 返回 `entries` 与 `omitted`（人工放入超量记录时单次最多显示 64 条）。条目含 `id / token / projectName / capturedAtMs / sourceFile / state / older / problem / canDiscard`。`state` 为 `ready / active / damaged`；是否可丢弃必须看 `canDiscard`，不能只看损坏状态。锁定的活动副本不可恢复、不可清理；时间不作为进程存活判据。超过 30 天只作提示。

`claim` 持有独立租约，直到调用者丢弃来源记录或释放认领。取消只释放租约，来源记录保留。`token` 是列表读取到的文件内容摘要；每次恢复／丢弃重新核对摘要和租约。超限／读取失败的普通文件以元数据令牌标识，仅允许显式丢弃，不可恢复。符号链接和目录均拒绝。

检查点采用 tempfile、文件同步、原子替换、目录同步。替换前错误保留旧文件；替换后目录同步失败返回明确的持久性提示，不假定已恢复旧版本。清理可重试，且仅处理指定会话的记录及临时文件。列表回收已无记录、无持有者的孤立租约与关联临时文件，不删除未知文件或旧工程数据。

封装版本为 1，内含原始 `Document` JSON、文档摘要、来源、时间、会话和检查点标识。`RawValue` 保留重复键供核心拒绝；校验摘要不替代工程格式和语义校验。这是恢复检查点，不是授权／加密格式，也不是多版本历史库。

PROJECT-002 后，检查点复用核心[容量与编码](project-capacity.md)：必要时嵌入紧凑 JSON，避免缩进膨胀阻止恢复保护；封装版本和容量上限不变。

## 桌面宿主

`apps/desktop/src/recovery.rs` 管理恢复策略，`Service.operations` 串行化所有工程请求与检查点顺序。常规编辑先提交核心，再抓取有效文档，释放 `Session` 锁后写恢复点；预览轮询可继续获取会话。保存／打开等现有原生模态流程仍是短期互斥边界。

相同工程代数且已同步时不重复落盘。写入失败不回滚已提交编辑、不伪装为“编辑失败”；快照携带 `recovery: {state, capturedAtMs, problem}`，保护状态为 `clean / protected / unprotected`。失败后可重试，后续工程操作也会再次尝试。

IPC：

- `project_request({kind: "recover", generation, id, token})`：先认领，再处理当前工程的保存／不保存／取消，最后把来源作为 `file=None` 的未保存工程；历史与播放器清空。先建立本窗口新检查点，成功后才清除来源。失败则持续持有并保留来源记录。
- `project_request({kind: "snapshot"})`：读取工程和恢复保护状态；未成功保护时也可用于显式重试。
- `recovery_request({kind: "list"})`：读取恢复中心；不切换工程。
- `recovery_request({kind: "discard", id, token})`：明确丢弃未变化、未占用的普通记录，返回刷新后的目录。

恢复后 `sourceFile` 仅作来源说明，并在后续恢复点延续；文件保存目标始终为空，首次保存必须选路径。若当前工程保存已经成功、后续切换失败，前端重新读取会话代数，保留输入上下文，不继续发旧代数请求。

开发构建路径：项目 `data/recovery/`。正式构建通过 Tauri 本机数据目录适配；本轮仅验证 macOS 开发构建，不宣称安装版和其他平台已验收。

## 界面与边界

顶部“恢复”和欢迎页“恢复工程”打开独立恢复中心，含来源／日期、搜索、状态、刷新、恢复为副本与确认丢弃。恢复保护状态与工程“已保存／未保存”分开：只保护已经应用的有效内容；输入框尚未应用的草稿会单独提示。

应用不会自动恢复、自动开始播放或覆盖来源文件。恢复不包含撤销栈、选择、输入草稿、播放进度和外部素材文件。工程现已包含音乐身份与编排，但外部音乐不嵌入检查点；恢复副本需本机有效资源或[重新定位原音乐](audio-editing.md)，另行保存时归档随附文件。离线恢复与设备运行授权是不同模块。

验证详情见[工单](../development/tasks/RECOVERY-001-project-recovery.md)。
