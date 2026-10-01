# 灯组与预设编辑接口

状态：DESKTOP-004 已实现的接口；依据 [ADR-015](../development/decisions/PRODUCT-ADR-015-groups-presets.md)。实现入口 `crates/stagemaster-project/src/library.rs`，界面类型 `apps/ui-prototype/src/library-types.ts`。

## 调用边界

桌面沿用 `ApplicationHost.request({ kind: "edit", generation, command })`，内部命令为 `{ op: "library", command: LibraryEdit }`。Rust Document 校验、一次提交；Session 负责版本冲突、撤销／重做和预览过期；文件存储独立。普通组件不持有工程真值，不直接写文件、编译或输出。

```json
{
  "op": "library",
  "command": {
    "kind": "recordPreset",
    "name": "暖色面光",
    "sceneId": "来源场景的 UUID",
    "fixtureIds": ["所选灯具的 UUID"],
    "attributes": ["red", "green", "blue"]
  }
}
```

上例 UUID 为调用方从当前工程取得的身份；不是可导入的工程文件。所有资源 ID 由 Rust 创建。完整字段以 Rust 枚举与同名 TS 声明为准。

| 命令 | 内容与语义 |
| --- | --- |
| saveGroup | id=null 新建，否则替换名称与有序 fixtureIds；1–10000 个不重复且存在的成员 |
| duplicate | resource=group/preset、id、name；独立新 ID，预设副本不接管原引用 |
| renamePreset | 稳定 ID 下改名，引用不变；允许同名对象，身份以 ID 区分 |
| recordPreset | name、sceneId、fixtureIds、attributes；记录明确来源中存在的数值，引用解析为独立预设值 |
| updatePreset | id、来源范围、mode=existing/merge/replace；交集更新、合并或完整替换；保护仍被引用的属性 |
| applyPreset | id、sceneId、fixtureIds、attributes、linked；只应用预设与所选范围交集，空交集拒绝；linked 决定引用或独立值 |
| detach | sceneId、fixtureIds、attributes；解除所选属性的全部预设引用，数值不变；无引用时报错 |
| copyValues | sceneId、sourceId、fixtureIds、attributes；向指定目标复制来源当前记录的独立值；所有目标需支持所选属性；不复制释放或缺项；最多 10000 项 |
| remove | resource、id、keepValues；灯组删除不改灯具／场景；预设被引用时默认拒绝，keepValues=true 原子固化所有引用后删除 |

范围数组不可为空、重复或含失效灯具。掩码中每个属性至少有一台所选灯具支持；逐灯不支持／未记录项不自动填零。复制另有更严格的全目标兼容要求。资源操作可以放入已有 Batch，整体失败时零修改。

## 投影和编辑状态

ProjectView.groups 包含有序 fixtureIds；presets 包含值、usedByScenes 和 usedBySequences。SceneValue 的 presetId 与 presetName 同时显示引用身份和名称，数值由同一 Rust 解析器提供。编译器复用该投影，界面不能根据显示颜色反推持久值。

ResourcePool 管理查询、掩码、选中资源与编辑窗；GroupEditor 管理成员草稿；PresetEditor 管理捕获／更新范围；LibraryDialog 管理可取消提交、错误和焦点。UI 选择变换不写工程；只有记录灯组才持久化。工程切换重建资源上下文；工作区切换保留上下文。窗口关闭遇到未结束的编辑窗，先聚焦编辑窗并提示处理，避免丢失录入。

现阶段工程没有完整现场编程器：捕获来源是明确场景直接记录的属性，不是现场输出或列表继承求值。预设更新不热改运行计划；离线预览重新载入后生效。共享／通用／嵌套预设与效果另定契约。

UX-036 界面范围约束：记录／更新窗口继承当前明确属性掩码与可用属性的交集（去重）；明确空选保持为空，窗口内调整为独立草稿，取消不反写资源池。未限定范围时，新记录默认所有可用属性，更新默认当前预设属性。既有 existing／merge／replace 命令与核心保护不变；镜头明细使用共享中文名称。

UX-037：快捷预设、资源池及记录／更新／复制窗口共用 `preset-attribute-scopes` 与 `PresetScopeSelect`。颜色包含 RGB／色盘，图案包含图案盘／棱镜，镜头包含变焦／调焦／光圈；位置仍为双轴。动作“仅亮度”只含 dimmer，快门／频闪单独选择，不从界面亮度分类扩大动作范围。范围按当前灯具可用属性取交集，无能力选项禁用；明确掩码不随增选灯具自动扩大。乱序／重复键以集合识别范围，无有效项显示“未选择属性”，保留逐项勾选与清空。只影响界面选取，现有核心能力校验、预设覆盖及空交集保护不变。

UX-038：`CopyValuesEditor` 独立于预设窗口，来源用统一搜索，目标可在当前已选灯具中再筛选。源切换只保留属性交集，候选来源始终排除实际写入；目标检查使用当前投影发现缺少属性／功能键／类型不匹配／非法离散位置，显示对应灯具。逐项内容、跳过释放／未记录及写入量可见；超过 10000 项、无有效源／无目标／不兼容选择时阻止。搜索和分页不改变选择，“全选目标／仅选可写入／清空”明确作用于全部候选。草稿取消隔离、失败保留；Rust 继续做完整版本及事务校验，未引入另一个核心求值器。复制是值与功能身份的复用，不是不同灯型的自动光学转换。

UX-039：灯组编辑的有序成员与候选成员共用一个检索区，50 项分页；批量加入／移出只作用于明确的全部筛选结果（含其他页），隐藏成员保留。反转、奇数位在前、自然名称及配适排序作用于完整组，与效果共享 `fixture-order` 纯变换。逐台移动使用完整序号，跨页移动会展示目标页；替换当前选灯／恢复初始成员均仅修改窗口草稿。保存仍是一条 saveGroup 命令；空组／重复／失效／超容量明确阻止，Rust 最终验证。资源窗口搜索 Enter 不触发表单保存，不改变名称输入与资源选择器本身的键盘行为。
