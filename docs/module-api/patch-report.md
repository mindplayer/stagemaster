# 配灯表导出

依据 [ADR-078](../development/decisions/PRODUCT-ADR-078-patch-report-export.md)，由 [REPORT-001](../development/tasks/REPORT-001-patch-report-export.md) 实施。本接口用于搭建／交接资料，不是工程交换、MVR 或设备执行输入。

```rust
let report = document.patch_report()?;
let count = report.fixture_count();
let mut destination = PatchReportFile::select(path, source_path)?;
let warning = destination.save(&report)?;
```

Document 只读生成当前精确快照，每台灯具一行；包括未配适／未布置对象。Rust 负责模式／地址、世界安装位置与空间／桁架／灯组关联。缺失数值为空，安装旋转与演出中的水平／垂直轴角分开。工程快照 SHA256 来自同一 Document.encode，保存修订不因导出而递增。

28 列中文表头由 `stagemaster-project/src/patch_report/csv_format.rs` 定义，按输出域／线路／数值地址／身份稳定排序。复用 csv 1.4.0；UTF-8 BOM、逗号、CRLF；公式前缀的人类文本加单引号，数字字段保持数值。最大 8 MiB，写缓冲在追加之前检查预算；超限不创建半份报表。

```ts
const receipt = await host.exportPatchReport(generation);
// { generation, path: string | null, fixtureCount, warning: string | null }
```

桌面命令 `patch_report_export` 持独立导出门及现有工程操作门；短暂取得 Session 锁核对 generation、捕获文档和源路径，然后在锁外编码、弹原生保存窗与落盘。过期版本拒绝，取消返回 path=null。TS 不提交自行组装的 CSV；先调用既有 captureCheck 收集有效草稿，无效字段定位原处。结果与当前 generation 或未应用草稿不一致时提示重新导出；跨页面保留结果，换工程清空。

`.csv` 目的文件由 project-store 复用临时写入／文件基线／所有权锁与原子提交。拒绝当前工程、符号链接、非普通文件以及非本版配灯表的已有文件；同格式报表仍须通过文件未被外部更改的检查。导出不写工程历史、不激活预演、也不持有现场输出权限。工程操作在导出门后排队，取样会话锁不会在保存窗口期间被长时间占用。

CSV 是人类资料：转义文字不保证回导还原，也不承诺所有表格软件的类型推断行为。具体内容、文件冲突、原生选择器及应用边界验收见工单；其他平台与表格应用兼容需分别验证。
