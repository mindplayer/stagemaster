# 统一功能索引

核查日期：2026-09-10。共 304 条对照记录，其中 284 条功能比较、20 条版本变化；同一能力可出现在版本变化与功能模块中，因此不能把总数当作互不重叠的产品功能数量。

所有条目均为官方文档研究，未实机验证、未实现。完整行为、操作流程和设计判断请点模块链接；“未核实”不表示“不支持”。原始可检索数据见 [JSON 目录](feature-catalogue.json)。

| 模块 | 内容 | 对照记录 |
| --- | --- | --- |
| M01 | [产品体系、输出容量与授权](M01-products-capacity.md) | 10 |
| M02 | [工作区、视图、用户与操作入口](M02-workspace-users.md) | 15 |
| M03 | [工程文件、备份、导入导出](M03-show-files.md) | 12 |
| M04 | [灯具档案、配适与设备维护](M04-fixtures-patch.md) | 19 |
| M05 | [选灯、编组、顺序与空间布局](M05-selection-groups-layout.md) | 11 |
| M06 | [编程器、属性与编辑状态](M06-programmer-attributes.md) | 14 |
| M07 | [预设、素材板与引用更新](M07-presets-palettes.md) | 14 |
| M08 | [Cue、序列、Chase 与跟踪](M08-cues-tracking.md) | 19 |
| M09 | [播放器、优先级、合成与总控](M09-playback-mixing.md) | 16 |
| M10 | [动态效果、Phaser、Shape 与关键帧](M10-effects-phasers.md) | 15 |
| M11 | [Recipe、选择变换与重复利用](M11-recipes-reuse.md) | 13 |
| M12 | [像素映射、视频与媒体协作](M12-pixel-media.md) | 12 |
| M13 | [时间码、时间线、音频与节拍](M13-timecode-audio.md) | 15 |
| M14 | [预览、盲编、3D 与舞台布局](M14-preview-3d.md) | 12 |
| M15 | [宏、命令、插件与自动化](M15-macros-automation.md) | 12 |
| M16 | [网络输出、RDM 与协议接入](M16-dmx-network.md) | 14 |
| M17 | [遥控、外部触发与 API](M17-remotes-api.md) | 17 |
| M18 | [多用户、会话、备份与故障接管](M18-sessions-backup.md) | 15 |
| M19 | [现场演出、Set List 与临时改动](M19-live-show.md) | 14 |
| M20 | [系统设置、维护、诊断与升级](M20-maintenance-diagnostics.md) | 15 |
| M21 | [版本基线、新增能力与兼容性](M21-version-baseline.md) | 20 |
| M22 | [跨系统术语与 StageMaster 吸收建议](M22-glossary-design.md) | 术语、架构与验收建议 |

## M01 · 产品体系、输出容量与授权

