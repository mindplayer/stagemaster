# 灯具模式文件

依据 [ADR-079](../development/decisions/PRODUCT-ADR-079-portable-fixture-modes.md) 和 [FIXTURE-004](../development/tasks/FIXTURE-004-portable-modes.md)。这是工程灯库的文件复用能力，不是 GDTF／OFL 解析或云端个人库。

```rust
let portable = document.profile_file(profile_id)?; // 捕获现有模式，无历史
let bytes = portable.encode()?;
let imported = ProfileFile::decode(&bytes)?; // 完整检查，无活动工程修改
// 用户检查后：
document.edit(EditCommand::Fixture {
    command: FixtureEdit::SaveProfile {
        id: None,
        definition: Box::new(imported.definition().clone()),
    },
})?;
```

JSON 外层固定 format=`stagemaster-fixture-profile`、formatVersion=1；source 为 `{profileId,revision}`，definition 使用既有 [ProfileDefinition](fixture-authoring.md)。嵌套未知字段、未来格式、无效身份与通道、功能表或机械范围拒绝；不下载外部资源，不执行文件内容。最多 512 KiB。导出先在临时 Document 中重建并完整校验，去除身份／修订后须与源模式相同；不支持无损保留时明确拒绝。导入检查复用完整 saveProfile 校验，不直接修改当前 Document。source 仅记录原来源，不能据此认证厂商，也不作为导入身份；新增模式用新 id 和 revision。

存储 `ProfileFileStore::read(path)` 有界读取普通文件；`select(path).save(&portable)` 只允许 `.smfixture.json`，已有目的须是本版合法模式文件；临时写入、文件基线、锁与原子提交沿用 DiskFile。源工程即使误取模式文件扩展名也不能被覆盖。新文件／内容冲突不以自动覆盖处理。

宿主：

```ts
host.importProfile(generation): Promise<ImportedProfile | null>
host.exportProfile(generation, profileId): Promise<ExportedProfile>
// ImportedProfile = { generation, fileName, source, definition }
// ExportedProfile = { generation, profileId, revision, path: string | null, warning: string | null }
```

原生服务在独立模式文件门后取得既有工程操作门；核对 generation／捕获内容后释放短 Session 锁，选择器及编解码在锁外。取消返回 null 或 path=null，失败为中文错误；服务不创建工程历史或激活播放。UI 校验回执工程版本、当前草稿、页面经历和挂载状态，过时文件不覆盖新草稿。

导入进入完整模式编辑器，显示来源文件、可展开来源修订和同名提示；默认焦点为模式名称。导入草稿必须显式“保存到工程”或“取消导入”，全局收集／导航不会隐式接受；失败定位保存按钮或具体字段，Esc／取消回到导入按钮。普通手建／编辑模式继续沿原草稿收集规则。保存后只是新增工程内模式，已有灯具不替换；使用既有配适入口选择新模式。导出草稿期间禁用，明确提示先保存或取消；导出结果带源名称及过期提示，跨页保留。

没有更改主工程格式、运行值、时钟或设备包；个人库目录、多模式包／标准格式适配、跨定义换灯映射均仍单独实施。
