# 本地灯效模板核心 · v1

LIBRARY-002／[ADR-091](../development/decisions/PRODUCT-ADR-091-local-intensity-templates.md)。核心及桌面文件入口已实现，正式应用跨工程复用与保存重开通过。云服务伪 API 仍见 [effect-library.md](effect-library.md)，不能把设计稿全部当成已实现。

`EffectTemplateFile::decode(&[u8])` 严格读取最多 16 KiB JSON；`create(EffectTemplateDefinition)` 创建独立模板／修订；`encode()` 写独立文件，`source()` 计算完整语义内容摘要。格式在 project.schema.json 的 `EffectTemplate` 定义，与工程来源共用同一校验器。

```rust,ignore
let file = document.effect_template_file(scene_id, effect_id)?;
let bytes = file.encode()?; // 不含场景、灯具身份、配适地址或静态颜色
let imported = EffectTemplateFile::decode(&bytes)?;
let reviewed = target.review_effect_template(&imported, target_scene, &ordered_fixture_ids)?;
show_review(reviewed.view()); // 目标场景、完整效果和实际桌面计划预算
// 取消：释放 reviewed，不修改文档。
target.apply_effect_template(reviewed)?; // 一次历史由应用适配器维护；不播放
```

首版 recipe 为 `intensity-wave`，waveform 为 `smooth | triangle | pulse`；low/high 为归一化 0..65535，dutyPercent 1..99。timing 为周期 100..3600000 ms、phaseDegrees 0..359、spreadDegrees 0..360 和 reverseOrder。平滑沿用核心 smoothstep，灯序展开沿用 i/N，相位／暂停等运行语义全部由现有播放器承接。模板不引用颜色、灯位或基础位置。

导出只接受恰好一个 dimmer 范围属性的基础曲线；停用状态不写模板。每次导出是新的独立模板版本，参数与当前效果一致。导入要求 1..512 个不重复目标、显式连续调光；只读检查和实际单场景编译通过后获得不可从 JSON 构造的 `EffectTemplateReview`。审阅保存完整原文档与候选；应用核对完整内容，包括尚未保存的编辑，拒绝过期审阅。应用生成已启用效果，仍须用户正常启动预演／执行；同属性冲突不能默默覆盖。

`SceneEffect.templateSource` 是 `{template, sha256}` 完整来源快照，额外要求 `lighting.effects.template-source@1`。哈希为模板经固定对象键排序的紧凑 UTF-8 JSON 的 SHA256；v1 不含浮点数。重复模板来源不是重复工程对象；工程内 ID 与模板 ID 独立。来源属于历史信息，手工编辑／复制时完整保留，已有效果的 Put 不允许改写或移除来源；当前效果参数可以与来源不同。编译不读取网络或来源配方，而读取工程内已绑定的完整效果参数。

来源每次解码都检查严格字段、版本、范围、能力声明与摘要。摘要是完整性核对，不是数字签名或云权限。当前只报告桌面计划容量；ESP32 预算、其他 recipe、多发光单元、云端目录与签名尚未接通。

同一工程内若出现相同 templateId／revision 而摘要不同的来源，拒绝整个导入或打开；不同内容必须使用新修订。允许多处重复引用同一完整快照，也允许同一模板的不同修订并存。

桌面适配由 `effect_template/files`、`effect_template/pending` 与 `session/edit` 分别负责文件交互、单项审阅缓存和统一历史安装。界面通过 `ApplicationHost.importEffectTemplate(generation, sceneId, fixtureIds)` 打开原生文件选择器，返回取消或只读审阅及随机 token；再用 `applyEffectTemplate` 工程请求提交 token。核心候选只保存在 Rust 中，不接受界面重新拼接候选数据。缓存最多一项，五分钟后拒绝应用，取消只释放匹配 token；新导入替换旧审阅。全部应用继续走工程事务队列、代次保护、恢复保存和原来的撤销栈，不能自行加载或启动预演。

`exportEffectTemplate(generation, sceneId, effectId)` 导出当前已应用效果，原生保存框要求 `.smeffect.json`；`EffectTemplateFileStore` 沿用 `DiskFile` 的原子写入和并发修改检测，拒绝链接、不兼容文件和工程文件覆盖。文件读取有 16 KiB 上限。导出不改工程修订。

界面先提交有效草稿，再冻结场景及有序目标。场景、灯序、页签或工程代次变化会取消审阅并拒绝迟到回执；离开再返回同一选择也不能接收旧结果。审阅明确显示启用状态、亮度范围、节奏及灯序；错误保留在审阅内，取消后回到导入按钮。来源标记随效果显示，离线打开不依赖模板原文件。