详细流程与边界：[模块文档](M01-products-capacity.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M01-01 · 独立控台 | full-size、light、compact／XT、replay unit 等 | D9、D7、D3、Quartz、Tiger Touch、Arena、Sapphire 等 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_calculate.html)、[来源2](https://manual.avolites.com/docs/about-the-consoles) |
| M01-02 · 电脑控制 | onPC 配合授权硬件、控制面和节点 | Titan Go 配合 T1／T2／T3／Mobile 等 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_expand.html)、[来源2](https://manual.avolites.com/docs/about-the-consoles/t1-and-t2) |
| M01-03 · 离线编排 | 不解锁实际输出也能编程及内置 3D 预览 | Simulator 需要相应硬件／AvoKey，模拟输出有周期性 spoiler | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter.html)、[来源2](https://manual.avolites.com/docs/titan-basics/titan-simulator) |
| M01-04 · 容量计量 | Parameter 不等于 DMX 通道；粗／细通道可能只计一个参数 | 以 DMX Line／Universe、系统许可和本机处理能力区分 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M01-05 · 分布计算 | Processing Unit 提供参数及计算扩展 | TNP 分担输出计算，不自行提升系统许可上限 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_expand.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M01-06 · 端口扩展 | 普通 Node 和 onPC 解锁 Node 的角色不同 | 面板／Wing／网络节点增加接口或控制面，不等于增加许可 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_expand.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping)、[来源3](https://manual.avolites.com/docs/about-the-consoles/t3) |
| M01-07 · 无人值守形态 | replay／rack 等角色，运行能力依组合 | D3 Core、TNP Console Mode、开机节目与触发接口 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_calculate.html)、[来源2](https://manual.avolites.com/docs/about-the-consoles/d3)、[来源3](https://manual.avolites.com/docs/titan-net) |
| M01-08 · 平台 | 当前 onPC 文档列 Windows 和 macOS；Windows ARM 不支持 | Titan PC Suite 使用 Windows，移动端遥控另属客户端 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/onpc_system_requirements.html)、[来源2](https://manual.avolites.com/docs/titan-basics/titan-simulator) |
| M01-09 · 第三方可视化授权 | 内置 3D 与第三方可视化输出授权不同，viz-key 有专门用途 | 内置 Capture 与外部完整 Capture 的功能／版本分别核对 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter.html)、[来源2](https://manual.avolites.com/docs/capture-visualiser/capture-show-files) |
| M01-10 · 接口差异 | MIDI、音频、LTC、DC、网络及 DMX 端口依设备 | 官方外部触发表按型号列 Audio／GPIO／MIDI／LTC／WebAPI 等 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/device_overview.html)、[来源2](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |

## M02 · 工作区、视图、用户与操作入口

详细流程与边界：[模块文档](M02-workspace-users.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M02-01 · 多窗口多屏 | Workspace、窗口和 Views；onPC 显示配置 | 各显示器可布置窗口并保存 Workspace | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/workspace.html)、[来源2](https://manual.avolites.com/docs/titan-basics/workspace-windows) |
| M02-02 · 视图召回 | Views 保存显示组织并可分配操作入口 | 可保存单屏／多屏，按原屏或所选屏召回 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/wvm.html)、[来源2](https://manual.avolites.com/docs/titan-basics/workspace-windows) |
| M02-03 · 内容池／库 | Groups、Presets、Sequences、Macros 等池 | Show Library 汇集节目对象及导入内容 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/datapool.html)、[来源2](https://manual.avolites.com/docs/titan-basics/show-library) |
| M02-04 · 内容分区 | 多个 Data Pool 共享配适，可跨池引用 | Show Library、页面和用户工作区分别组织；不视为 Data Pool 等价物 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/datapool.html)、[来源2](https://manual.avolites.com/docs/titan-basics/show-library) |
| M02-05 · 命令入口 | Command Area、语法、历史和命令编辑 | 数字键盘语法、软键及上下文菜单 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/workspace.html)、[来源2](https://manual.avolites.com/docs/titan-reference) |
| M02-06 · 直接操作 | 编码器、Calculator、手势、快捷键 | 属性轮、触屏、软键、键盘快捷键 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/workspace.html)、[来源2](https://manual.avolites.com/docs/titan-basics/front-panel-buttons) |
| M02-07 · 个性化操作键 | Executor Configuration、Quickeys 和宏分配 | Key Profiles 配置 Select／Flash 等按键行为 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/executor_configurations.html)、[来源2](https://manual.avolites.com/docs/titan-basics/front-panel-buttons) |
| M02-08 · 名称与视觉标识 | Label、Scribble 与 Appearance 可分别组织对象的名称和外观 | Legend、Picture Legend、Halo、轨道／对象颜色 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/wvm_pool_label.html)、[来源2](https://help.malighting.com/grandMA3/2.5/HTML/scribbles.html)、[来源3](https://help.malighting.com/grandMA3/2.5/HTML/appear.html)、[来源4](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M02-09 · 页面与固定按钮 | Pages、Executor Bar、池显示选项 | 页／滚动、固定行列、Handle 锁定 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/executor.html)、[来源2](https://manual.avolites.com/docs/titan-basics/workspace-windows) |
| M02-10 · 操作反馈 | 状态颜色、命令反馈、Running Playbacks、Masters Window | 系统提示、属性状态、活动播放与上下文按钮 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/workspace.html)、[来源2](https://manual.avolites.com/docs/titan-basics/workspace-windows) |
| M02-11 · 撤销／恢复 | Oops Menu，具体动作仍有不可撤销边界 | Undo／Redo 与操作历史；并非所有外部动作可逆 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/workspace.html)、[来源2](https://manual.avolites.com/docs/titan-basics/front-panel-buttons) |
| M02-12 · 用户与偏好 | User／User Profile、World 与权限分别配置 | 用户独立工作区和编程器，演出共享内容 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/user.html)、[来源2](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M02-13 · 对象备注 | Notes 可附在 Preset、Group、Macro、Cue、灯型等对象，Info 可查看 | 灯具备注、Set List 曲目备注等按对象提供 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/notes.html)、[来源2](https://manual.avolites.com/docs/patching/changing-the-patch)、[来源3](https://manual.avolites.com/docs/running-the-show/set-list-window) |
| M02-14 · 可复用图形外观 | Scribble 独立成池；Image／Symbol 可用于 Appearance，外观可复用 | Picture Legend 等用于对象识别；本行不认定具有同构独立外观引用图 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/scribbles.html)、[来源2](https://help.malighting.com/grandMA3/2.5/HTML/appear.html)、[来源3](https://help.malighting.com/grandMA3/2.5/HTML/symbols.html)、[来源4](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M02-15 · 跨对象分类与调用 | Tags 可多重附加并引用其他 Tag，也可转发播放命令 | Show Library／分组／Playback Group 分别承担组织或联动，不直接等同 Tags | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/tags.html)、[来源2](https://manual.avolites.com/docs/titan-basics/show-library)、[来源3](https://manual.avolites.com/docs/running-the-show/playback-controls) |

## M03 · 工程文件、备份、导入导出

详细流程与边界：[模块文档](M03-show-files.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M03-01 · 保存与另存 | Save／Save As、Quick Save、内部／外部存储 | Save／Save As、Quick Save、版本标签 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/sfh_save.html)、[来源2](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-02 · 自动保存和历史 | 自动保存及 Backup 文件 | 自动保存、手动／快速／自动版本浏览 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/sfh_backup.html)、[来源2](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-03 · 分项加载 | Show Data、Local Settings、Output Stations、DMX Protocols 等可分开选择 | 可保留现有 DMX 设置或载入文件中的设置 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/sfh_load.html)、[来源2](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-04 · 部分节目导入 | PSR 匹配灯具、选择对象、处理引用和目标池位置 | Import Show 建立灯具映射，从 Show Library 选内容 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/sc_psr.html)、[来源2](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-05 · 灯具映射 | PSR 依据 ID／GUID／名称等匹配并显示冲突 | 源灯可映射到当前节目中的目标灯具 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/sc_psr.html)、[来源2](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-06 · 模板与基础工程 | 模板、演示节目、Show Creator 和池对象导入导出 | 开机标准 Show、节目片段复用 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/sfh_backup.html)、[来源2](https://help.malighting.com/grandMA3/2.5/HTML/import-export.html)、[来源3](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-07 · 灯型随节目 | Show／Fixture Types 及导入导出共同管理 | 已配适 Personality 内嵌于 Show，库更新不自动改变它 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html)、[来源2](https://manual.avolites.com/docs/fixture-personalities) |
| M03-08 · 舞台交换 | MVR 导入／导出及网络交换 | Capture 舞台可独立导入／导出 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_mvr.html)、[来源2](https://manual.avolites.com/docs/capture-visualiser/capture-show-files) |
| M03-09 · 报表 | 可用对象导出／命令相关功能；本次未确认等同 Titan 报表向导 | Create Reports 生成节目和配适资料 | [来源1](https://manual.avolites.com/docs/titan-basics/creating-reports) |
| M03-10 · 异常恢复 | 备份与软件／设备恢复流程分别管理 | Recover Show 尝试读取临时数据；另可载入旧版本 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/sfh_backup.html)、[来源2](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-11 · 版本兼容 | PSR 要求源 Show 保存于相同软件版本；其他迁移看发布说明 | 支持旧 Show 进入新软件；不支持新 Show 可靠回读旧软件 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/sc_psr.html)、[来源2](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-12 · 对象依赖 | 预设／Recipe／跨池引用需连同依赖检查 | Palette／Playback／灯具映射需检查引用结果 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/datapool.html)、[来源2](https://manual.avolites.com/docs/titan-basics/show-library) |

## M04 · 灯具档案、配适与设备维护

详细流程与边界：[模块文档](M04-fixtures-patch.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M04-01 · 灯具档案 | Fixture Type 含模式、通道与物理信息；内置编辑器及 GDTF 来源 | Personality 定义通道、属性和设备行为；独立 Builder 与官方库 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html)、[来源2](https://manual.avolites.com/docs/fixture-personalities) |
| M04-02 · 批量配适 | 灯型／模式、数量、ID、地址等向导 | 厂商／型号／模式、数量、Line、地址、Offset 与 Handle | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_add_fixtures.html)、[来源2](https://manual.avolites.com/docs/patching/patching-new-fixtures-or-dimmers) |
| M04-03 · 多种灯具身份 | Fixture ID、Channel ID／ID Type、名称等 | User Number、Handle、Legend 等 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_add_fixtures.html)、[来源2](https://manual.avolites.com/docs/patching/patching-new-fixtures-or-dimmers) |
| M04-04 · 地址及冲突 | 配适表、Universe 和 DMX Sheet | Patch View 显示占用；冲突可取消或 Park 相关灯具 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_dmx_universe.html)、[来源2](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M04-05 · 多 cell 灯具 | 子灯与几何层级、属性定义 | Super Fixture／Sub-fixture，整体移动并保留 cell 控制 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_what_are_fixtures.html)、[来源2](https://manual.avolites.com/docs/patching/patching-new-fixtures-or-dimmers) |
| M04-06 · 同步地址副本 | Multipatch 跟随主灯，独立地址／3D 位置；不参加 Selection Grid | 多 Dimmer 可配到同一 Handle；不是同一种完整灯具镜像 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_add_multipatch.html)、[来源2](https://manual.avolites.com/docs/patching/patching-new-fixtures-or-dimmers) |
| M04-07 · 独立调光器 | 灯型和属性模型表达控制关系 | Pending Dimmer 可把独立调光通道与灯具合并操作 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html)、[来源2](https://manual.avolites.com/docs/patching/patching-new-fixtures-or-dimmers) |
| M04-08 · 虚拟属性 | 参数可包含虚拟控制及 XYZ 等计算 | Generic RGB 可使用 Virtual Dimmer | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter.html)、[来源2](https://manual.avolites.com/docs/fixture-personalities) |
| M04-09 · 地址重排 | Patch／Live Patch 修改可编辑地址项 | Repatch 保留编程；批量地址、交换地址与布局策略 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_live.html)、[来源2](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M04-10 · 暂停物理配适 | 根据配适及输出功能分别处理 | Park 移出 DMX Map 但保留编排与原地址；不是 Freeze 输出 | [来源1](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M04-11 · 灯型交换 | Fixture Type／PSR 等支持替换和迁移工作流 | Fixture Exchange、函数映射、范围映射 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/sc_psr.html)、[来源2](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M04-12 · 属性修正 | DMX Invert、Encoder Invert、3D Invert、Pan／Tilt Offset 等 | Attribute Behaviour 的反向、冻结、曲线和限制 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_live.html)、[来源2](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M04-13 · 空间／分类信息 | Stage、Layer、Class、位置、旋转、标签与外观 | 可视化位置、灯具标签、备注及颜色 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch.html)、[来源2](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M04-14 · 库与已用版本 | 档案导入／导出、Show 内灯型数据分别处理 | 更新官方库不改变已配适档案；Update Personality 才升级已用版本 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html)、[来源2](https://manual.avolites.com/docs/fixture-personalities) |
| M04-15 · 通用档案救急 | 可创建灯型或采用相应通用灯型 | Generic Multi-DMX、RGB 等提供基本控制 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html)、[来源2](https://manual.avolites.com/docs/fixture-personalities) |
| M04-16 · RDM 发现与维护 | 支持发现、Get／Set、设备信息和地址等，受设备能力限制 | 配适 RDM 工作流及设备信息 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/rdm.html)、[来源2](https://manual.avolites.com/docs/patching/patching-new-fixtures-or-dimmers) |
| M04-17 · 灯具维护动作 | 通过灯型属性／功能和相应命令操作 | Fixture Macro 执行复位、灯泡等动作 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/advanced-options) |
| M04-18 · 复制灯具与节目 | 配适中复制／粘贴可克隆相关 Show 数据，离开配适时执行 | Copy 同时复制 Cue／Palette 数据，新灯具先处于 Park，需重新分配地址 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_clone_patch.html)、[来源2](https://manual.avolites.com/docs/patching/copying-moving-and-deleting-fixtures) |
| M04-19 · 删除灯具的影响 | 按对象与依赖范围分别检查，本行不承诺删除可恢复 | 删除灯具会移除相关编程，手册说明不能撤销；在原 Handle 重新配适不恢复旧编程 | [来源1](https://manual.avolites.com/docs/patching/copying-moving-and-deleting-fixtures) |

## M05 · 选灯、编组、顺序与空间布局

详细流程与边界：[模块文档](M05-selection-groups-layout.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M05-01 · 单灯与范围选择 | 灯具可用 ID 操作；Selection Grid 表示当前选择 | 灯具窗口、布局窗口、实体句柄和数字输入可用于选择 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_what_are_fixtures.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/) |
| M05-02 · 保存灯具组 | Group 保存灯具选择、顺序和网格位置 | Group 保存选择及顺序，可放在池、推杆或按钮句柄上 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/group.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/) |
| M05-03 · 顺序影响编程 | 顺序与 MAtricks、Phaser 的分布相关 | 顺序用于 Prev／Next、Fan、Shape 和 Overlap | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_selection.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/) |
| M05-04 · 编程空间 | Selection Grid 支持三维关系，独立于真实 3D 舞台位置 | Group Layout 使用二维布局，X 位置与 Fixture Order 关联 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_selection.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/) |
| M05-05 · 自动组 | 本行不推断 MA 自动组的等价范围 | 配适时可建立灯型组和本次添加批次组，受用户设置控制 | [来源1](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/) |
| M05-06 · 子灯具选择 | 父子层级可进入不同网格层级，子单元布局受灯具几何描述影响 | 支持多单元灯具与子灯具选择 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_selection.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/) |
| M05-07 · 组总控 | Group 可作为不同类型的 Group Master | Group 放到推杆句柄后可作为强度总控 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/group.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/) |
| M05-08 · 组与已录节目关系 | 普通 Cue／Preset 不保存组引用，Recipe 可以引用组 | 自动组和用于像素效果的组存在特殊删除／解除句柄规则 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/group.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups/) |
| M05-09 · 临时子选择 | MAtricks 可在整体选择中进行子选择；支持两套用户选择 | 支持模式选择、逐灯检查和 Highlight 等工具 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/matricks.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/) |
| M05-10 · 自定义二维操作布局 | Layout 可安排灯具、宏、组和其他池对象，单布局上限 10,000 元素 | Workspace、Group Layout 与对象窗口提供不同组织方式 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/layouts.html)、[来源2](https://manual.avolites.com/docs/titan-basics/workspace-windows)、[来源3](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups) |
| M05-11 · 分布与时间变换 | MAtricks 的各轴有 Block／Group／Wings／Width，以及 Fade／Delay／Speed／Phase 的 From／To、Shuffle／Shift | Fan、顺序、Group Layout 和效果分布参数组合，未确认完全同构池对象 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/matricks.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes)、[来源3](https://manual.avolites.com/docs/effects/shape-generator) |

## M06 · 编程器、属性与编辑状态

详细流程与边界：[模块文档](M06-programmer-attributes.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M06-01 · 临时编辑区 | 每个用户配置有编程器；区分选择、激活值与未激活值 | 选择状态和编程器中已有改动分别显示 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/) |
| M06-02 · 清除动作 | Clear 可依次取消选择、取消激活、释放全部值 | 默认一次清选择和编程器；可配置分步顺序与清除范围；LTP 的恢复还应区分 Clear 与 Release + Clear | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/)、[来源3](https://manual.avolites.com/docs/cue-lists/theatre-programming) |
| M06-03 · 对记录的影响 | 未激活值可能仍参与输出，但通常不作为激活数据录入 | Locate 通常不使原本未进入编程器的属性成为按通道记录的数据 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/#setting-fixtures-to-a-start-position-locate) |
| M06-04 · 局部清除 | 可对属性或特征组执行 Off／移出编程器 | Clear 支持属性掩码、选中灯具范围及单属性清除 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/#clear-button-hold-down-options) |
| M06-05 · 记录来源与范围 | 可选择 Programmer、Output、DMX，并区分激活值／选中灯具／全部 | Record 模式与掩码决定录入哪些数据，需与 Locate、Clear 一起理解 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_store_settings_preferences.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/#setting-fixtures-to-a-start-position-locate) |
| M06-06 · 属性编辑入口 | 编码器、Fixture Sheet、图层和命令等共同操作参数 | 属性轮、Attribute Editor、软键和数值输入；硬件轮配置因型号而异 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/fixture-sheet-dmx-layer.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes/) |
| M06-07 · 专用属性控件 | 独立的颜色、切割片等编辑入口需按对应属性模型组织 | 支持色彩选择、固定色／图案槽、位置、切割片和媒体属性控件 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_fixtures.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes/#attribute-editor-window) |
| M06-08 · 值与时间分层 | 绝对／相对值、Fade／Delay 及输出来源可分别查看 | 可设置灯具／属性时间，清除时可控制时间数据是否保留 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/fixture-sheet-dmx-layer.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes/#setting-fixtureattribute-times) |
| M06-09 · 编程器分部 | Preset 可按设置进入不同 Programmer Part，再存入 Cue Part | 本行不假定存在同语义的 Programmer Part | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/programmer-parts.html) |
| M06-10 · 设备维护动作 | 与普通属性编程、宏和输出适配分别核查 | Fixture Macro 提供 Lamp On／Off、Reset 等设备命令，有些是持续执行的序列 | [来源1](https://manual.avolites.com/docs/controlling-fixtures/advanced-options/) |
| M06-11 · 扇形与数值分布 | Align 按选灯顺序分布属性，带方向与过渡曲线 | Fan 支持曲线、分段、组间／组内分布，修改后值留在编程器 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_align.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes) |
| M06-12 · 灯具间复制属性 | At 或 Clone 可把来源值复制到目标编程器 | Align 复制属性，可选 Spread／Repeat、保留或展开 Palette 引用 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_clone_overlay.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes) |
| M06-13 · 混色与色纸库 | CIE、HSB、色纸，灯具色域、多发射器 Quality、恒亮度和色轮混用策略 | Channel、HSI／RGB／CMY、Picker、Filters，多发射器及色纸号码选择 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_color_picker.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes) |
| M06-14 · 切割片图形编辑 | 插入／旋转与 A+B 模式、连动／平行／镜像、视角与重置 | Blade／Keystone 图形控制，依赖 Personality 中的对应定义 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_shapers.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes) |

