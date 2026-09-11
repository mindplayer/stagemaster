# 宏、命令、插件与自动化

自动化应分成简单重复操作、结构化节目命令、脚本扩展、日历触发和外部 API。它们的权限、状态依赖与时间精度要求不同。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M15-01 | 命令体系 | 命令语法和 Keywords 覆盖对象操作／播放／系统功能 | 控台语法配合内部 Provider／API | [MA Keywords](https://help.malighting.com/grandMA3/2.5/HTML/command_syntax_keywords.html)、[Titan Reference](https://manual.avolites.com/docs/titan-reference) |
| M15-02 | 多步宏 | Macro 行包含命令、等待和启用状态 | 按键录制宏，支持 Real Time／Full Speed | [MA Macros](https://help.malighting.com/grandMA3/2.5/HTML/macros.html)、[Titan Buttons](https://manual.avolites.com/docs/titan-basics/front-panel-buttons) |
| M15-03 | 人工介入步骤 | Go 等待、AddToCmdline、Execute 等控制自动执行与交互 | 录制可重放操作顺序，部分触屏动作不在录制范围 | 同上 |
| M15-04 | 宏库与分配 | 导入库、分配物理键／视图按钮 | 工厂宏、Macro 窗口、Handle／Executor 按钮 | 同上 |
| M15-05 | 变量与作用域 | 用户／全局变量及命令替换 | API 属性及脚本上下文；不按名称直接等同 MA 变量 | [MA Variables](https://help.malighting.com/grandMA3/2.5/HTML/macro_variables.html)、[Titan API](https://api.avolites.com/19.2/) |
| M15-06 | 脚本插件 | Lua Plugin 多组件、权限、版本和源文件管理 | 脚本／宏可调用 Titan API，不是 MA Lua Plugin 模型 | [MA Plugins](https://help.malighting.com/grandMA3/2.5/HTML/plugins.html)、[Titan API](https://api.avolites.com/19.2/) |
| M15-07 | API 对象访问 | Object API、Object-Free API、变量和界面函数 | Providers 提供属性 Get／Set、方法与 Handle 查询 | [MA Lua API](https://help.malighting.com/grandMA3/2.5/HTML/lua_object.html)、[MA Object-Free](https://help.malighting.com/grandMA3/2.5/HTML/lua_objectfree.html)、[Titan API](https://api.avolites.com/19.2/) |
| M15-08 | 插件打包／重载 | Show 内嵌或本地文件、Installed／InStream、ReloadAllPlugins | 按具体宏／脚本部署机制管理，API 客户端可独立于控台 | [MA Plugins](https://help.malighting.com/grandMA3/2.5/HTML/plugins.html)、[Titan API](https://api.avolites.com/19.2/) |
| M15-09 | 日历调度 | Agenda 支持按日历及周期执行对象／命令 | 本次未确认同等原生日历调度器；可研究外部自动化调用 | [MA Agenda](https://help.malighting.com/grandMA3/2.5/HTML/agenda.html)、[Titan API](https://api.avolites.com/19.2/) |
| M15-10 | 计时器 | Stopwatch／Countdown，可关联序列、显示及分配 Executor | Timecode／Timeline 另有时间功能，不当作完全相同对象 | [MA Timers](https://help.malighting.com/grandMA3/2.5/HTML/timers.html) |
| M15-11 | 软件按键 | Quickeys 作为硬键／功能的软入口 | Virtual Panel、触屏按钮及 Key Profiles | [MA Quickeys](https://help.malighting.com/grandMA3/2.5/HTML/quickeys.html)、[Titan Simulator](https://manual.avolites.com/docs/titan-basics/titan-simulator) |
| M15-12 | 节目联动自动化 | Cue Commands、Timecode、Macros 可协作 | Set List 的全局／单曲宏、Playback 和时间线触发 | [MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[Titan Set List](https://manual.avolites.com/docs/running-the-show/set-list-window) |

## 参考目录与兼容边界

MA 完整 Keyword／Lua 主题目录已经纳入 [官方来源索引](source-index.json)。Titan 19.2 API 的方法、属性与类型主题另见 [API 索引](titan-api-index.json)。目录项目不等于独立用户功能，也不表示每项 API 已在实机调用验证。

MA 2.5 发行说明明确将 Lua Core 升到 **5.5.0**，并要求重新编译旧 Lua 字节码；这优先于同一帮助包中仍写着 5.4.x 的概览／插件页面。脚本接口也会调整：`HasActivePlayback()` 已弃用，替代为 `IsRunningPlayback()`。实际部署仍须按目标版本检查 API 与插件行为。[MA 2.5 Other Enhancements](https://help.malighting.com/grandMA3/2.5/HTML/rn_otherenhancements-2-5.html)、[MA 2.5 Deprecated](https://help.malighting.com/grandMA3/2.5/HTML/rn_deprecated-2-5.html)。

Titan 按键录制不是完整 UI 操作录像：只录物理按键和部分触屏操作，改变某些触屏属性不会被记录。D3 Touch 调宏的执行上下文也可能不同。[Titan Front Panel Buttons](https://manual.avolites.com/docs/titan-basics/front-panel-buttons)、[Titan Touch Panels](https://manual.avolites.com/docs/remote-control/programming-touch-panels)。

## 工作流程与 StageMaster 建议

自拟流程：把“选组、调用位置、调用颜色、记录 Cue”保存为自动化；换当前页面／用户／选择后重放，确认依赖显式参数还是隐藏状态。再模拟中途暂停、缺失对象、重复调用和版本变化。

以下属于设计建议。先提供类型化命令和简单宏，复用所有界面所用的核心入口；宏记录语义动作，而不是鼠标坐标。优先使用已选 TypeScript 作为非实时自动化层，Rust 执行受控命令，不为对标 Lua 而增加一种必须维护的语言。

脚本运行时不得同步阻塞 DMX 计算循环。日历任务与节目时间码分开，命令有取消、结果和错误状态；文件／网络权限应按插件能力声明。复杂插件商店和在线分发放在基础单机语义稳定之后。
