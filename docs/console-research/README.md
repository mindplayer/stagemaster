# grandMA3 与 Avolites Titan 功能研究

本资料库用于专业灯光系统的功能对照与产品设计：将控台能力拆解为稳定的业务概念、现场操作流程、行为差异和可验证的设计要求。StageMaster 的技术框架已经确认，功能研究服务于专业单机系统及后续多端、云端与盒子能力。

## 版本与范围

| 对象 | 本轮基线 | 说明与来源 |
| --- | --- | --- |
| grandMA3 | 软件 2.5.0.3；在线手册 2.5 | 官方下载列表标注 2026-09-01 发布；当前手册入口指向 2.5。[下载列表](https://www.malighting.com/downloads/products/grandMA3/)、[在线手册](https://help.malighting.com/grandMA3/) |
| Avolites Titan | 软件 19.2；在线手册 19.0，加 19.1／19.2 发行说明 | PC Suite 下载列表标注 19.2；稳定版在线手册显示 19.0，不采用 Next 文档作为已发布功能证据。[软件下载](https://www.avolites.com/support/all-titan-pc-suite-downloads/)、[在线手册](https://manual.avolites.com/docs/)、[发行说明目录](https://www.avolites.com/support/release-notes/) |
| 控台与配套设备 | 软件通用能力＋具体型号差异 | 包括 onPC／Titan Go、实体控台、扩展翼、网络处理设备及可视化接入；接口、输出容量与授权单独注明 |
| 旧版本 | 仅用于解释兼容性与功能演进 | 不将 grandMA2、老虎 Classic／非 Titan 系统的行为直接算作当前产品能力 |

核查日期为 2026-09-10。官方手册是功能说明依据，不是性能实测证明；同一手册版本下个别主题仍可能显示较早版本标签，应在来源记录中保留差异。资料中将“官方明确支持”“有条件支持”“概念相近”“尚未核实”分开，未查到不等于不支持。

## 阅读入口与本轮成果

本轮已整理 **22 个模块、304 条对照记录**，包括 284 条功能比较和 20 条版本变化。每条保留原始证据和行为差异；模块另有工作流程、限制及对 StageMaster 的设计建议。版本变化与功能清单可能涉及同一能力，总数不代表互不重叠的功能数量。

- [统一功能索引](feature-matrix.md)：按功能编号比较两家系统，可全文搜索。
- [跨系统术语与吸收建议](M22-glossary-design.md)：51 组术语、模块所有权、阶段建议和 16 个验收场景。
- [来源覆盖表](source-coverage.md)：逐类对应官方目录，说明获取与引用范围。
- [证据差异与未核实项](evidence-issues.md)：新版覆盖旧文档、条件限制与后续设备验证。
- [结构化功能数据](feature-catalogue.json)：供后续筛选、拆需求和建立跟踪工具使用。
- [本轮核查记录](quality-check.md)：编号、表格、来源、链接与研究边界的检查结果。

如果先看专业单机主线，建议依次阅读 M04–M11（配适到编程与播放）、M13（音频时间线）、M16（输出），再用 M22 讨论产品自身的行为与实现顺序。M17／M18 和 M03 连接未来多端、协作与云端需求。

研究依据为官方公开文档；**没有实机操作或性能验证**。本轮完成全模块功能地图及重点行为整理，命令／API 全目录用于查阅，不宣称已经逐项调用或完整翻译原版手册。

## 功能分类

| 编号 | 模块 | 需要回答的专业问题 |
| --- | --- | --- |
| M01 | [产品体系、输出容量与授权](M01-products-capacity.md) | 软件能力、硬件接口、参数／通道／Universe 如何区分？ |
| M02 | [工作区、视图、用户与操作入口](M02-workspace-users.md) | 灯光师如何组织多屏、触摸、快捷键和工作习惯？ |
| M03 | [工程文件、备份、导入导出](M03-show-files.md) | 如何保存、迁移、合并工程，并保留引用关系？ |
| M04 | [灯具档案、配适与设备维护](M04-fixtures-patch.md) | 如何定义模式、通道、子灯具、地址和设备能力？ |
| M05 | [选灯、编组、顺序与空间布局](M05-selection-groups-layout.md) | 同一组灯如何按不同空间与顺序参与编程？ |
| M06 | [编程器、属性、过滤与编辑语义](M06-programmer-attributes.md) | 选中、激活、输出、记录之间是什么关系？ |
| M07 | [预设／素材板与引用更新](M07-presets-palettes.md) | 如何复用位置、颜色和其他灯光数据？ |
| M08 | [Cue、序列、Chase 与跟踪](M08-cues-tracking.md) | 场景怎样存储、继承、修改、释放和组织？ |
| M09 | [播放器、优先级、合成与总控](M09-playback-mixing.md) | 多路播放怎样共同决定最终输出？ |
| M10 | [动态效果、Phaser、Shape 与关键帧](M10-effects-phasers.md) | 波形、步骤、相位、速度与灯具分布怎样协作？ |
| M11 | [Recipe、选择变换与重复利用](M11-recipes-reuse.md) | 更换灯具或调整分组后如何重用编程成果？ |
| M12 | [像素映射、视频与媒体协作](M12-pixel-media.md) | 图像、视频与灯具空间位置怎样映射？ |
| M13 | [时间码、时间线、音频与节拍](M13-timecode-audio.md) | 节目由谁计时，怎样录制、同步和修改事件？ |
| M14 | [预览、盲编、3D 与舞台布局](M14-preview-3d.md) | 如何检查或修改节目而不影响现场输出？ |
| M15 | [宏、命令、插件与自动化](M15-macros-automation.md) | 高频操作如何组合，扩展接口覆盖到什么层次？ |
| M16 | [网络输出、RDM 与协议接入](M16-dmx-network.md) | DMX、Art-Net、sACN 等怎样配置、合并与诊断？ |
| M17 | [遥控、外部触发与 API](M17-remotes-api.md) | 手机、触摸面板、MIDI、OSC 和其他系统怎样接入？ |
| M18 | [多用户、会话、备份与故障接管](M18-sessions-backup.md) | 数据同步、操作权限和输出责任怎样划分？ |
| M19 | [现场演出、Set List 与临时改动](M19-live-show.md) | 如何快速调用、覆盖、恢复和换场？ |
| M20 | [系统设置、维护、诊断与升级](M20-maintenance-diagnostics.md) | 如何识别故障、维护依赖、恢复工程与系统？ |
| M21 | [新版变化与兼容性](M21-version-baseline.md) | 最新软件改变了哪些行为，哪些旧说法不再适用？ |
| M22 | [跨系统术语与 StageMaster 吸收建议](M22-glossary-design.md) | 同名功能是否同义，应该抽象成哪些模块与验收场景？ |

每个模块按“功能清单—操作流程—行为差异—限制条件—设计启示—原始来源”组织。功能存在性和适用范围属于来源事实；对 StageMaster 的模块划分与优先级属于设计判断，分别标识。

## 来源与覆盖记录

[来源索引](source-index.json)保存官方主题标题、目录层级、原始 URL 与获取状态。当前目录索引包含 grandMA3 帮助包的 1,576 个唯一主题链接，以及 Titan 稳定版手册的 125 个主题链接；grandMA3 总数还包含快速指南、硬件手册、命令／插件参考和历史发行说明，不能直接当作功能数量或已研究页数。

索引中的 `indexed` 表示目录已登记，`retrieved` 表示正文已获取，均不代表已经完成分析。模块报告和逐项证据才构成研究结论。来源正文仅用于临时阅读，交付资料采用独立整理与精确引用，不复制整套原版手册。

主要发布者与原始目录：

1. MA Lighting，grandMA3 User Manual 2.5：[当前手册](https://help.malighting.com/grandMA3/)。
2. MA Lighting，grandMA3 Software and Release Notes：[官方版本列表](https://www.malighting.com/downloads/products/grandMA3/)。
3. Avolites，Titan User Manual 19.0：[稳定版手册](https://manual.avolites.com/docs/)。
4. Avolites，Titan PC Suite：[官方版本列表](https://www.avolites.com/support/all-titan-pc-suite-downloads/)。
5. Avolites，Titan Release Notes：[版本变更目录](https://www.avolites.com/support/release-notes/)。
