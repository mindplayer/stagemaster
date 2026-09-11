# 预设、素材板与引用更新

预设的价值是把“是什么颜色、打到哪里、怎样运动”变成可复用的对象。比较时必须区分按灯具保存、按灯型共享、跨灯型通用，以及调用时保留引用还是展开为实际值。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M07-01 | 单灯差异 | Selective 保存指定灯具的数据 | Normal 保存各灯具的数据 | [MA Presets](https://help.malighting.com/grandMA3/2.5/HTML/presets.html)、[Titan 创建](https://manual.avolites.com/docs/palettes/creating-palettes) |
| M07-02 | 同灯型共享 | Global 以灯型共享数据，可同时有选择性差异 | Shared 对同型号灯具共享 | 同上 |
| M07-03 | 跨灯型共享 | Universal 根据属性适用性应用 | Global 限于 Dimmer、Pan、Tilt、Colour；颜色转换受灯具能力影响 | 同上 |
| M07-04 | 分类与记录过滤 | 特征组池有输入过滤，All 池可存多类属性；单对象也能过滤 | 属性掩码和记录模式共同决定素材板内容 | 同上 |
| M07-05 | 更新引用 | Cue 和其他预设可以引用预设 | Playback 和嵌套 Palette 可以引用素材板 | [MA 编辑](https://help.malighting.com/grandMA3/2.5/HTML/presets_edit.html)、[Titan 编辑](https://manual.avolites.com/docs/palettes/editing-palettes) |
| M07-06 | 嵌套素材 | Embedded Preset；深链有更新风险提示 | Nested Palette；Fire Nested 可决定采用引用结果还是原存储值 | [MA Presets](https://help.malighting.com/grandMA3/2.5/HTML/presets.html)、[Titan 创建](https://manual.avolites.com/docs/palettes/creating-palettes) |
| M07-07 | 更新范围 | Original Content Only 与 Add New Content；新加图层涉及 Recast 或重新 Cook | Merge、Replace、Quick Merge；Quick Merge 限定已有属性 | [MA 编辑](https://help.malighting.com/grandMA3/2.5/HTML/presets_edit.html)、[Titan 编辑](https://manual.avolites.com/docs/palettes/editing-palettes) |
| M07-08 | 时间素材 | 可包含时间；只有时间的预设不按通常值引用方式进入 Cue | 可存时间素材；直接调用默认忽略已存时间，可通过 Key Profile 改变 | [MA Presets](https://help.malighting.com/grandMA3/2.5/HTML/presets.html)、[Titan 时间](https://manual.avolites.com/docs/palettes/timing-with-palettes) |
| M07-09 | 运动素材 | 多步 Phaser Preset、Phaser Recipe | Shape／Key Frame 等效果可进入 Palette | [MA 创建](https://help.malighting.com/grandMA3/2.5/HTML/presets_create.html)、[Titan 创建](https://manual.avolites.com/docs/palettes/creating-palettes) |
| M07-10 | 选择变换素材 | Preset 可带 MAtricks；Recipe 可组合选择和素材 | Group、效果中的灯具顺序与素材板配合；不据此推定等同 Recipe | [MA 创建](https://help.malighting.com/grandMA3/2.5/HTML/presets_create.html)、[Titan Groups](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups) |
| M07-11 | 插值素材 | MAgic 定义多个控制点，并按当前选择展开 | 本次未建立与 MAgic 多轴插值完全等价的对象 | [MA 创建](https://help.malighting.com/grandMA3/2.5/HTML/presets_create.html) |
| M07-12 | 直接现场调用 | 通过预设、选择和编程器应用；动作可配置 | Quick Palette 无选择时作用于相关灯具；效果素材不能这样调用 | [MA Presets](https://help.malighting.com/grandMA3/2.5/HTML/presets.html)、[Titan 调用](https://manual.avolites.com/docs/palettes/using-palettes) |
| M07-13 | 现场时间控制 | 结合编程器时间、预设时间及播放控制 | 调用 Fade、Overlap、Master Time、Master Overlap | [MA 时间控制](https://help.malighting.com/grandMA3/2.5/HTML/masters_grand_time.html)、[Titan 时间](https://manual.avolites.com/docs/palettes/timing-with-palettes) |
| M07-14 | 相关性与依赖查看 | 池图标、引用和 Info 帮助查看对象关系 | 可灰显不适用素材，查看使用它的 Playback | [MA Presets](https://help.malighting.com/grandMA3/2.5/HTML/presets.html)、[Titan 调用](https://manual.avolites.com/docs/palettes/using-palettes)、[Titan 编辑](https://manual.avolites.com/docs/palettes/editing-palettes) |

## 最容易误用的对应关系

**MA Global ≈ Titan Shared；MA Universal ≈ Titan Global。** 这里只是适用范围的近似对应，不能按同名枚举迁移数据。混合灯型的白色、色轮颜色和连续混色也不能假设视觉上完全相同。

MAgic 直接调用会计算出实际值，不保留通常的预设引用；放在 Recipe 中才有可重新计算的引用路径。不能把所有“调用素材”都实现成同一种绑定操作。[MA Create New Presets](https://help.malighting.com/grandMA3/2.5/HTML/presets_create.html)。

## 工作流程：演出前修正一个面光位置

自拟比较流程：创建“主持人位”位置素材，供三个 Cue 引用；只修改该位置，确认三个 Cue 的位置随之变化，颜色和亮度保持各自内容。然后只给素材增加一个以前没有的属性，检查既有 Cue 是立即采用、需要更新引用范围，还是需要重新计算。最后将一个 Cue 的位置展开成固定值，再更新素材，确认该 Cue 已脱离引用。

这组操作应分别记录“值变了”“引用指向变了”“引用覆盖的属性集合变了”。Titan 更新素材时可使用 Quick Merge 防止混入新属性；MA 的新增内容与新图层传播需要结合 Recast、Recipe Cook 理解。[Titan Editing Palettes](https://manual.avolites.com/docs/palettes/editing-palettes)、[MA Edit Presets](https://help.malighting.com/grandMA3/2.5/HTML/presets_edit.html)。

## 对 StageMaster 的吸收建议

以下属于设计建议。统一素材对象可以保存静态值、时间和效果引用，但必须显式记录适用范围、属性掩码、引用方式和版本。Rust 负责解析引用、检查环、计算受影响对象；TypeScript 界面展示变更影响并提交命令。

首版应包含单灯／同灯型素材、跨灯型的基础属性映射、引用更新、合并／替换和依赖查看。复杂插值、嵌套配方可后续增强，但文件模型应预留类型扩展。云端分发时将素材与依赖按版本封装，避免演出中因云端素材更新而改变正在运行的 Cue。
