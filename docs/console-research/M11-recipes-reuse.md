# Recipe、选择变换与重复利用

本模块比较的是“换了一批灯后，节目能否按原规则重新生成”。MA Recipe 是显式的组合对象；Titan 的素材板、灯具交换、编组和效果编辑提供相近用途，但不能据此宣称有完全同构的 Recipe 系统。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M11-01 | 可重算配方 | Recipe 组合选择、值、变换和时间，存在于 Cue Part／Preset／编程器 | 以 Group、Palette、Shape 等复用；未确认同构 Recipe 对象 | [MA Recipes](https://help.malighting.com/grandMA3/2.5/HTML/recipes.html) |
| M11-02 | 引用选择 | Recipe 可引用 Group；改组后重新 Cook | 效果可编辑应用灯具和顺序；另有 Group Layout | [MA Editor](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[Titan Key Frame](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M11-03 | 引用值素材 | 引用 Preset、Shape、MAtricks | Palette 嵌套、Key Frame 引用 Palette | [MA Editor](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[Titan 创建](https://manual.avolites.com/docs/palettes/creating-palettes)、[Titan Key Frame](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M11-04 | 子灯传播策略 | Normal／Strict 明确是否向子灯传播 | 效果提供整灯／线性子灯／编组布局子灯模式 | [MA Editor](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[Titan Shape](https://manual.avolites.com/docs/effects/shape-generator) |
| M11-05 | 空间变换引用 | MAtricks 池引用及行内覆盖 | Group Layout 与方向／Spread 等共同控制效果 | [MA MAtricks](https://help.malighting.com/grandMA3/2.5/HTML/matricks.html)、[Titan Groups](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups) |
| M11-06 | 随数量适应 | Adaptive Measure／Width；Width 有两步限制 | Beats 可匹配 Spread，不能视为完全相同的自适应算法 | [MA Editor](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[Titan Shape](https://manual.avolites.com/docs/effects/shape-generator) |
| M11-07 | 手工改动与生成内容 | Cook Merge 保留手工内容优先，Overwrite 会重建并移除无关原内容 | 不假定存在相同 Cook 合并语义 | [MA Recipes](https://help.malighting.com/grandMA3/2.5/HTML/recipes.html) |
| M11-08 | 编排模板 | Recipe Template 调入编程器，再应用于当前选择 | 可通过已有素材／Cue 快建效果并改应用灯具 | [MA Recipe Presets](https://help.malighting.com/grandMA3/2.5/HTML/presets_recipes.html)、[Titan Key Frame](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M11-09 | 生成错误可见性 | 颜色和图标显示缺失、不可用、部分应用及 Cook 状态 | 交换映射可查看未匹配属性与函数 | [MA Recipes](https://help.malighting.com/grandMA3/2.5/HTML/recipes.html)、[Titan Exchange](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M11-10 | 换灯型复用 | 灯型、预设模式、PSR 与 Recipe 分别处理对应层次 | Fixture Exchange 保留编排要素，Exchange Mapping 可人工匹配函数和范围 | [MA PSR](https://help.malighting.com/grandMA3/2.5/HTML/sc_psr.html)、[Titan Exchange](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M11-11 | 自动生成基础素材 | Show Creator 可建立组和预设、从灯型产生素材 | 配适时自动灯型组及自动素材能力 | [MA Show Creator](https://help.malighting.com/grandMA3/2.5/HTML/show-creator.html)、[Titan 新配适](https://manual.avolites.com/docs/patching/patching-new-fixtures-or-dimmers) |
| M11-12 | 跨灯复制编程 | Clone 可选 Sequence／Group／Preset／World／Layout，指定源灯与目标灯及对象过滤 | Copy Fixture 带走相关 Cue／Palette；Align 另处理编程器属性 | [MA Clone](https://help.malighting.com/grandMA3/2.5/HTML/operate_clone_overlay.html)、[Titan Copy](https://manual.avolites.com/docs/patching/copying-moving-and-deleting-fixtures) |
| M11-13 | 复制冲突策略 | Clone 有低优先级合并、高优先级合并与覆盖；Show 克隆可带依赖，单独 Programmer 克隆不带 | 属性 Align 可选择分布／重复以及素材引用是否保留；与整 Show 克隆范围不同 | [MA Clone](https://help.malighting.com/grandMA3/2.5/HTML/operate_clone_overlay.html)、[Titan Attributes](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes) |

## 工作流程：巡演灯架从十二台变成二十台

自拟流程：用逻辑组“后排染色”创建波浪效果；替换灯型并增加数量，重建空间顺序，修正颜色素材，再重新生成效果。检查手动加给一台灯的例外是否保留，缺少某属性的新灯是否报告不兼容，以及原来单次扫过的速度是否仍符合节奏。

Titan 的 Fixture Exchange 会尝试匹配属性函数，但官方仍要求按新灯修正素材；固定值编程不能期待自动得到相同视觉效果。MA Recipe 的配方和手工值可以并存，更新策略也必须考虑优先级和来源。[Titan Fixture Exchange](https://manual.avolites.com/docs/patching/changing-the-patch)、[MA Recipes](https://help.malighting.com/grandMA3/2.5/HTML/recipes.html)。

## 对 StageMaster 的吸收建议

以下属于设计建议。把“选择器＋素材引用＋空间变换＋时间分布”作为独立可编译配方模块。生成内容记录来源，手动覆盖另存；界面应展示重算影响和不兼容项。

初期不必复刻全部 Recipe 编辑器，但 Group、Preset、Fixture 的稳定 ID、依赖图和编译接口应从第一版保留。素材在云端复用时应描述所需属性和布局条件，设备下载后先校验能力再激活，不把换灯型视为无损文件转换。
