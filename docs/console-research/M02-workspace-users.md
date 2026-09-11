# 工作区、视图、用户与操作入口

专业控台既要支持快速肌肉记忆操作，也要让复杂状态可见。这里把显示布局、内容组织、输入动作和用户偏好拆开，避免将“控台界面”视为一块固定的大屏。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M02-01 | 多窗口多屏 | Workspace、窗口和 Views；onPC 显示配置 | 各显示器可布置窗口并保存 Workspace | [MA Workspace](https://help.malighting.com/grandMA3/2.5/HTML/workspace.html)、[Titan Workspace](https://manual.avolites.com/docs/titan-basics/workspace-windows) |
| M02-02 | 视图召回 | Views 保存显示组织并可分配操作入口 | 可保存单屏／多屏，按原屏或所选屏召回 | [MA Windows](https://help.malighting.com/grandMA3/2.5/HTML/wvm.html)、[Titan Workspace](https://manual.avolites.com/docs/titan-basics/workspace-windows) |
| M02-03 | 内容池／库 | Groups、Presets、Sequences、Macros 等池 | Show Library 汇集节目对象及导入内容 | [MA Data Pools](https://help.malighting.com/grandMA3/2.5/HTML/datapool.html)、[Titan Library](https://manual.avolites.com/docs/titan-basics/show-library) |
| M02-04 | 内容分区 | 多个 Data Pool 共享配适，可跨池引用 | Show Library、页面和用户工作区分别组织；不视为 Data Pool 等价物 | 同上 |
| M02-05 | 命令入口 | Command Area、语法、历史和命令编辑 | 数字键盘语法、软键及上下文菜单 | [MA Workspace](https://help.malighting.com/grandMA3/2.5/HTML/workspace.html)、[Titan Reference](https://manual.avolites.com/docs/titan-reference) |
| M02-06 | 直接操作 | 编码器、Calculator、手势、快捷键 | 属性轮、触屏、软键、键盘快捷键 | [MA Workspace](https://help.malighting.com/grandMA3/2.5/HTML/workspace.html)、[Titan Buttons](https://manual.avolites.com/docs/titan-basics/front-panel-buttons) |
| M02-07 | 个性化操作键 | Executor Configuration、Quickeys 和宏分配 | Key Profiles 配置 Select／Flash 等按键行为 | [MA Executor](https://help.malighting.com/grandMA3/2.5/HTML/executor_configurations.html)、[Titan Buttons](https://manual.avolites.com/docs/titan-basics/front-panel-buttons) |
| M02-08 | 名称与视觉标识 | Label、Scribble 与 Appearance 可分别组织对象的名称和外观 | Legend、Picture Legend、Halo、轨道／对象颜色 | [MA Label](https://help.malighting.com/grandMA3/2.5/HTML/wvm_pool_label.html)、[MA Scribbles](https://help.malighting.com/grandMA3/2.5/HTML/scribbles.html)、[MA Appearance](https://help.malighting.com/grandMA3/2.5/HTML/appear.html)、[Titan Patch](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M02-09 | 页面与固定按钮 | Pages、Executor Bar、池显示选项 | 页／滚动、固定行列、Handle 锁定 | [MA Executor](https://help.malighting.com/grandMA3/2.5/HTML/executor.html)、[Titan Workspace](https://manual.avolites.com/docs/titan-basics/workspace-windows) |
| M02-10 | 操作反馈 | 状态颜色、命令反馈、Running Playbacks、Masters Window | 系统提示、属性状态、活动播放与上下文按钮 | [MA Workspace](https://help.malighting.com/grandMA3/2.5/HTML/workspace.html)、[Titan Workspace](https://manual.avolites.com/docs/titan-basics/workspace-windows) |
| M02-11 | 撤销／恢复 | Oops Menu，具体动作仍有不可撤销边界 | Undo／Redo 与操作历史；并非所有外部动作可逆 | [MA Oops](https://help.malighting.com/grandMA3/2.5/HTML/workspace.html)、[Titan Buttons](https://manual.avolites.com/docs/titan-basics/front-panel-buttons) |
| M02-12 | 用户与偏好 | User／User Profile、World 与权限分别配置 | 用户独立工作区和编程器，演出共享内容 | [MA Users](https://help.malighting.com/grandMA3/2.5/HTML/user.html)、[Titan Users](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M02-13 | 对象备注 | Notes 可附在 Preset、Group、Macro、Cue、灯型等对象，Info 可查看 | 灯具备注、Set List 曲目备注等按对象提供 | [MA Notes](https://help.malighting.com/grandMA3/2.5/HTML/notes.html)、[Titan Patch](https://manual.avolites.com/docs/patching/changing-the-patch)、[Titan Set List](https://manual.avolites.com/docs/running-the-show/set-list-window) |
| M02-14 | 可复用图形外观 | Scribble 独立成池；Image／Symbol 可用于 Appearance，外观可复用 | Picture Legend 等用于对象识别；本行不认定具有同构独立外观引用图 | [MA Scribbles](https://help.malighting.com/grandMA3/2.5/HTML/scribbles.html)、[MA Appearance](https://help.malighting.com/grandMA3/2.5/HTML/appear.html)、[MA Symbols](https://help.malighting.com/grandMA3/2.5/HTML/symbols.html)、[Titan Patch](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M02-15 | 跨对象分类与调用 | Tags 可多重附加并引用其他 Tag，也可转发播放命令 | Show Library／分组／Playback Group 分别承担组织或联动，不直接等同 Tags | [MA Tags](https://help.malighting.com/grandMA3/2.5/HTML/tags.html)、[Titan Library](https://manual.avolites.com/docs/titan-basics/show-library)、[Titan Playback](https://manual.avolites.com/docs/running-the-show/playback-controls) |

## 对象组织边界

MA 的 Data Pool 是节目内容分区，不是操作系统文件夹或独立输出引擎。复制后仍可能存在跨池引用，不能默认得到依赖全部内嵌的独立副本。[MA Data Pools](https://help.malighting.com/grandMA3/2.5/HTML/datapool.html)。

Titan 固定行列布局用于避免不同屏幕尺寸使按钮位置改变；Workspace 召回可以只作用于某个屏幕。这一点与未来桌面、平板的适配直接相关：共享对象顺序和操作语义，不必共享每个像素的位置。[Titan Workspace Windows](https://manual.avolites.com/docs/titan-basics/workspace-windows)。

## 工作流程与 StageMaster 建议

自拟流程：建立“配适”“编程”“演出”三套视图，放入同一组灯和同一组素材；换成单屏／双屏／平板尺寸，检查对象编号、排序、固定播放和选择状态。用触屏、键盘和硬件按钮分别调用同一动作，核对结果一致。

以下属于设计建议。React 共享控件、命令类型和状态订阅；Workspace 只是布局配置，不拥有节目数据。用户偏好、工程内容、设备设置分别存储。首版保留命令入口、可保存视图、稳定对象标识、清晰状态反馈和基本撤销；复杂多用户布局管理可以后续增强。
