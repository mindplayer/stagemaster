# 版本基线、新增能力与兼容性

对照功能时，应同时记录软件发行版、手册版本和硬件条件。只有产品名称相同，不能保证两个现场的操作和输出行为相同。当前基线是 grandMA3 2.5.0.3，以及 Titan 19.2；Titan 的稳定在线手册仍标为 19.0，需要结合后续发行说明阅读。[MA 版本目录](https://www.malighting.com/downloads/products/grandMA3/)、[Titan 版本目录](https://www.avolites.com/support/all-titan-pc-suite-downloads/)、[Titan 手册](https://manual.avolites.com/docs/)。

## grandMA3 2.5 的新增与调整

| 功能编号 | 官方变化 | 功能分类 |
| --- | --- | --- |
| M21-MA-01 | Masters Window 集中显示多类总控，并可筛选偏离默认值的对象 | 总控与现场诊断 |
| M21-MA-02 | Locate 在对象池与执行器位置间定位对象，支持跨页查找 | 工作区与对象导航 |
| M21-MA-03 | Phaser Recipe 可在 Edit Recipe 流程中逐步创建，补充速度总控与两步联动编辑 | 动态效果与配方 |
| M21-MA-04 | Channel Map 展示通道属性与灯具结构，帮助区分通道数相同的模式 | 灯具档案与配适 |
| M21-MA-05 | 图形平台改为 Vulkan，系统要求随之调整 | 图形与平台兼容 |
| M21-MA-06 | 默认色彩主题强化选中状态，提供旧版主题 | 工作区与视觉反馈 |

以上是 2.5 功能变化的归纳，各功能的完整对象和操作语义归入对应模块。[MA 2.5 Features](https://help.malighting.com/grandMA3/2.5/HTML/rn_features-2-5.html)。

**术语陷阱：MA3 2.5 的 Locate 是对象定位。** 它不能直接等同于 Titan 把灯具调到便于观察状态的 Locate。功能对照必须根据作用对象与结果匹配，不能只按英文名称匹配。[MA 2.5 Locate 说明](https://help.malighting.com/grandMA3/2.5/HTML/rn_features-2-5.html)、[Titan Locate](https://manual.avolites.com/docs/controlling-fixtures/#setting-fixtures-to-a-start-position-locate)。

2.5 还改变了若干兼容性细节：Remote 用户默认权限由 Admin 改为 Presets；旧 None 权限迁移为 View；OSC 命令中序列名与 Cue 编号之间改用分号；Shapes 在 Data Pool 中的目标编号变化，依赖数字路径的宏需要调整；全新 onPC 的音频输入与输出默认设为 None。[MA 2.5 Changes](https://help.malighting.com/grandMA3/2.5/HTML/rn_changes-2-5.html)。

官方限制中，旧版工程加载会清除编程器内容；Recast 向 Cue 重建预设关联受到绝对层链接条件限制；网络更新 onPC 仍需目标机器确认。这些分别涉及临时编辑状态、引用传播和系统维护流程，不能归为同一类“版本兼容”。[MA 2.5 Known Limitations](https://help.malighting.com/grandMA3/2.5/HTML/rn_knownlimitations-2-5.html)。

## grandMA3 2.5 的兼容性补充

以下补充条目以各行的新版说明为证据，避免旧专题页面覆盖新行为。

| 功能编号 | 官方变化 | 功能分类与依据 |
| --- | --- | --- |
| M21-MA-07 | Lua Core 升到 5.5.0，旧字节码需要重新编译 | 插件兼容：[Other Enhancements](https://help.malighting.com/grandMA3/2.5/HTML/rn_otherenhancements-2-5.html) |
| M21-MA-08 | Presets 权限可更新已有预设，但不允许新建预设或编辑 Recipe 等对象 | 遥控权限：[Other Enhancements](https://help.malighting.com/grandMA3/2.5/HTML/rn_otherenhancements-2-5.html) |
| M21-MA-09 | `HasActivePlayback()` 替换为 `IsRunningPlayback()`；Recipe 清理选项改为 `/Type "Recipe"` | 命令兼容：[Deprecated](https://help.malighting.com/grandMA3/2.5/HTML/rn_deprecated-2-5.html) |
| M21-MA-10 | macOS 安装改为每个软件版本独立 App；支持从应用启动额外实例及 Terminal | 桌面宿主：[Other Enhancements](https://help.malighting.com/grandMA3/2.5/HTML/rn_otherenhancements-2-5.html) |

## Titan 19.0 到 19.2 的关键变化与边界

| 功能编号 | 版本 | 官方变化与边界 |
| --- | --- | --- |
| M21-TI-01 | 19.0 | sACN 输入可驱动对应灯具属性，并用于录制 Cue 和 Palette |
| M21-TI-02 | 19.0 | 可使用 1–9999 的线路编号，总分配上限仍为 64 条；编号范围不等于输出容量或授权额度 |
| M21-TI-03 | 19.0 | 增加释放回 Quick Palette 的选项 |
| M21-TI-04 | 19.0 | 新节目默认关闭 Art-Net 模块；这不表示取消 Art-Net 支持 |
| M21-TI-05 | 19.0 | 备份控台的授权线路数必须不少于主控台 |
| M21-TI-06 | 19.0 | Timeline 的合并与按触发类型替换分开，需关注已有工作流含义 |

这些变化来自 Titan 19.0 发行说明，不能仅凭旧版手册推断当前行为。[Titan 19.0 Release Notes](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.0.pdf)。

19.1 属于维护版，修复 AvoKey、面板更新与部分推杆换页问题。它未给出一套新的功能体系；维护版本仍会影响现场操作可靠性，因此需要保留具体版本号。[Titan 19.1 Release Notes](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.1.pdf)。

| 功能编号 | 版本 | 官方变化与边界 |
| --- | --- | --- |
| M21-TI-07 | 19.2 | 恢复 Video Multi View 和 Group Layout 视频叠加，改用 RTSP；外部流需手工添加 |
| M21-TI-08 | 19.2 | Synergy 视频预览要求 Prism 2.1；新增 D3 Core 与 D3-010 硬件支持 |
| M21-TI-09 | 19.2 | WebAPI 可通过兼容配置恢复部分旧调用方式 |
| M21-TI-10 | 19.2 | 时间码值重新限制在 24 小时内 |

19.2 的已知问题仍包括重负载下短暂输出停顿、部分备份接管状态异常，以及单 Cue 像素效果不遵守渐入时间。功能存在与具体行为可靠性需要分开评价。[Titan 19.2 Release Notes](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.2.pdf)。

较早 D9 的 RTSP 支持还取决于恢复镜像更新，不能只看安装了 19.2 升级包。现场核查需要记录控台型号、硬件批次和系统镜像条件。[D9 官方支持说明](https://www.avolites.com/support/D9/)。

## 跨系统对照规则

| 容易产生的误判 | 正确比较方式 |
| --- | --- |
| 把最新发行版和手册页上的版本标签当成一回事 | 分别登记发行版、文档集合版本与页面标签；发行说明用于解释变化 |
| “有这个按钮”就代表行为一致 | 比较作用对象、状态变化、默认值、时序及失败后的结果 |
| 看到 9999 就当作 9999 个 Universe 的输出能力 | 区分编号空间、实际处理容量、网络承载和许可证上限 |
| 当前手册写着某项功能就认为所有控台都能使用 | 同时检查软件版本、硬件接口、授权与配套软件条件 |
| 修复列表证明产品没有其他问题 | 已修复项、官方已知问题和本项目未做的实测分别记录 |

## 对 StageMaster 的吸收建议

以下属于产品与工程设计判断。

1. **工程语义版本独立于应用版本。** 工程格式、设备执行包、API 契约和程序安装包各自版本化，迁移时明确哪些数据会变化。
2. **功能可用性由能力声明决定。** 分开记录硬件接口、已启用的输出能力和资源预算，避免用对象编号推断实际容量。
3. **操作名称对应确定的对象与结果。** 例如“定位对象”和“灯具归位／定位光束”采用清楚的中文名称，避免跨控台习惯造成误操作。
4. **可见地解释输出。** 集中显示总控、过滤、盲编、输出禁用等会改变现场结果的状态，帮助灯光师回答“这盏灯为什么不亮”。
5. **兼容性进入验收场景。** 包括旧工程迁移、预设属性增删后的引用传播、双机接管、音频设备丢失、跨版本 API 和效果在重负载下的时序。

本模块列出影响功能对照的版本变化；各版本全部修复条目仍以原始发行说明为准，不将修复清单当作独立产品功能重复计数。

## 来源

1. MA Lighting，grandMA3 2.5：[Features](https://help.malighting.com/grandMA3/2.5/HTML/rn_features-2-5.html)、[Changes](https://help.malighting.com/grandMA3/2.5/HTML/rn_changes-2-5.html)、[Known Limitations](https://help.malighting.com/grandMA3/2.5/HTML/rn_knownlimitations-2-5.html)。
2. Avolites，Titan Release Notes：[19.0](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.0.pdf)、[19.1](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.1.pdf)、[19.2](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.2.pdf)。
3. Avolites，[D9 软件与硬件更新条件](https://www.avolites.com/support/D9/)。
