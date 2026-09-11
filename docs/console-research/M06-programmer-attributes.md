# 编程器、属性与编辑状态

编程器是灯光师的临时工作区。选中了灯、改动参与了现场输出、某个值将被录入 Cue，是三个需要分别解释的问题。两套控台的清除和记录习惯不同，不能只用一个“已修改”标记覆盖全部状态。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M06-01 | 临时编辑区 | 每个用户配置有编程器；区分选择、激活值与未激活值 | 选择状态和编程器中已有改动分别显示 | [MA Programmer](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)、[Titan 选择](https://manual.avolites.com/docs/controlling-fixtures/) |
| M06-02 | 清除动作 | Clear 可依次取消选择、取消激活、释放全部值 | 默认一次清选择和编程器；可配置分步顺序与清除范围；LTP 的恢复还应区分 Clear 与 Release + Clear | 同上；[Titan 剧场清除](https://manual.avolites.com/docs/cue-lists/theatre-programming) |
| M06-03 | 对记录的影响 | 未激活值可能仍参与输出，但通常不作为激活数据录入 | Locate 通常不使原本未进入编程器的属性成为按通道记录的数据 | [MA Programmer](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)、[Titan Locate](https://manual.avolites.com/docs/controlling-fixtures/#setting-fixtures-to-a-start-position-locate) |
| M06-04 | 局部清除 | 可对属性或特征组执行 Off／移出编程器 | Clear 支持属性掩码、选中灯具范围及单属性清除 | [MA Programmer](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)、[Titan Clear](https://manual.avolites.com/docs/controlling-fixtures/#clear-button-hold-down-options) |
| M06-05 | 记录来源与范围 | 可选择 Programmer、Output、DMX，并区分激活值／选中灯具／全部 | Record 模式与掩码决定录入哪些数据，需与 Locate、Clear 一起理解 | [MA Store Settings](https://help.malighting.com/grandMA3/2.5/HTML/cue_store_settings_preferences.html)、[Titan Locate](https://manual.avolites.com/docs/controlling-fixtures/#setting-fixtures-to-a-start-position-locate) |
| M06-06 | 属性编辑入口 | 编码器、Fixture Sheet、图层和命令等共同操作参数 | 属性轮、Attribute Editor、软键和数值输入；硬件轮配置因型号而异 | [MA Programmer Layers](https://help.malighting.com/grandMA3/2.5/HTML/fixture-sheet-dmx-layer.html)、[Titan 属性编辑](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes/) |
| M06-07 | 专用属性控件 | 独立的颜色、切割片等编辑入口需按对应属性模型组织 | 支持色彩选择、固定色／图案槽、位置、切割片和媒体属性控件 | [MA Operate Fixtures](https://help.malighting.com/grandMA3/2.5/HTML/operate_fixtures.html)、[Titan Attribute Editor](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes/#attribute-editor-window) |
| M06-08 | 值与时间分层 | 绝对／相对值、Fade／Delay 及输出来源可分别查看 | 可设置灯具／属性时间，清除时可控制时间数据是否保留 | [MA Programmer Layers](https://help.malighting.com/grandMA3/2.5/HTML/fixture-sheet-dmx-layer.html)、[Titan 属性时间](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes/#setting-fixtureattribute-times) |
| M06-09 | 编程器分部 | Preset 可按设置进入不同 Programmer Part，再存入 Cue Part | 本行不假定存在同语义的 Programmer Part | [MA Programmer Parts](https://help.malighting.com/grandMA3/2.5/HTML/programmer-parts.html) |
| M06-10 | 设备维护动作 | 与普通属性编程、宏和输出适配分别核查 | Fixture Macro 提供 Lamp On／Off、Reset 等设备命令，有些是持续执行的序列 | [Titan Fixture Macros](https://manual.avolites.com/docs/controlling-fixtures/advanced-options/) |
| M06-11 | 扇形与数值分布 | Align 按选灯顺序分布属性，带方向与过渡曲线 | Fan 支持曲线、分段、组间／组内分布，修改后值留在编程器 | [MA Align](https://help.malighting.com/grandMA3/2.5/HTML/operate_align.html)、[Titan Attributes](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes) |
| M06-12 | 灯具间复制属性 | At 或 Clone 可把来源值复制到目标编程器 | Align 复制属性，可选 Spread／Repeat、保留或展开 Palette 引用 | [MA Clone](https://help.malighting.com/grandMA3/2.5/HTML/operate_clone_overlay.html)、[Titan Attributes](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes) |
| M06-13 | 混色与色纸库 | CIE、HSB、色纸，灯具色域、多发射器 Quality、恒亮度和色轮混用策略 | Channel、HSI／RGB／CMY、Picker、Filters，多发射器及色纸号码选择 | [MA Color Picker](https://help.malighting.com/grandMA3/2.5/HTML/operate_color_picker.html)、[Titan Attributes](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes) |
| M06-14 | 切割片图形编辑 | 插入／旋转与 A+B 模式、连动／平行／镜像、视角与重置 | Blade／Keystone 图形控制，依赖 Personality 中的对应定义 | [MA Shapers](https://help.malighting.com/grandMA3/2.5/HTML/operate_shapers.html)、[Titan Attributes](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes) |

本表的“未假定”表示没有在本行建立一对一等价关系，不表示另一套软件缺少可实现相近工作流的功能。

MA Align 对应分布动作，Titan Align 对应复制动作；二者不能按名字直接映射。色彩与切割片的计算结果也受档案准确性及真实灯具机构限制，图形一致不保证物理结果完全一致。[MA Align](https://help.malighting.com/grandMA3/2.5/HTML/operate_align.html)、[MA Color Picker](https://help.malighting.com/grandMA3/2.5/HTML/operate_color_picker.html)、[MA Shapers](https://help.malighting.com/grandMA3/2.5/HTML/operate_shapers.html)。

## 典型工作流程：临时改色后记录

1. 播放一个已有场景，确认现场的初始状态。
2. 选中一部分灯，只修改颜色，观察哪些属性被标记为待记录。
3. 再选择另一部分灯，修改亮度，确认前一批灯的改动是否仍在编程器中。
4. 按“记录激活属性”保存场景，检查有没有意外录入位置或 Locate 值。
5. 只清除当前选择，检查现场是否保持；再释放指定颜色，检查输出回到哪个播放来源。
6. 撤销编辑与撤销录制分别验证，避免将临时工作区和已经保存的节目混为一体。

流程需要先选定一致的录制模式再比较。比如 Titan 使用按通道记录时，直接 Locate 与手动改属性的录入结果不同；已有激活属性在 Locate 后是否继续可记录，还受其清除选项影响。[Titan Locate 与清除选项](https://manual.avolites.com/docs/controlling-fixtures/)。

## 盲编、优先级与预览边界

MA 的 Blind 把编程器与现场输出隔开；在已有编程器值时进入或退出 Blind，可能改变输出，不能把“开启盲编”理解成任何时候都无感切换。Freeze 用于保留编程器对运行序列的覆盖；合成规则仍需结合具体播放优先级理解。[MA Programmer](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)。

Preview 和 Blind 也需要分开建模：前者着眼于预览节目状态，后者控制编辑值是否进入输出。其多用户和播放行为在预览模块中单独比较，不能因名称相近而合并。

## 对 StageMaster 的吸收建议

以下属于设计建议。

核心至少分别表示“被选中”“有临时值”“待记录”“参与输出”“来自哪个来源”。这样可以支持专业用户习惯的清除、局部录制、输出查看和错误定位，同时允许手机界面只展示更少入口。

记录命令应明确包含来源、灯具范围、属性范围、合并方式及时间数据策略。界面可提供合理默认值，但命令本身不能依赖某个窗口隐藏的状态，便于撤销、脚本调用和远程控制采用相同规则。

设备维护命令应使用灯具档案定义的动作与时序，独立于持续效果计算。自拟验收包括：混合灯型操作、连续选择、记录后继续编辑、局部 Clear、Blind 切换、播放期间修改、长时设备动作以及编辑过程中的保存恢复。

## 来源

1. MA Lighting，grandMA3 User Manual 2.5：[The Programmer](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)、[Programmer Layers](https://help.malighting.com/grandMA3/2.5/HTML/fixture-sheet-dmx-layer.html)、[Programmer Parts](https://help.malighting.com/grandMA3/2.5/HTML/programmer-parts.html)、[Store Settings](https://help.malighting.com/grandMA3/2.5/HTML/cue_store_settings_preferences.html)。
2. Avolites，Titan Manual 19.0：[Selecting Fixtures](https://manual.avolites.com/docs/controlling-fixtures/)、[Changing Fixture Attributes](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes/)、[Fixture Advanced Options](https://manual.avolites.com/docs/controlling-fixtures/advanced-options/)。
