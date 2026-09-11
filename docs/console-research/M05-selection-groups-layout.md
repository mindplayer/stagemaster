# 选灯、编组、顺序与空间布局

灯具组不仅解决“选哪些灯”，还可能携带选择顺序、效果分布和空间关系。比较这两套系统时，需要区分灯具身份、当前选择、储存的组、效果排列以及舞台几何位置，避免把它们合并成一个列表。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 对照结论 |
| --- | --- | --- | --- | --- |
| M05-01 | 单灯与范围选择 | 灯具可用 ID 操作；Selection Grid 表示当前选择 | 灯具窗口、布局窗口、实体句柄和数字输入可用于选择 | 选择入口可多样，灯具身份应稳定。来源：[MA 灯具](https://help.malighting.com/grandMA3/2.5/HTML/patch_what_are_fixtures.html)、[Titan 选择](https://manual.avolites.com/docs/controlling-fixtures/) |
| M05-02 | 保存灯具组 | Group 保存灯具选择、顺序和网格位置 | Group 保存选择及顺序，可放在池、推杆或按钮句柄上 | 组不是简单的无序 ID 集合。来源：[MA Groups](https://help.malighting.com/grandMA3/2.5/HTML/group.html)、[Titan Groups](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/) |
| M05-03 | 顺序影响编程 | 顺序与 MAtricks、Phaser 的分布相关 | 顺序用于 Prev／Next、Fan、Shape 和 Overlap | 更改顺序应能预览受影响的效果。来源：[MA Selection Grid](https://help.malighting.com/grandMA3/2.5/HTML/operate_selection.html)、[Titan Groups](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/) |
| M05-04 | 编程空间 | Selection Grid 支持三维关系，独立于真实 3D 舞台位置 | Group Layout 使用二维布局，X 位置与 Fixture Order 关联 | 两种坐标语义不完全相同。来源同上 |
| M05-05 | 自动组 | 本行不推断 MA 自动组的等价范围 | 配适时可建立灯型组和本次添加批次组，受用户设置控制 | Titan 可减少初始编组工作；MA 对应自动化另看 Show Creator。来源：[Titan Groups](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/) |
| M05-06 | 子灯具选择 | 父子层级可进入不同网格层级，子单元布局受灯具几何描述影响 | 支持多单元灯具与子灯具选择 | 灯具与单元需要分层标识。来源：[MA Selection Grid](https://help.malighting.com/grandMA3/2.5/HTML/operate_selection.html)、[Titan 选择](https://manual.avolites.com/docs/controlling-fixtures/) |
| M05-07 | 组总控 | Group 可作为不同类型的 Group Master | Group 放到推杆句柄后可作为强度总控 | 选灯行为和总控行为应分开描述。来源：[MA Groups](https://help.malighting.com/grandMA3/2.5/HTML/group.html)、[Titan Groups](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/) |
| M05-08 | 组与已录节目关系 | 普通 Cue／Preset 不保存组引用，Recipe 可以引用组 | 自动组和用于像素效果的组存在特殊删除／解除句柄规则 | 不应承诺改组会让所有旧节目自动跟着变化。来源：[MA Groups](https://help.malighting.com/grandMA3/2.5/HTML/group.html)、[Titan Groups](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/) |
| M05-09 | 临时子选择 | MAtricks 可在整体选择中进行子选择；支持两套用户选择 | 支持模式选择、逐灯检查和 Highlight 等工具 | 原始选择和临时筛选结果应可区分。来源：[MAtricks](https://help.malighting.com/grandMA3/2.5/HTML/matricks.html)、[Titan 选择](https://manual.avolites.com/docs/controlling-fixtures/) |
| M05-10 | 自定义二维操作布局 | Layout 可安排灯具、宏、组和其他池对象，单布局上限 10,000 元素 | Workspace、Group Layout 与对象窗口提供不同组织方式 | 按钮布局与效果坐标分开。来源：[MA Layouts](https://help.malighting.com/grandMA3/2.5/HTML/layouts.html)、[Titan Workspace](https://manual.avolites.com/docs/titan-basics/workspace-windows)、[Titan Groups](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups) |
| M05-11 | 分布与时间变换 | MAtricks 的各轴有 Block／Group／Wings／Width，以及 Fade／Delay／Speed／Phase 的 From／To、Shuffle／Shift | Fan、顺序、Group Layout 和效果分布参数组合，未确认完全同构池对象 | 选择变换与灯具选择分开保存。来源：[MAtricks](https://help.malighting.com/grandMA3/2.5/HTML/matricks.html)、[Titan Fan](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes)、[Titan Shape](https://manual.avolites.com/docs/effects/shape-generator) |

## 典型工作流程

以下使用一排 12 台灯作为自拟对照场景，不代表两家软件菜单完全相同。

1. 在配适中确认 12 台灯的身份、灯型、模式和地址。
2. 按从舞台左侧到右侧的顺序选择，存为“后区染色灯”组。
3. 建立一个从中间向两边展开的编程排列；真实安装位置保持不变。
4. 用这套排列施加位置展开、亮度延迟或效果相位，检查顺序是否符合现场预期。
5. 保存静态 Cue 后修改组，再比较普通 Cue、引用组的 Recipe 和基于组布局的像素效果各自怎样变化。
6. 用逐灯选择与 Highlight 检查一台灯的位置，然后恢复整组选择。

步骤 5 是关键验收场景：同样是“改了组”，不同对象可能使用值快照、灯具引用或布局引用。界面应说明依赖关系，不能由操作者猜测。

## 重要边界

- **编程排列不等于舞台坐标。** MA 明确区分 Selection Grid 与 3D Viewer；Titan 的组布局也有其编程用途。将空间可视化与效果分布解耦，是合理的共享概念。[MA Selection Grid](https://help.malighting.com/grandMA3/2.5/HTML/operate_selection.html)、[Titan Groups](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/)。
- **句柄不等于对象本体。** Titan 组可以放到不同类型的操作入口上，解除入口与删除对象可能产生不同结果。对象与入口的关系应在演出工作区中可查询。[Titan Groups](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/)。
- **同名动作要按结果对照。** MA3 2.5 的 Locate 定位池对象／执行器；Titan 的 Locate 将灯具调到可观察状态，属于不同功能。[版本与术语核查](M21-version-baseline.md)。

## 对 StageMaster 的吸收建议

以下属于设计建议。

把“选择模块”划分为当前选择、存储的组、临时选择变换和编程坐标四个职责。灯具 ID 与子灯具 ID 由工程模型提供；显示窗口、触摸操作和实体按钮都通过同一选择命令接入。

组至少需要表达成员、顺序与可选的编程布局；实际舞台位置独立保存。是否引用组由使用方明确声明：普通 Cue 存具体灯具数据，配方引用组，像素映射引用布局。修改被引用对象时，应能查看影响范围。

建议验收：组成员增删、顺序反转、空组、重复选择、父子灯具混选、跨页面选择、布局重叠以及撤销恢复。每项都检查灯具身份、当前选择、节目数据与现场输出是否发生了预期变化。

## 来源

1. MA Lighting，grandMA3 User Manual 2.5：[Groups](https://help.malighting.com/grandMA3/2.5/HTML/group.html)、[Selection Grid](https://help.malighting.com/grandMA3/2.5/HTML/operate_selection.html)、[MAtricks](https://help.malighting.com/grandMA3/2.5/HTML/matricks.html)。
2. Avolites，Titan Manual 19.0：[Selecting Fixtures](https://manual.avolites.com/docs/controlling-fixtures/)、[Fixture Groups](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/)。
