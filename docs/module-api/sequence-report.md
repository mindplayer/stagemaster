# 场景列表节目单

依据 [ADR-081](../development/decisions/PRODUCT-ADR-081-sequence-report-export.md)，由 [REPORT-002](../development/tasks/REPORT-002-sequence-report-export.md) 实施。属于只读交接投影，不参与调度或现场输出。

```rust
let report = document.sequence_report(sequence_id)?;
let mut destination = SequenceReportFile::select(path, source_path)?;
let warning = destination.save(&report)?;
```

一行一个步骤，按工程内列表顺序，步骤号不参与排序。22 列定义见 `crates/stagemaster-project/src/sequence_report/rows.rs`：来源格式、工程名称／身份／保存修订／精确快照 SHA256、列表名称／身份／继承方式／结束行为、执行顺序、步骤号／名称／身份、幕场／台词或动作提示／备注、场景名称／身份、延时／渐变／推进方式／渐变完成后自动等待。手动等待为空，自动零等待为 0.000；秒数三位小数精确保留毫秒，不虚构人工等待时长。

```ts
const receipt = await host.exportSequenceReport(generation, sequenceId);
// { generation, sequenceId, sequenceName, stepCount, path: string | null, warning: string | null }
```

桌面 `sequence_report_export` 与配灯表共用报表门、工程操作门；Session 锁内捕获并核对代次，锁外生成／保存。未知列表在打开原生保存窗前拒绝。取消 path=null；名称和数量来自同一 Rust 快照。UI 导出前收集有效草稿，无效输入沿用原字段定位；回执保留原导出列表，跨页不清除，工程更换后重建。新代次或未应用草稿提示资料过期。

复用 Rust csv 1.4.0、有界 8 MiB 写入、UTF-8 BOM／CRLF 和文字公式前缀保护；不保证所有表格软件的自动类型推断。存储共用 ReportFile 的 .csv 扩展／源工程／链接／无关文件保护与 DiskFile 原子写入、外部修改比较及所有权锁。不同种类报表互不覆盖。导出不推进保存修订、不写历史、不触发预演。音乐时间线／多轨导出、PDF／打印和跨平台原生选择器另验。
