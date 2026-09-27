# 工程容量与编码

PROJECT-002；[ADR-029](../development/decisions/PRODUCT-ADR-029-project-capacity.md)。这是当前 Rust 编辑器的容量契约；不修改 JSON Schema、文件版本、灯光语义或播放包格式。

`stagemaster_project::MAX_BYTES` 为 8 MiB（8,388,608 字节）。`Document::decode` 先限制原始输入字节，再执行严格 JSON／Schema／领域校验，最后检查规范紧凑 JSON 和必要修订空间。`Document::new`／`edit` 使用同一容量检查；编辑失败不提交任何字段。

空父修订列表需要预留 38 字节，使以后保存能从 `[]` 变成一个 36 字符 UUID；已有父修订列表的单父替换不再增长。按实际 UTF-8 和 JSON 转义计数，不把字符数或缩进大小作为内容大小。处于文件上限但没有修订余量的外部文档会在打开时明确拒绝，原文件不被修改。

```rust
let mut document = Document::decode(&input)?;
document.edit(command)?; // 同一完整容量检查，失败时保持原文档
let bytes = document.encode()?; // 完整内容；正常缩进，必要时紧凑
let reopened = Document::decode(&bytes)?;
let receipt = file.save(&document)?; // 原有新修订、冲突检测与原子提交
recovery_session.checkpoint(&document, source_label, now_ms)?;
```

`encode()` 的返回值始终不超过 8 MiB：优先缩进加尾换行；超限则复用缓冲区输出紧凑 JSON，有空间才附加换行。极限紧凑正文正好 8 MiB 时不附加换行。字段／数组顺序语义及内容完整保留；不截断、删对象或自动分拆工程。容量计数不建立整个输出副本，缩进输出到达限制时立即中止。

`DiskFile` 和恢复检查点只调用核心编码，不维护另一个排版容量规则。恢复封装仍为工程最多 8 MiB 加 64 KiB；恢复结果继续走核心读取校验。手动保存与恢复不改变播放包的来源摘要规则：来源仍是 ADR-028 的无换行规范紧凑 JSON 流。

`next_revision()` 和 `use_revision_from()` 的正常保存／撤销路径沿用固定长度修订身份。若未来改变身份编码、父修订规则或允许不受控合并，必须同时调整容量预留并补边界测试。当前保证容量可编码，不保证磁盘永不满；磁盘冲突、权限和 I/O 失败继续由存储层明确报告。