## M07 · 预设、素材板与引用更新

详细流程与边界：[模块文档](M07-presets-palettes.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M07-01 · 单灯差异 | Selective 保存指定灯具的数据 | Normal 保存各灯具的数据 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets.html)、[来源2](https://manual.avolites.com/docs/palettes/creating-palettes) |
| M07-02 · 同灯型共享 | Global 以灯型共享数据，可同时有选择性差异 | Shared 对同型号灯具共享 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets.html)、[来源2](https://manual.avolites.com/docs/palettes/creating-palettes) |
| M07-03 · 跨灯型共享 | Universal 根据属性适用性应用 | Global 限于 Dimmer、Pan、Tilt、Colour；颜色转换受灯具能力影响 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets.html)、[来源2](https://manual.avolites.com/docs/palettes/creating-palettes) |
| M07-04 · 分类与记录过滤 | 特征组池有输入过滤，All 池可存多类属性；单对象也能过滤 | 属性掩码和记录模式共同决定素材板内容 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets.html)、[来源2](https://manual.avolites.com/docs/palettes/creating-palettes) |
| M07-05 · 更新引用 | Cue 和其他预设可以引用预设 | Playback 和嵌套 Palette 可以引用素材板 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets_edit.html)、[来源2](https://manual.avolites.com/docs/palettes/editing-palettes) |
| M07-06 · 嵌套素材 | Embedded Preset；深链有更新风险提示 | Nested Palette；Fire Nested 可决定采用引用结果还是原存储值 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets.html)、[来源2](https://manual.avolites.com/docs/palettes/creating-palettes) |
| M07-07 · 更新范围 | Original Content Only 与 Add New Content；新加图层涉及 Recast 或重新 Cook | Merge、Replace、Quick Merge；Quick Merge 限定已有属性 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets_edit.html)、[来源2](https://manual.avolites.com/docs/palettes/editing-palettes) |
| M07-08 · 时间素材 | 可包含时间；只有时间的预设不按通常值引用方式进入 Cue | 可存时间素材；直接调用默认忽略已存时间，可通过 Key Profile 改变 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets.html)、[来源2](https://manual.avolites.com/docs/palettes/timing-with-palettes) |
| M07-09 · 运动素材 | 多步 Phaser Preset、Phaser Recipe | Shape／Key Frame 等效果可进入 Palette | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets_create.html)、[来源2](https://manual.avolites.com/docs/palettes/creating-palettes) |
| M07-10 · 选择变换素材 | Preset 可带 MAtricks；Recipe 可组合选择和素材 | Group、效果中的灯具顺序与素材板配合；不据此推定等同 Recipe | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets_create.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups) |
| M07-11 · 插值素材 | MAgic 定义多个控制点，并按当前选择展开 | 本次未建立与 MAgic 多轴插值完全等价的对象 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets_create.html) |
| M07-12 · 直接现场调用 | 通过预设、选择和编程器应用；动作可配置 | Quick Palette 无选择时作用于相关灯具；效果素材不能这样调用 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets.html)、[来源2](https://manual.avolites.com/docs/palettes/using-palettes) |
| M07-13 · 现场时间控制 | 结合编程器时间、预设时间及播放控制 | 调用 Fade、Overlap、Master Time、Master Overlap | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/masters_grand_time.html)、[来源2](https://manual.avolites.com/docs/palettes/timing-with-palettes) |
| M07-14 · 相关性与依赖查看 | 池图标、引用和 Info 帮助查看对象关系 | 可灰显不适用素材，查看使用它的 Playback | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets.html)、[来源2](https://manual.avolites.com/docs/palettes/using-palettes)、[来源3](https://manual.avolites.com/docs/palettes/editing-palettes) |

## M08 · Cue、序列、Chase 与跟踪

详细流程与边界：[模块文档](M08-cues-tracking.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M08-01 · 基础节目对象 | Sequence 包含 Cue，Cue 内含 Part 和 Step | 独立 Cue、Chase、Cue List 分别组织不同演出习惯 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence.html)、[来源2](https://manual.avolites.com/docs/cue-lists/creating-a-cue-list) |
| M08-02 · 分部 | Cue Part 同次触发，拥有自己的内容和时间 | 本次不把 Cue List 的单灯时间、Autoload 视为同一种 Part | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence.html) |
| M08-03 · 跟踪 | 只显式记录变化，后续继承；可配置序列跟踪 | Cue List 默认启用 Tracking，可关闭 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking.html)、[来源2](https://manual.avolites.com/docs/cue-lists/cue-list-options) |
| M08-04 · 只改当前 Cue | Cue Only 在后续恢复原有状态，涉及 Part 对应关系 | 剧场工作流提供 Cue Only 等更新选项 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking_cue-only.html)、[来源2](https://manual.avolites.com/docs/cue-lists/theatre-programming) |
| M08-05 · 阻断与保护 | Block／Unblock、Break、Tracking Shield 分属不同机制 | Block Cue 阻断前面改动向后传播；不能直接等同 MA 所有保护类型 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking_break.html)、[来源2](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking_shield.html)、[来源3](https://manual.avolites.com/docs/cue-lists/theatre-programming) |
| M08-06 · 有限跟踪范围 | Tracking Distance 可按目标 Cue 或号码差控制范围 | 本次未核实完全等价机制 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking_distance.html) |
| M08-07 · 录入来源 | Programmer／Output／DMX，Active／Selected 等范围 | Channel／Fixture／Stage；Stage 与编程器记录不同 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_store_settings_preferences.html)、[来源2](https://manual.avolites.com/docs/cue-lists/theatre-programming) |
| M08-08 · 复制语义 | Content／Status／Look；跟踪和目的地继承策略可选 | Cue、Chase、Cue List 有复制／移动／链接与删除工作流 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_copy.html)、[来源2](https://manual.avolites.com/docs/cue-lists/copying-moving-linking-and-deleting) |
| M08-09 · 编辑与校核 | Sequence Sheet、Content Sheet、更新及重编号 | Playback View、Unfold、Cue List 表格和语法编辑 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_sheet.html)、[来源2](https://manual.avolites.com/docs/cue-lists/editing-cue-lists) |
| M08-10 · 时间层级 | Cue 进出、特征组、单属性、执行覆盖及动态 Rate | Cue 时间、属性时间、Fixture Overlap、播放时间控制 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_timing.html)、[来源2](https://manual.avolites.com/docs/cue-lists/cue-list-timing) |
| M08-11 · 自动衔接 | Go、Follow、Timed 等触发 | Wait For Go、Link After／With Previous；可全局禁用 Cue Links | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_playback.html)、[来源2](https://manual.avolites.com/docs/cue-lists/cue-list-options) |
| M08-12 · 暗场预定位 | MIB，支持不同预定位时机、目标、时间及抑制 | Move In Dark，序列及单 Cue 设置 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_mib.html)、[来源2](https://manual.avolites.com/docs/cue-lists/cue-list-options) |
| M08-13 · 循环与追逐 | 用序列触发、时间和 Phaser 等组织循环节目 | Chase 有速度、交叉淡化、顺序、循环和步间链接 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[来源2](https://manual.avolites.com/docs/chases/chase-options) |
| M08-14 · 释放与空步 | Release 值与关闭序列等动作各有作用 | Cue Release 可使下一步未编程灯具恢复之前状态 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking.html)、[来源2](https://manual.avolites.com/docs/chases/chase-options) |
| M08-15 · 共享内容 | 多个 Sequence 可共享 Cue 数据，保留独立序列设置和运行位置 | 链接复制与节目复用另按对应对象处理 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence.html)、[来源2](https://manual.avolites.com/docs/cue-lists/copying-moving-linking-and-deleting) |
| M08-16 · 效果延续 | Phaser 融入 Cue／Part 的值与播放体系 | Shape Tracking 可独立控制效果延续，或跟随 Cue Tracking | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence.html)、[来源2](https://manual.avolites.com/docs/cue-lists/cue-list-options) |
| M08-17 · 联动其他播放 | Cue Command 等入口可执行播放命令，按命令状态处理 | Autoload 可加载 Cue／Chase／Cue List，并定义起始步和时间；离开 Cue 后通常停止，除非下一 Cue 继续加载 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[来源2](https://manual.avolites.com/docs/cue-lists/creating-a-cue-list) |
| M08-18 · 追溯更新与批量改动 | 通过跟踪、Update 和表格检查来源 | Update 可追溯到保存硬值的 Cue；支持前向／后向／双向／Cue Only 及跨 Cue 范围合并 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking.html)、[来源2](https://manual.avolites.com/docs/cue-lists/editing-cue-lists) |
| M08-19 · 暂停使用与提取 | Cue 内容和执行状态分别管理，具体操作看对应表格 | 可临时 Disable 跳过 Cue；Include 把单 Cue 载回编程器 | [来源1](https://manual.avolites.com/docs/cue-lists/editing-cue-lists) |

## M09 · 播放器、优先级、合成与总控

详细流程与边界：[模块文档](M09-playback-mixing.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M09-01 · 内容与操作柄分离 | Sequence 不必分配 Executor 也能播放 | Playback 可分配至按钮／推杆；Handle 设置控制操作方式 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_playback.html)、[来源2](https://manual.avolites.com/docs/cues/playback-options) |
| M09-02 · 按键和推杆配置 | Executor Configurations、Assign、自定义功能 | Key Profiles、Cue Fader Modes | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/executor_configurations.html)、[来源2](https://manual.avolites.com/docs/cues/playback-options) |
| M09-03 · 多级优先级 | 多级 LTP、HTP、Swap、Super；Super 可高于编程器 | Low／Normal／High／Programmer／Very High | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_playback.html)、[来源2](https://manual.avolites.com/docs/cues/playback-options) |
| M09-04 · 强度合成 | HTP 对强度取高，其他属性依 LTP；优先级同时参与 | 常规 HTP／LTP；Cross Fade HTP 可改变常规取高行为 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_playback.html)、[来源2](https://manual.avolites.com/docs/cues/playback-options) |
| M09-05 · 推杆接管 | Soft LTP 决定同级序列强度随推杆接管的过渡方式 | Mode 2 可让 HTP／LTP 随推杆，Mode 3 是整体交叉淡化行为 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_playback.html)、[来源2](https://manual.avolites.com/docs/cues/playback-options) |
| M09-06 · 启停条件 | Auto Start／Stop、Restart、Off When Overridden | Kill Point、Cue List Kill At 0／Kill With Off、Fire First Cue | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[来源2](https://manual.avolites.com/docs/cue-lists/cue-list-options) |
| M09-07 · 闪光与临时压制 | Flash、Temp、Swap、Kill 及保护设置 | Flash、Swop、Timed Flash 与优先级配合 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[来源2](https://manual.avolites.com/docs/cues/playback-options) |
| M09-08 · 释放策略 | 关闭、Release、优先级和跟踪需分别处理 | Release Mask／Time 决定哪些属性恢复；Kill 不等于所有 LTP 自动恢复 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking.html)、[来源2](https://manual.avolites.com/docs/cues/playback-options) |
| M09-09 · 手动交叉淡化 | XFade 单／双推杆，Split 与 AB 模式 | Cue List Manual Crossfader，可接管正在执行的淡化 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[来源2](https://manual.avolites.com/docs/cue-lists/cue-list-options) |
| M09-10 · 速度与速率 | Speed 与 Rate 可独立或关联，全局 Master 可统一调节 | Speed／Rate／BPM Masters，效果与 Chase 配置对应来源 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/masters_speed.html)、[来源2](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M09-11 · 效果幅度 | Phaser 参数及 Master／执行操作共同控制 | Size Master、Size On Fader、Speed On Fader、倍率 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[来源2](https://manual.avolites.com/docs/cues/playback-options) |
| M09-12 · 现场时间覆盖 | Exec Time 在触发时取值；Rate 能改变正在运行的时间进程 | 播放／闪光时间、速度主控和现场临时时间 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_timing.html)、[来源2](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M09-13 · 分组与播放总控 | Grand、Group、World、Playback 等 Masters | Grand、Playback、Group 等 Masters，型号及分配方式不同 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/masters.html)、[来源2](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M09-14 · 换页保护 | Executor 页、Auto Fix 与操作分配 | Handle Locked／Transparent Lock／Unlocked | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[来源2](https://manual.avolites.com/docs/cues/playback-options) |
| M09-15 · 活动播放查看 | Running Playbacks、Sequence 状态、来源层 | 活动 Playback、显示窗口及释放操作 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/executor_running_playbacks.html)、[来源2](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M09-16 · 过滤与隔离 | Sequence 输入／输出 Filter 或 World；同一序列的所有 Executor 共用输出过滤 | Playback Blind 只送可视化；Release Mask 是另一种用途 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[来源2](https://manual.avolites.com/docs/cues/playback-options) |

## M10 · 动态效果、Phaser、Shape 与关键帧

详细流程与边界：[模块文档](M10-effects-phasers.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M10-01 · 属性循环 | Phaser 用一个或多个属性的多步值形成循环 | Shape Generator 提供按属性分类的预制效果 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[来源2](https://manual.avolites.com/docs/effects/shape-generator) |
| M10-02 · 自定义步骤 | 编辑 Step，可通过预设构建多步 | Key Frame 可手动录制，或用 Palette／Cue 快速构建 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[来源2](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M10-03 · 绝对与相对 | Phaser 可使用绝对／相对层 | 预制位置 Shape 围绕当前基值运动；Key Frame 定义各帧目标 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[来源2](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M10-04 · 步间过渡 | Width、Transition、Acceleration、Deceleration | 帧时间、宽度、中点、曲线及目标幅度 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[来源2](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M10-05 · 空间分布 | Phase 与 Selection Grid／MAtricks 结合 | Spread／Phase／Offset、选灯顺序及二维方向 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/matricks.html)、[来源2](https://manual.avolites.com/docs/effects/shape-generator) |
| M10-06 · 节拍尺度 | Speed 与 Measure 共同定义周期 | Beats Per Cycle 分频，可匹配帧数或 Spread | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[来源2](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M10-07 · 有限次运行 | NShot、方向、结束保持等 Recipe 参数 | Cycles 可限定次数；Key Frame 可按层设置 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[来源2](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M10-08 · 多属性和多效果 | 属性层与多步、Shape 和 Recipe 配合 | 同灯可运行多个 Shape；Key Frame 有多层效果 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[来源2](https://manual.avolites.com/docs/effects/shape-generator)、[来源3](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M10-09 · 子灯／像素运行 | 选择网格、子灯选择和 Recipe Selection Mode | Super Fixture、Sub Fixture Linear／Group；Key Frame 也可跨 cell | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[来源2](https://manual.avolites.com/docs/effects/shape-generator) |
| M10-10 · 同步与重触发 | Sync 控制可预期的起始相位 | Restart Shapes，可结合相位和节拍设置 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[来源2](https://manual.avolites.com/docs/effects/shape-generator) |
| M10-11 · 停止或屏蔽效果 | Stomp 将相关属性收敛为静态步骤 | Mask FX 屏蔽指定属性／灯具的 Shape、Key Frame 或 Pixel Map | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[来源2](https://manual.avolites.com/docs/effects/shape-generator)、[来源3](https://manual.avolites.com/docs/effects/pixel-mapper) |
| M10-12 · 与其他播放的关系 | 合成、静态调用及序列设置共同决定行为 | Overlay／LTP 控制效果是否被后续属性变化覆盖 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[来源2](https://manual.avolites.com/docs/cues/playback-options) |
| M10-13 · 形状素材 | Shapes 对象含曲线和可选参数，可用于 Phaser Recipe | Shape 文件定义预制效果；Key Frame 是另一编辑方式 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/shapes.html)、[来源2](https://manual.avolites.com/docs/effects/shape-generator) |
| M10-14 · 随机变化 | Random Generator 控制上下界、速度、相位等变化，仅作用于绝对层 | Pixel Mapper 有 Random 动画及多种随机参数 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/generator.html)、[来源2](https://manual.avolites.com/docs/effects/pixel-mapper) |
| M10-15 · 现场幅度和速度 | Phaser 参数与 Speed Master 等配合 | Size／Speed Master、推杆幅度／速度及效果淡入淡出 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[来源2](https://manual.avolites.com/docs/effects/advanced-options) |

## M11 · Recipe、选择变换与重复利用

详细流程与边界：[模块文档](M11-recipes-reuse.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M11-01 · 可重算配方 | Recipe 组合选择、值、变换和时间，存在于 Cue Part／Preset／编程器 | 以 Group、Palette、Shape 等复用；未确认同构 Recipe 对象 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/recipes.html) |
| M11-02 · 引用选择 | Recipe 可引用 Group；改组后重新 Cook | 效果可编辑应用灯具和顺序；另有 Group Layout | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[来源2](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M11-03 · 引用值素材 | 引用 Preset、Shape、MAtricks | Palette 嵌套、Key Frame 引用 Palette | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[来源2](https://manual.avolites.com/docs/palettes/creating-palettes)、[来源3](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M11-04 · 子灯传播策略 | Normal／Strict 明确是否向子灯传播 | 效果提供整灯／线性子灯／编组布局子灯模式 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[来源2](https://manual.avolites.com/docs/effects/shape-generator) |
| M11-05 · 空间变换引用 | MAtricks 池引用及行内覆盖 | Group Layout 与方向／Spread 等共同控制效果 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/matricks.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/fixture-groups) |
| M11-06 · 随数量适应 | Adaptive Measure／Width；Width 有两步限制 | Beats 可匹配 Spread，不能视为完全相同的自适应算法 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[来源2](https://manual.avolites.com/docs/effects/shape-generator) |
| M11-07 · 手工改动与生成内容 | Cook Merge 保留手工内容优先，Overwrite 会重建并移除无关原内容 | 不假定存在相同 Cook 合并语义 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/recipes.html) |
| M11-08 · 编排模板 | Recipe Template 调入编程器，再应用于当前选择 | 可通过已有素材／Cue 快建效果并改应用灯具 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/presets_recipes.html)、[来源2](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M11-09 · 生成错误可见性 | 颜色和图标显示缺失、不可用、部分应用及 Cook 状态 | 交换映射可查看未匹配属性与函数 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/recipes.html)、[来源2](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M11-10 · 换灯型复用 | 灯型、预设模式、PSR 与 Recipe 分别处理对应层次 | Fixture Exchange 保留编排要素，Exchange Mapping 可人工匹配函数和范围 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/sc_psr.html)、[来源2](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M11-11 · 自动生成基础素材 | Show Creator 可建立组和预设、从灯型产生素材 | 配适时自动灯型组及自动素材能力 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/show-creator.html)、[来源2](https://manual.avolites.com/docs/patching/patching-new-fixtures-or-dimmers) |
| M11-12 · 跨灯复制编程 | Clone 可选 Sequence／Group／Preset／World／Layout，指定源灯与目标灯及对象过滤 | Copy Fixture 带走相关 Cue／Palette；Align 另处理编程器属性 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_clone_overlay.html)、[来源2](https://manual.avolites.com/docs/patching/copying-moving-and-deleting-fixtures) |
| M11-13 · 复制冲突策略 | Clone 有低优先级合并、高优先级合并与覆盖；Show 克隆可带依赖，单独 Programmer 克隆不带 | 属性 Align 可选择分布／重复以及素材引用是否保留；与整 Show 克隆范围不同 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_clone_overlay.html)、[来源2](https://manual.avolites.com/docs/controlling-fixtures/changing-fixture-attributes) |

## M12 · 像素映射、视频与媒体协作

详细流程与边界：[模块文档](M12-pixel-media.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M12-01 · 映射坐标 | Bitmap Canvas 映射 Selection Grid | Pixel Mapper 使用 Group Layout | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/bitmap.html)、[来源2](https://manual.avolites.com/docs/effects/pixel-mapper) |
| M12-02 · 内容来源 | 图像、Gobo、Symbol、视频；支持 NDI 输入相关工作流 | 几何元素、文字、手绘、位图和 Synergy 内容 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/bitmap.html)、[来源2](https://manual.avolites.com/docs/effects/pixel-mapper) |
| M12-03 · 内容变换 | 平移、缩放、宽高比、旋转、Clip／Wrap、透明度相关选项 | 元素／层／效果的变换、颜色和不透明度 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/bitmap.html)、[来源2](https://manual.avolites.com/docs/effects/pixel-mapper) |
| M12-04 · 参数映射 | Bitmap Channel 将采样来源映射到属性上下界，不限于 RGB | 以灯具为像素输出二维效果，具体行为取决于灯具档案 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/bitmap.html)、[来源2](https://manual.avolites.com/docs/effects/pixel-mapper) |
| M12-05 · 动画构建 | 通过视频、控制灯具及 Master 改变 Bitmap 行为 | Rotate、Slide、Zoom、Random、渐变、拖尾等动画 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/bitmap.html)、[来源2](https://manual.avolites.com/docs/effects/pixel-mapper) |
| M12-06 · 分层及生成 | 不把多条 Bitmap Configuration 当作同时混合的视频层 | 效果由层、元素、动画组合；有生成数量和结束动作 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/bitmap.html)、[来源2](https://manual.avolites.com/docs/effects/pixel-mapper) |
| M12-07 · 现场控制 | Bitmap Control Fixture 可将参数存为 Cue／Preset | Layer Masters、效果速度和不透明度等可现场控制 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/bitmap.html)、[来源2](https://manual.avolites.com/docs/effects/pixel-mapper) |
| M12-08 · 预览与抑制 | 布局／选择网格辅助定位与预览 | Pixel Mapper Preview、Mask FX、Pre-Spool | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/bitmap.html)、[来源2](https://manual.avolites.com/docs/effects/pixel-mapper) |
| M12-09 · 媒体控制 | 灯具属性模型与媒体相关资源参与节目 | Synergy 控制 Ai／Prism，媒体属性可存 Palette／Cue | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/videos.html)、[来源2](https://manual.avolites.com/docs/synergy) |
| M12-10 · 媒体管理 | 本次未确认与 Synergy 上传／转码同等的服务器管理功能 | Media Browser 上传素材，按目标处理转码及进度 | [来源1](https://manual.avolites.com/docs/synergy/operating-synergy) |
| M12-11 · 灯光与屏幕内容统一 | Bitmap 采样外部媒体输入的工作流 | Lightmap 从 Ai／Prism 层或合成输出映射到灯具 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/bitmap.html)、[来源2](https://manual.avolites.com/docs/synergy/operating-synergy) |
| M12-12 · 外部视频监看 | 依据输入及视频功能分别配置 | Multi View／Overlay 的 RTSP 能力按 19.2 和硬件条件确认 | [来源1](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.2.pdf) |

## M13 · 时间码、时间线、音频与节拍

详细流程与边界：[模块文档](M13-timecode-audio.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M13-01 · 时间节目 | Timecode Show：Track Group、Track、时间段和事件 | Timeline：轨道、Playback 触发与编辑点；Cue List 也可绑定时间码 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timecode.html)、[来源2](https://manual.avolites.com/docs/timelines/creating-a-timeline) |
| M13-02 · 内部与外部时钟 | 内部计时、Timecode Slot、外部信号及生成器 | 内部、系统时间、MIDI／SMPTE、Winamp 等，依硬件配置 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timecode_slots.html)、[来源2](https://manual.avolites.com/docs/cue-lists/cue-list-timing) |
| M13-03 · 外部格式 | SMPTE/LTC、MIDI Timecode、ArtTimeCode；部分可发送 | 外部时间码与接口支持需按机型核对 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timecode_external_connections.html)、[来源2](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M13-04 · 现场录制 | 可录操作事件／推杆；选择手动或全部事件及远程事件 | Live Record 可叠加录制和自动简化推杆点 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timecode_settings.html)、[来源2](https://manual.avolites.com/docs/timelines/creating-a-timeline) |
| M13-05 · 手工编辑 | 事件、轨道、时间段和标记编辑 | 图形／表格编辑、吸附、轮控和手工添加触发 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timecode_events.html)、[来源2](https://manual.avolites.com/docs/timelines/running-and-editing-timelines) |
| M13-06 · 标记导入 | Marker 对象辅助时间节目编排 | 可导入音频编辑器导出的时间标记；格式应为时间而非小节 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timecode_markers.html)、[来源2](https://manual.avolites.com/docs/timelines/creating-a-timeline) |
| M13-07 · 偏移和范围 | Offset、Duration、Auto Start／Stop | Offset、Start／Duration、Activate In Range／Kill Out Of Range | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timecode_settings.html)、[来源2](https://manual.avolites.com/docs/timelines/timeline-options) |
| M13-08 · 循环和独立排练 | 内部时间循环／暂停；外部来源另按设置处理 | 断开外部关联后本地排练；内部源可循环 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timecode_settings.html)、[来源2](https://manual.avolites.com/docs/timelines/timeline-options) |
| M13-09 · 跳转时恢复状态 | Assert Previous Events、Go 或 Goto 状态录制 | 中途进入可能受未执行历史 LTP 影响，Release 设置帮助控制 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timecode_settings.html)、[来源2](https://manual.avolites.com/docs/timelines/running-and-editing-timelines) |
| M13-10 · 停止后的播放 | 可关闭由时间节目启动的播放，或保留 | Timeline Release 可覆盖所触发播放的释放设置 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timecode_settings.html)、[来源2](https://manual.avolites.com/docs/timelines/timeline-options) |
| M13-11 · 音频素材播放 | Sounds Pool，可作为时间码轨道目标 | 手册描述 Winamp 时间源；不据此推定等同完整多轨音频工作站 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/sound_pool.html)、[来源2](https://manual.avolites.com/docs/cue-lists/cue-list-timing) |
| M13-12 · 声控与节拍 | Sound Viewer 波形／频带／节拍，BPM Master 可跟随 | Audio Trigger 和 BPM／Rate 主控，支持范围按硬件 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/sound_viewer.html)、[来源2](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M13-13 · 录制来源控制 | 可选择本用户／全部用户及远程事件 | Timeline 有自己的释放配置，不能假定完全继承当前操作用户 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timecode_settings.html)、[来源2](https://manual.avolites.com/docs/timelines/creating-a-timeline) |
| M13-14 · 时间码槽及配置归属 | 专题页描述 16 个固定 Slot；槽配置属于站点，不随 Show 传递 | 时间源与 Show／硬件的关系须按 Cue List／Timeline 配置分别核查 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timecode_slots.html)、[来源2](https://manual.avolites.com/docs/cue-lists/cue-list-timing) |
| M13-15 · 信号锁定与丢失 | Slot 的 Pre Roll 等待有效信号，After Roll 定义丢失后的继续计时范围 | 本次不承诺同样的锁定／自由运行参数，需按目标接口测试 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timecode_slots.html) |

## M14 · 预览、盲编、3D 与舞台布局

详细流程与边界：[模块文档](M14-preview-3d.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M14-01 · 编程器盲编 | Blind 隔离编程器的输出 | Blind 编程；可用 Blind to Live 过渡 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)、[来源2](https://manual.avolites.com/docs/palettes/using-palettes) |
| M14-02 · 独立节目预览 | Preview 环境可载入序列，切 Cue 和播放 | 单 Playback 可设 Blind，仅输出至 Visualiser | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/preview.html)、[来源2](https://manual.avolites.com/docs/cues/playback-options) |
| M14-03 · 多项预备后过渡 | Preview 与 Live 的环境切换和复制工具 | Scene Master 先组合多项变化，再用推杆送入现场 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/preview.html)、[来源2](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M14-04 · 预览界面提示 | 相关窗口显示预览边框，3D／DMX 可设是否跟随 | Blind、Scene Master 的状态提示与可视化 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/preview.html)、[来源2](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M14-05 · 内置可视化 | 3D Viewer 显示灯具、场景、光束及雾效 | 内置 Capture Visualiser，与配适和节目结合 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_3d_viewer.html)、[来源2](https://manual.avolites.com/docs/capture-visualiser/setting-up-the-rig) |
| M14-06 · 舞台物体与坐标 | Stages、灯具位置／旋转、网格／Mesh 等 | 舞台地面、墙及附加物体、灯具位置／姿态 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_3d_viewer.html)、[来源2](https://manual.avolites.com/docs/capture-visualiser/setting-up-the-rig) |
| M14-07 · 摄像机与视角 | Camera Pool、移动／环绕／适配、多个 Viewer | 摄像机、单视图／四分屏、平移／旋转／环绕 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_3d_viewer.html)、[来源2](https://manual.avolites.com/docs/capture-visualiser/setting-up-the-rig) |
| M14-08 · 渲染性能设置 | Render Quality 和光束／表面等显示设置 | Capture 外观与质量；Throws Light 可减少渲染负担 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_render_quality.html)、[来源2](https://manual.avolites.com/docs/capture-visualiser/setting-up-the-rig) |
| M14-09 · 在场景里选灯 | 3D 可选择灯具，视角投影可生成选择网格 | 在控台选择灯具并编辑可视化位置；具体选择入口依窗口 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_3d_viewer.html)、[来源2](https://manual.avolites.com/docs/capture-visualiser/setting-up-the-rig) |
| M14-10 · 世界坐标编程 | XYZ 可与 Pan／Tilt 结合，MArker 定义目标空间 | 本次未确认同语义世界坐标引擎，不把 3D 摆灯当作 XYZ 编程 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/xyz.html) |
| M14-11 · 场景交换 | MVR／MVR-xchange 关联舞台与灯具资料 | Capture 场景随 Show 保存，也可单独导入导出 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_mvr.html)、[来源2](https://manual.avolites.com/docs/capture-visualiser/capture-show-files) |
| M14-12 · 外部可视化连接 | 节点／协议／viz-key 等需结合授权核对 | Stand-alone Capture 有专门连接流程 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter.html)、[来源2](https://manual.avolites.com/docs/capture-visualiser/linking-the-console-to-stand-alone-capture) |

## M15 · 宏、命令、插件与自动化

详细流程与边界：[模块文档](M15-macros-automation.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M15-01 · 命令体系 | 命令语法和 Keywords 覆盖对象操作／播放／系统功能 | 控台语法配合内部 Provider／API | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/command_syntax_keywords.html)、[来源2](https://manual.avolites.com/docs/titan-reference) |
| M15-02 · 多步宏 | Macro 行包含命令、等待和启用状态 | 按键录制宏，支持 Real Time／Full Speed | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/macros.html)、[来源2](https://manual.avolites.com/docs/titan-basics/front-panel-buttons) |
| M15-03 · 人工介入步骤 | Go 等待、AddToCmdline、Execute 等控制自动执行与交互 | 录制可重放操作顺序，部分触屏动作不在录制范围 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/macros.html)、[来源2](https://manual.avolites.com/docs/titan-basics/front-panel-buttons) |
| M15-04 · 宏库与分配 | 导入库、分配物理键／视图按钮 | 工厂宏、Macro 窗口、Handle／Executor 按钮 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/macros.html)、[来源2](https://manual.avolites.com/docs/titan-basics/front-panel-buttons) |
| M15-05 · 变量与作用域 | 用户／全局变量及命令替换 | API 属性及脚本上下文；不按名称直接等同 MA 变量 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/macro_variables.html)、[来源2](https://api.avolites.com/19.2/) |
| M15-06 · 脚本插件 | Lua Plugin 多组件、权限、版本和源文件管理 | 脚本／宏可调用 Titan API，不是 MA Lua Plugin 模型 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/plugins.html)、[来源2](https://api.avolites.com/19.2/) |
| M15-07 · API 对象访问 | Object API、Object-Free API、变量和界面函数 | Providers 提供属性 Get／Set、方法与 Handle 查询 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/lua_object.html)、[来源2](https://help.malighting.com/grandMA3/2.5/HTML/lua_objectfree.html)、[来源3](https://api.avolites.com/19.2/) |
| M15-08 · 插件打包／重载 | Show 内嵌或本地文件、Installed／InStream、ReloadAllPlugins | 按具体宏／脚本部署机制管理，API 客户端可独立于控台 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/plugins.html)、[来源2](https://api.avolites.com/19.2/) |
| M15-09 · 日历调度 | Agenda 支持按日历及周期执行对象／命令 | 本次未确认同等原生日历调度器；可研究外部自动化调用 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/agenda.html)、[来源2](https://api.avolites.com/19.2/) |
| M15-10 · 计时器 | Stopwatch／Countdown，可关联序列、显示及分配 Executor | Timecode／Timeline 另有时间功能，不当作完全相同对象 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/timers.html) |
| M15-11 · 软件按键 | Quickeys 作为硬键／功能的软入口 | Virtual Panel、触屏按钮及 Key Profiles | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/quickeys.html)、[来源2](https://manual.avolites.com/docs/titan-basics/titan-simulator) |
| M15-12 · 节目联动自动化 | Cue Commands、Timecode、Macros 可协作 | Set List 的全局／单曲宏、Playback 和时间线触发 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[来源2](https://manual.avolites.com/docs/running-the-show/set-list-window) |

## M16 · 网络输出、RDM 与协议接入

详细流程与边界：[模块文档](M16-dmx-network.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M16-01 · 物理 DMX 输出 | 端口分配 Universe，Off／Out／RDM／In | 物理输出节点关联逻辑 Line | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/dmx_port_config.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-02 · Art-Net 输出 | Art-Net 4，广播／单播／自动模式，节点发现与配置 | Art-Net 节点、广播／单播及输出配置 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/dmx_ethernet_artnet.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-03 · sACN 输出 | 单播／组播、Universe、优先级、Preview 等 | sACN 输出、优先级、多网卡及同步地址 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/dmx_ethernet_sacn.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-04 · 内外地址映射 | Local Universe 映射 Art-Net／sACN 编号 | 逻辑 Line 可同时连多个节点，产生相同内容的多个输出 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/dmx_ethernet_sacn.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-05 · 输出启用层级 | 协议总开关、行启用、IdleMaster 输出等 | 模块总开关、节点／Line 分配、自动分配 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/dmx_ethernet_artnet.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-06 · 网络接口选择 | 各协议可选择网卡／Preferred IP | 模块选择输出网卡；多接口需正确地址规划 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/dmx_ethernet_artnet.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping)、[来源3](https://manual.avolites.com/docs/networking/controlling-fixtures-over-a-network) |
| M16-07 · 输入及合并 | DMX、Art-Net、sACN 输入；Prio／HTP／LowTP／Off 等合并策略 | 外部 sACN 合并到输出节点，并设置本机输出相对优先级 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/dmx_port_config.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-08 · 输入用于编程 | 可从 DMX 来源录制，合并与记录分别处理 | 19.0 新增 sACN 输入录制 Cue／Palette 的工作流 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_store_settings_preferences.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-09 · 数据失效／缓存 | 端口 Failure Mode 可保留或超时停止 | 输入状态区分接收／保持／无数据，可清输入缓存 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/dmx_port_config.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-10 · 发送节奏 | 物理 Out 文档列 30 Hz，RDM 模式节奏不同；网络另按规则 | System Render Rate 可设，物理 Break／MAB 等可调 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/dmx_port_config.html)、[来源2](https://manual.avolites.com/docs/system-settings/user-settings)、[来源3](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-11 · 慢设备兼容 | Art-Net 包间延时和输出延时 | Continuous／Overrun／Legacy 等 Art-Net 配置 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/dmx_ethernet_artnet.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-12 · RDM 管理 | 全局和端口启用，发现、参数读写；网络 RDM 依相应协议设置 | 配适发现与各模块 Block RDM 等设置 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/rdm.html)、[来源2](https://help.malighting.com/grandMA3/2.5/HTML/dmx_ethernet_artnet.html)、[来源3](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-13 · 分布输出诊断 | 设备列表、参数授权、缺席设备和 DMX Sheet | DMX Overview 显示 TNP 分配、处理槽和负载 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/dmx_port_config.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-14 · 配置保存与迁移 | 整体或单设备输出配置可导入导出 | Show 加载时选择是否沿用 DMX 设置，缺失节点可重分配 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/dmx_port_config.html)、[来源2](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |

## M17 · 遥控、外部触发与 API

详细流程与边界：[模块文档](M17-remotes-api.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M17-01 · 浏览器遥控 | Web Remote 显示主机的用户屏幕配置和操作能力 | 可用 WebAPI 构建专用 Web 控制页面，不视为同等内置完整控台镜像 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_webremote.html)、[来源2](https://api.avolites.com/19.2/) |
| M17-02 · 手机／平板 App | 浏览器即可连接对应站点 | Titan Remote：Keypad、Fixture、Group、Palette、Cue 等 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_webremote.html)、[来源2](https://manual.avolites.com/docs/remote-control/operating-the-remote) |
| M17-03 · 遥控编程 | 根据登录用户权限操作主机功能 | 遥控可录 Group／Palette／Cue，属性轮和简单 Fan | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_webremote.html)、[来源2](https://manual.avolites.com/docs/remote-control/operating-the-remote) |
| M17-04 · 遥控自身状态 | 可不同用户登录；同 Profile 的命令行可关联 | 遥控有自己的编程器，离开后改动不会因关 App 自动释放 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_webremote.html)、[来源2](https://manual.avolites.com/docs/remote-control/operating-the-remote) |
| M17-05 · 连接和发现 | IP／二维码、分辨率／连接数配置 | 局域网发现及手动 IP，显示响应时间和连接状态 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_webremote.html)、[来源2](https://manual.avolites.com/docs/remote-control/setting-up-the-remote)、[来源3](https://manual.avolites.com/docs/remote-control/operating-the-remote) |
| M17-06 · MIDI 映射 | Note／Attack／Decay／CC 等触发模式 | MIDI Learn，映射硬件动作或节目对象 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/remote_inputs_midi.html)、[来源2](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M17-07 · MSC | 收发、设备／组、Executor 映射 | MSC 预设映射与官方列出的命令子集 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/remote_inputs_msc.html)、[来源2](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M17-08 · DMX／sACN 触发 | DMX Remotes，可设置分辨率及会话变化触发条件 | DMX／sACN 输入映射，区别于把输入直接合并到输出 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/remote_inputs_dmx.html)、[来源2](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M17-09 · 接点触发 | DC Remote 按设备信号起点映射 | GPIO 接点，机型接口不同 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/remote_inputs_dc.html)、[来源2](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M17-10 · OSC | UDP／TCP，推杆／按键／对象及命令行收发；不支持 OSC Bundle | 本次未核实原生同等 OSC；不能把 WebAPI 当作 OSC | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/remote_inputs_osc.html) |
| M17-11 · HTTP API | 本次未核实与 Titan 同构的官方 HTTP 对象 API | HTTP 4430、JSON、Get／Set、脚本方法、Handle 查询 | [来源1](https://api.avolites.com/19.2/) |
| M17-12 · 外部追踪 | PSN 映射追踪点到 MArker，支持轴映射与反向 | 本次未核实同构原生 PSN／XYZ 追踪链路 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/remote_inputs_psn.html) |
| M17-13 · 场馆简化面板 | 可基于用户权限、布局和 Web Remote 配置 | D3 Touch 的分区场景／宏按钮及锁定 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_webremote.html)、[来源2](https://manual.avolites.com/docs/remote-control/programming-touch-panels) |
| M17-14 · 外部控制接管 | 按协议映射和执行对象处理 | Level Match、Set／Fire／Re-Fire At Level 等区别接管与重发 LTP | [来源1](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M17-15 · 音频／DJ 来源 | Sound 输入和 BPM，另可通过协议集成 | Audio Trigger、Pioneer Pro DJ Link Bridge 与 BPM Master | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/sound.html)、[来源2](https://manual.avolites.com/docs/running-the-show/linking-pioneerdj-system-to-titan) |
| M17-16 · API 运行与删除分离 | 不在本次比较中推断同构 HTTP 方法 | `KillPlayback` 停止播放，不是删除；以方法页语义纠正简介中不严谨的描述 | [来源1](https://api.avolites.com/19.2/api/Playbacks.KillPlayback.html) |
| M17-17 · API 强制设置电平 | 以已核实的 OSC／执行器入口为准，不推断 HTTP 等价 | `FirePlaybackAtLevel` 不做 Level Match，未加载时会加载，`alwaysRefire` 可先停止再触发 | [来源1](https://api.avolites.com/19.2/api/Playbacks.FirePlaybackAtLevel.html) |

## M18 · 多用户、会话、备份与故障接管

详细流程与边界：[模块文档](M18-sessions-backup.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M18-01 · 会话 | 多站点加入 Session，共享节目，存在 Master | TitanNet 的 Multi-User、Backup、组合模式 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_session.html)、[来源2](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-02 · 版本条件 | 会话要求相同 Streaming Version，即版本号前三段 | 联网控台要求相同 Titan 版本 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_session.html)、[来源2](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-03 · 用户共享状态 | 同用户／Profile 可共享编程器与相应状态；Screen Configuration 可不同 | User 与 Handle World 决定布局和连接操作的共享方式 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/user.html)、[来源2](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-04 · 多编程器 | 不同 User Profile 各有编程器，输出共同合成 | 用户及 Remote 有独立编程器，可清全部编程器 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/user.html)、[来源2](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-05 · 操作范围 | User Rights 与 World 对灯具／属性范围限制 | Handle World 主要是操作柄布局，不等于 MA World 的灯具范围 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/worldfilter.html)、[来源2](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-06 · 编辑冲突 | Object Ownership 对整个对象或部分内容加临时锁 | 多用户同灯编辑有接管规则；本次未确认同构细粒度对象锁 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/user_ownership.html)、[来源2](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-07 · 锁交接 | Drop／List Ownership，可请求释放当前编辑权 | 不将 Handle 锁定或 Venue Mode 当作对象编辑锁 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/user_ownership.html) |
| M18-08 · 会话加入的数据方向 | 加入站优先级影响谁上传／下载 Show | Slave 加入 Master，接收 Master 的 Show | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_session_master.html)、[来源2](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-09 · 主站选择 | 站优先级、在线时长、IP 等规则决定接管 | 手册明确 Backup Takeover 操作；不承诺同等自动选主 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_session_master.html)、[来源2](https://manual.avolites.com/docs/running-the-show/linking-consoles-for-multi-user-or-backup) |
| M18-10 · 输出责任 | 会话 Master 发送网络 DMX，节点按配置输出 | Master 输出；Takeover 使原主机进入输出禁用状态 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_session.html)、[来源2](https://manual.avolites.com/docs/running-the-show/linking-consoles-for-multi-user-or-backup) |
| M18-11 · 同步与恢复 | Full Tracking Backup 共享运行和编程状态，失联设备可配置重新邀请 | 备机在主机保存／自动保存时同步，也可 Sync Now | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/user.html)、[来源2](https://help.malighting.com/grandMA3/2.5/HTML/network_session.html)、[来源3](https://manual.avolites.com/docs/running-the-show/linking-consoles-for-multi-user-or-backup) |
| M18-12 · 备机能力条件 | 主站与授权参数按系统规则决定 | 备机许可 Line 上限不能小于主机；控制柄数量也影响接管操作 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_expand.html)、[来源2](https://manual.avolites.com/docs/running-the-show/linking-consoles-for-multi-user-or-backup) |
| M18-13 · 连接状态查看 | Session／Station 状态、版本、网络负载及缺失站点 | Sessions View 和备份连接／同步状态 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_session.html)、[来源2](https://manual.avolites.com/docs/running-the-show/linking-consoles-for-multi-user-or-backup) |
| M18-14 · 离开会话 | Leave／Dismiss 是独立操作 | Slave 离开后恢复加入前本地 Show | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_session.html)、[来源2](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-15 · 分布计算设备 | Processing Units 与节点有不同职责 | TNP 处理输出；不把普通协作控台叠加为许可扩容 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_expand.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |

## M19 · 现场演出、Set List 与临时改动

详细流程与边界：[模块文档](M19-live-show.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M19-01 · 多播放即时演出 | Executor、Sequence、Preset、Master 与 Layout 协作 | Playbacks、Quick Palettes、Masters、Shapes 等 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_playback.html)、[来源2](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M19-02 · 歌单组织 | 可组合 Pages／Data Pools／Macros；本次未确认同名原生 Set List 对象 | Set List 将曲目与播放页、备注、Workspace 和宏关联 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/datapool.html)、[来源2](https://manual.avolites.com/docs/running-the-show/set-list-window) |
| M19-03 · 曲目重排与跳过 | 基于已有对象和宏构建，行为由编排决定 | Track 复制／移动／删除、Park Track、多歌单 | [来源1](https://manual.avolites.com/docs/running-the-show/set-list-window) |
| M19-04 · 每首歌恢复入口 | 页面、视图、宏可联动 | 全歌单宏每次切曲执行，单曲宏仅对指定 Track 执行 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/macros.html)、[来源2](https://manual.avolites.com/docs/running-the-show/set-list-window) |
| M19-05 · 互斥播放集合 | Tags 的 Kill Instant／Kill Delayed 分别在新播放启动时或完成淡入后关闭同 Tag 的其他播放，另有引用级 Protect | Playback Groups 可设置互斥及何时关闭其他成员 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/tags.html)、[来源2](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M19-06 · 现场素材过渡 | Programmer／Executor Time、预设和时间图层 | Palette Fade／Overlap、Master Time／Overlap | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/masters_grand_time.html)、[来源2](https://manual.avolites.com/docs/palettes/timing-with-palettes) |
| M19-07 · 预备下一组合 | Preview／Blind 与播放控制 | Scene Master 预备、提交、自动反向／手动提交 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/preview.html)、[来源2](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M19-08 · 临时冻结／静态覆盖 | Freeze、Stomp、Pause 是不同操作 | Shape Mask、Playback 优先级及暂停等 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)、[来源2](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[来源3](https://manual.avolites.com/docs/effects/shape-generator) |
| M19-09 · 分组强度控制 | Group Master 的多种模式 | Scale／HTP／Limit／Take Over／Disabled | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/group.html)、[来源2](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M19-10 · 节拍和速度控制 | Speed／Rate／BPM 及声音来源 | Tap Tempo、BPM／Rate／Size，Pioneer 自动 BPM | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/masters_speed.html)、[来源2](https://manual.avolites.com/docs/running-the-show/linking-pioneerdj-system-to-titan) |
| M19-11 · 换页时继续操作 | Executor 页与 Auto Fix | Handle Lock、Transparent Lock 和换页保持 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[来源2](https://manual.avolites.com/docs/cues/playback-options) |
| M19-12 · 快速定位异常 | 来源层、活动播放、Masters Window、对象 Locate | Active Playbacks、各 Master 状态、其他编程器标记 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/masters.html)、[来源2](https://manual.avolites.com/docs/running-the-show/playback-controls)、[来源3](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M19-13 · 临时节目修改 | Update、Cue Only、Preset 引用和 Preview | Include、Update、Palette Quick Merge 与 Cue List 编辑 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/cue_update.html)、[来源2](https://manual.avolites.com/docs/cues/editing-cues) |
| M19-14 · 场馆日常操作 | 受限用户、布局、Agenda 及运行对象 | Venue Mode、Startup Show／Playback、D3 Touch | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/user.html)、[来源2](https://help.malighting.com/grandMA3/2.5/HTML/agenda.html)、[来源3](https://manual.avolites.com/docs/system-settings/user-settings)、[来源4](https://manual.avolites.com/docs/remote-control/programming-touch-panels) |

## M20 · 系统设置、维护、诊断与升级

详细流程与边界：[模块文档](M20-maintenance-diagnostics.md)。

| 编号／能力 | grandMA3 | Titan | 证据 |
| --- | --- | --- | --- |
| M20-01 · 运行负载 | System Info：每帧计算、界面渲染、CPU、内存、磁盘等 | DMX Overview：处理节点、Line、槽位及负载 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/si_system_info.html)、[来源2](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M20-02 · 硬件健康 | 部分型号显示温度／风扇，onPC 等不提供同样读数 | USB Expert 查看面板、连接和硬件事件 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/si_system_info.html)、[来源2](https://manual.avolites.com/docs/system-settings/usb-expert) |
| M20-03 · 系统消息 | Message／Status Center、System Monitor | 系统提示、诊断工具及版本资料 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_message_center.html)、[来源2](https://manual.avolites.com/docs/system-settings/the-system-menu) |
| M20-04 · 输出检查 | DMX Sheet、Tester、参数和来源优先级 | DMX View、输入列、Patch View | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/patch_dmx_sheet.html)、[来源2](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M20-05 · 网络诊断 | Network Tests、站点状态、接口、协议监视 | IP／适配器、连接状态、端口与 DMX 节点设置 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/network_tests.html)、[来源2](https://manual.avolites.com/docs/networking/ports-used-by-titan) |
| M20-06 · 用户默认行为 | User Settings、Preference and Timings | User Settings 分类设置录制、Clear、Release、时间、效果、轮控等 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/user_settings.html)、[来源2](https://manual.avolites.com/docs/system-settings/user-settings) |
| M20-07 · 显示和控制面 | 显示配置、Desk Lights、本地设置 | 外部显示、触屏、面板亮度、Virtual Hardware | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/local_settings.html)、[来源2](https://manual.avolites.com/docs/system-settings/external-displays) |
| M20-08 · 软件升级 | 按设备类型选择包，可经网络更新多设备 | Upgrade Installer／Recovery Installer，支持保留多个版本的工作流 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/update.html)、[来源2](https://manual.avolites.com/docs/system-settings/upgrading-the-software) |
| M20-09 · 延后激活 | 网络更新文件复制与设备重启激活分开 | 安装、重启及所需恢复镜像按发布说明执行 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/update_network.html)、[来源2](https://manual.avolites.com/docs/system-settings/upgrading-the-software) |
| M20-10 · 控制面固件 | onPC 硬件／节点等有专门更新流程 | USB Expert 自动／手动面板固件更新及按键、推杆检查 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/update_windows_hardware.html)、[来源2](https://manual.avolites.com/docs/system-settings/usb-expert) |
| M20-11 · 灯具库维护 | Fixture Share／GDTF／内置灯型编辑与导入 | 官方 Personality 库、自定义目录、已配适更新 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html)、[来源2](https://manual.avolites.com/docs/fixture-personalities) |
| M20-12 · 恢复与重置 | Clean Start、Panic 等有不同影响范围 | Standard Recovery／Factory Restore／Full Erase | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/ts_clean_start.html)、[来源2](https://manual.avolites.com/docs/system-settings/recovering-reinstalling-the-console) |
| M20-13 · 远程支持数据 | World Server 提供灯型文件和 Crash Log 上传 | 版本、USB Expert、库问题及官方支持流程 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/system_world.html)、[来源2](https://manual.avolites.com/docs/system-settings/usb-expert) |
| M20-14 · 工程文档输出 | 对象导出、截图及信息窗口等 | Reports 支持灯具、Cue、Chase、Cue List、Palette、Group，输出多种文档格式 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/screenshot.html)、[来源2](https://manual.avolites.com/docs/titan-basics/creating-reports) |
| M20-15 · 已知问题与版本要求 | Release Notes、Known Limitations、系统要求 | Release Notes、机型下载和恢复要求 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/release-notes-2-5.html)、[来源2](https://www.avolites.com/support/release-notes/) |

## M21 · 版本基线、新增能力与兼容性

详细流程与边界：[模块文档](M21-version-baseline.md)。

| 编号 | 系统 | 变化 | 证据 |
| --- | --- | --- | --- |
| M21-MA-01 | ma3 | Masters Window 集中显示多类总控，并可筛选偏离默认值的对象 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/rn_features-2-5.html) |
| M21-MA-02 | ma3 | Locate 在对象池与执行器位置间定位对象，支持跨页查找 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/rn_features-2-5.html) |
| M21-MA-03 | ma3 | Phaser Recipe 可在 Edit Recipe 流程中逐步创建，补充速度总控与两步联动编辑 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/rn_features-2-5.html) |
| M21-MA-04 | ma3 | Channel Map 展示通道属性与灯具结构，帮助区分通道数相同的模式 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/rn_features-2-5.html) |
| M21-MA-05 | ma3 | 图形平台改为 Vulkan，系统要求随之调整 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/rn_features-2-5.html) |
| M21-MA-06 | ma3 | 默认色彩主题强化选中状态，提供旧版主题 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/rn_features-2-5.html) |
| M21-MA-07 | ma3 | Lua Core 升到 5.5.0，旧字节码需要重新编译 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/rn_otherenhancements-2-5.html) |
| M21-MA-08 | ma3 | Presets 权限可更新已有预设，但不允许新建预设或编辑 Recipe 等对象 | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/rn_otherenhancements-2-5.html) |
| M21-MA-09 | ma3 | `HasActivePlayback()` 替换为 `IsRunningPlayback()`；Recipe 清理选项改为 `/Type "Recipe"` | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/rn_deprecated-2-5.html) |
| M21-MA-10 | ma3 | macOS 安装改为每个软件版本独立 App；支持从应用启动额外实例及 Terminal | [来源1](https://help.malighting.com/grandMA3/2.5/HTML/rn_otherenhancements-2-5.html) |
| M21-TI-01 | titan | sACN 输入可驱动对应灯具属性，并用于录制 Cue 和 Palette | [来源1](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.0.pdf) |
| M21-TI-02 | titan | 可使用 1–9999 的线路编号，总分配上限仍为 64 条；编号范围不等于输出容量或授权额度 | [来源1](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.0.pdf) |
| M21-TI-03 | titan | 增加释放回 Quick Palette 的选项 | [来源1](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.0.pdf) |
| M21-TI-04 | titan | 新节目默认关闭 Art-Net 模块；这不表示取消 Art-Net 支持 | [来源1](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.0.pdf) |
| M21-TI-05 | titan | 备份控台的授权线路数必须不少于主控台 | [来源1](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.0.pdf) |
| M21-TI-06 | titan | Timeline 的合并与按触发类型替换分开，需关注已有工作流含义 | [来源1](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.0.pdf) |
| M21-TI-07 | titan | 恢复 Video Multi View 和 Group Layout 视频叠加，改用 RTSP；外部流需手工添加 | [来源1](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.2.pdf) |
| M21-TI-08 | titan | Synergy 视频预览要求 Prism 2.1；新增 D3 Core 与 D3-010 硬件支持 | [来源1](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.2.pdf) |
| M21-TI-09 | titan | WebAPI 可通过兼容配置恢复部分旧调用方式 | [来源1](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.2.pdf) |
| M21-TI-10 | titan | 时间码值重新限制在 24 小时内 | [来源1](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.2.pdf) |
