# Cue、序列、Chase 与跟踪

Cue 是一次编排状态或变化，Sequence／Cue List 决定它们如何衔接。跟踪不是简单复制上一帧，而是由前面的显式值形成当前状态；修改、插入、复制和删除都可能改变后续结果。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M08-01 | 基础节目对象 | Sequence 包含 Cue，Cue 内含 Part 和 Step | 独立 Cue、Chase、Cue List 分别组织不同演出习惯 | [MA 对象](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence.html)、[Titan Cue List](https://manual.avolites.com/docs/cue-lists/creating-a-cue-list) |
| M08-02 | 分部 | Cue Part 同次触发，拥有自己的内容和时间 | 本次不把 Cue List 的单灯时间、Autoload 视为同一种 Part | [MA 对象](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence.html) |
| M08-03 | 跟踪 | 只显式记录变化，后续继承；可配置序列跟踪 | Cue List 默认启用 Tracking，可关闭 | [MA Tracking](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking.html)、[Titan Options](https://manual.avolites.com/docs/cue-lists/cue-list-options) |
| M08-04 | 只改当前 Cue | Cue Only 在后续恢复原有状态，涉及 Part 对应关系 | 剧场工作流提供 Cue Only 等更新选项 | [MA Cue Only](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking_cue-only.html)、[Titan 剧场编程](https://manual.avolites.com/docs/cue-lists/theatre-programming) |
| M08-05 | 阻断与保护 | Block／Unblock、Break、Tracking Shield 分属不同机制 | Block Cue 阻断前面改动向后传播；不能直接等同 MA 所有保护类型 | [MA Break](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking_break.html)、[MA Shield](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking_shield.html)、[Titan 剧场编程](https://manual.avolites.com/docs/cue-lists/theatre-programming) |
| M08-06 | 有限跟踪范围 | Tracking Distance 可按目标 Cue 或号码差控制范围 | 本次未核实完全等价机制 | [MA Distance](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking_distance.html) |
| M08-07 | 录入来源 | Programmer／Output／DMX，Active／Selected 等范围 | Channel／Fixture／Stage；Stage 与编程器记录不同 | [MA Store](https://help.malighting.com/grandMA3/2.5/HTML/cue_store_settings_preferences.html)、[Titan 剧场编程](https://manual.avolites.com/docs/cue-lists/theatre-programming) |
| M08-08 | 复制语义 | Content／Status／Look；跟踪和目的地继承策略可选 | Cue、Chase、Cue List 有复制／移动／链接与删除工作流 | [MA Copy](https://help.malighting.com/grandMA3/2.5/HTML/cue_copy.html)、[Titan Copy](https://manual.avolites.com/docs/cue-lists/copying-moving-linking-and-deleting) |
| M08-09 | 编辑与校核 | Sequence Sheet、Content Sheet、更新及重编号 | Playback View、Unfold、Cue List 表格和语法编辑 | [MA Sequence Sheet](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_sheet.html)、[Titan 编辑](https://manual.avolites.com/docs/cue-lists/editing-cue-lists) |
| M08-10 | 时间层级 | Cue 进出、特征组、单属性、执行覆盖及动态 Rate | Cue 时间、属性时间、Fixture Overlap、播放时间控制 | [MA Timing](https://help.malighting.com/grandMA3/2.5/HTML/cue_timing.html)、[Titan Timing](https://manual.avolites.com/docs/cue-lists/cue-list-timing) |
| M08-11 | 自动衔接 | Go、Follow、Timed 等触发 | Wait For Go、Link After／With Previous；可全局禁用 Cue Links | [MA Playback](https://help.malighting.com/grandMA3/2.5/HTML/cue_playback.html)、[Titan Options](https://manual.avolites.com/docs/cue-lists/cue-list-options) |
| M08-12 | 暗场预定位 | MIB，支持不同预定位时机、目标、时间及抑制 | Move In Dark，序列及单 Cue 设置 | [MA MIB](https://help.malighting.com/grandMA3/2.5/HTML/cue_mib.html)、[Titan Options](https://manual.avolites.com/docs/cue-lists/cue-list-options) |
| M08-13 | 循环与追逐 | 用序列触发、时间和 Phaser 等组织循环节目 | Chase 有速度、交叉淡化、顺序、循环和步间链接 | [MA Sequence Settings](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[Titan Chase](https://manual.avolites.com/docs/chases/chase-options) |
| M08-14 | 释放与空步 | Release 值与关闭序列等动作各有作用 | Cue Release 可使下一步未编程灯具恢复之前状态 | [MA Tracking](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking.html)、[Titan Chase](https://manual.avolites.com/docs/chases/chase-options) |
| M08-15 | 共享内容 | 多个 Sequence 可共享 Cue 数据，保留独立序列设置和运行位置 | 链接复制与节目复用另按对应对象处理 | [MA Shared Data](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence.html)、[Titan Copy](https://manual.avolites.com/docs/cue-lists/copying-moving-linking-and-deleting) |
| M08-16 | 效果延续 | Phaser 融入 Cue／Part 的值与播放体系 | Shape Tracking 可独立控制效果延续，或跟随 Cue Tracking | [MA 对象](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence.html)、[Titan Options](https://manual.avolites.com/docs/cue-lists/cue-list-options) |
| M08-17 | 联动其他播放 | Cue Command 等入口可执行播放命令，按命令状态处理 | Autoload 可加载 Cue／Chase／Cue List，并定义起始步和时间；离开 Cue 后通常停止，除非下一 Cue 继续加载 | [MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[Titan Autoload](https://manual.avolites.com/docs/cue-lists/creating-a-cue-list) |
| M08-18 | 追溯更新与批量改动 | 通过跟踪、Update 和表格检查来源 | Update 可追溯到保存硬值的 Cue；支持前向／后向／双向／Cue Only 及跨 Cue 范围合并 | [MA Tracking](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking.html)、[Titan Editing](https://manual.avolites.com/docs/cue-lists/editing-cue-lists) |
| M08-19 | 暂停使用与提取 | Cue 内容和执行状态分别管理，具体操作看对应表格 | 可临时 Disable 跳过 Cue；Include 把单 Cue 载回编程器 | [Titan Editing](https://manual.avolites.com/docs/cue-lists/editing-cue-lists) |

## 关键语义

MA 的 Content 复制显式内容，Status 包含跟踪后的状态，Look 再考虑灯具是否点亮。三者解决的任务不同。将它们统一成“复制场景”会让插入节目时发生难以理解的变化。[MA Copy Cues](https://help.malighting.com/grandMA3/2.5/HTML/cue_copy.html)。

Go 到下一 Cue、Goto 指定 Cue、手动交叉淡化也不保证触发相同的重申和命令行为。MA 的 Goto 涉及跟踪值重申；序列设置说明 Temp／Xfade 不执行 Cue Commands。Titan 手动 Cue List 交叉淡化时，Autoload 仍用自身编程时间。[MA Playback](https://help.malighting.com/grandMA3/2.5/HTML/cue_playback.html)、[MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[Titan Options](https://manual.avolites.com/docs/cue-lists/cue-list-options)。

## 工作流程：排练时插入一个场景

自拟流程：先建立三个 Cue，让第一个定义位置、第二个改亮度、第三个改颜色。插入一个只改位置的 Cue，分别选择正常跟踪和 Cue Only，检查第三个 Cue 的最终位置。随后复制显式内容与完整状态到另一序列，比较第一步输出。再测试删除、重编号、倒跳以及从黑场直接跳到最后一个 Cue。

暗场预定位还要检查亮度实际为零的条件、是否被其他播放占用、再次点亮时的目标，以及运动效果是否应在暗场持续。这些是引擎行为，不能仅靠界面上的 MIB 开关解决。

## 对 StageMaster 的吸收建议

以下属于设计建议。Rust 核心应把显式内容、求值后的 Cue 状态、播放实例和输出贡献分开。Cue 号码应是用户可改的标签／排序信息，内部引用使用稳定 ID。插入、删除、更新必须能计算受影响范围并支持撤销。

专业单机首版需要可验证的跟踪、Cue Only、分层时间、Go／Back／Goto、释放、场景复制和基础暗场预定位。Chase 可以在底层复用序列执行器，但界面提供速度／交叉淡化的直接操作。复杂 Shield、Recipe 与多序列共享内容适合在基础语义稳定后扩展。
