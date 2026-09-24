# StageMaster 产品蓝图：取长补短后的统一系统

状态：产品方向 v0.4，2026-09-24；保持专业预演、场地采集、外部音视频控制／监看及实体控台方向，按当前决定校正交付顺序。它把 [grandMA3／Titan 功能研究](console-research/README.md)与 [Depence R4 重点对照](depence-r4-assessment.md)转成 StageMaster 自己的设计方向。详细边界以[架构设计 v0.5](architecture.md)及专项补充为准；能力覆盖、模块／界面落位和吸收规则统一见[产品能力总表](product-capability-plan.md)，实际完成范围见[实现状态](implementation-status.md)。

产品定位为专业舞台编排、预演与演出控制系统。专业控台、连接外部控台的独立预演、编排与预演一体三种模式共享身份与契约，分别运行所需模块。UE5 专业渲染与高斯场地采集当前均为待验证候选。

产品形态包括未来[专业实体控台与扩展翼](hardware-control-surfaces.md)，推子、编码器、按键与反馈通过独立控制面接入。演出音视频的播放、混音、合成、映射与正式输出由外部系统承担；StageMaster 提供[编排、控制和监听／监看](audiovisual-stage-design.md)，不因专业预演目标而内置完整媒体服务器。

## “最先进”的判断标准

StageMaster 不以按钮最多作为先进。它需要同时做到：复杂节目表达力强、临场操作快、输出可解释、现场链路稳定、工程可长期迁移、同一内容能安全进入桌面／平板／Web／盒子和云端。

| 目标 | 吸收的成熟经验 | StageMaster 的统一方向 |
| --- | --- | --- |
| 复杂节目 | MA3 的稳定对象身份、引用、跟踪、Recipe、Phaser、Selection Grid | 所有编程内容进入一份可检查的语义图，修改前能显示影响范围 |
| 快速编程和跑场 | Titan 的 Palette、Fan、Shape、Tracking View、Set List、Scene Master | 同一底层语义提供快速直接入口和深度编辑入口，适合触屏与实体控制面 |
| 音乐演出 | 两家的时间码与播放控制，Titan Timeline 的直观编辑 | 音频波形、标记、Cue、推杆包络、节拍和效果相位共享时间模型，按同步组关联或独立运行 |
| 专业预演与交付 | Depence R4 的灯具／多设备仿真、视频投影、镜头、场景和图纸流程 | 中立场景＋独立仿真＋可替换渲染后端，预演、编排与图纸引用同一版本化数据 |
| 现场诊断 | MA3 的来源／Master／表格体系，Titan 的活动播放与输出视图 | 任意灯具属性都能回答“现在为什么是这个值”，并追到来源和物理通道 |
| 多设备 | Web Remote、Titan Remote、Session／Backup | 桌面、移动、Web 共用命令；现场引擎独立持有输出和主时钟 |
| 内容交付 | Show 保存／导入、灯具库和本地备份经验 | 可编辑工程、不可变发布包、已下载版本、正在运行版本明确分开 |

## 八个产品支柱

1. **确定性引擎。** 值、相位、事件顺序和随机种子有明确表示；同一计划、输入和时钟轨迹在支持目标上按声明的精度得到一致逻辑结果，物理时序单独验证。这是目标，当前原型只验证局部整数运算。
2. **可解释输出。** 通过帧号、来源句柄和编译来源索引反查灯具、属性、Cue、效果、编程器、主控和外部输入；诊断按需展开，不要求每帧构造完整解释树。
3. **引用优先。** Group、Preset、Cue、Effect、Recipe、Timeline 形成显式依赖；用户可以选择保留引用、展开硬值或重算。
4. **修改前看影响。** 更新素材、Cue Only、换灯、克隆和工程迁移先生成差异与传播范围，再提交为可撤销命令。
5. **编排与演出双界面。** 深度表格／命令和直观画布／触屏共用同一对象，不复制业务逻辑；允许灯光师按场景切换工作方式。
6. **现场离线自治。** UI、云端或网络失效时，已准备的本地节目继续按策略运行；文件和日志操作不能进入输出关键路径。
7. **一次建模，多端适配。** Rust 持有语义和执行，TypeScript＋React 持有共享界面与客户端流程；平台能力在适配层声明。
8. **专业预演共用工程。** 场地外观、几何、灯具和媒体有明确身份；仿真消费已求值输入、渲染呈现结果。独立接入外部控台时通过来源适配映射，始终区分模拟与真实输出。

## 核心对象模型

```text
Project
├── FixtureLibraryRef ── FixtureProfile ── Attribute / Geometry / DMX mapping
├── Patch ── FixtureInstance ── OutputRoute / Calibration
├── Group ── ordered fixture selection ── SelectionLayout
├── Preset ── value / timing / effect references and scope
├── Sequence ── Cue ── Part ── tracked ProgrammedValue
├── Effect ── steps / curve / phase / distribution / termination
├── Recipe ── selection + preset/effect + transform
├── PlaybackSurface ── page / executor / control binding
├── Timeline ── clock source / reference / marker / external event / envelope
├── ExternalDevice ── stable target / capability requirements / content reference
├── MonitorBinding ── logical source / tap / virtual surface
├── VenueSceneRef ── pinned revision / geometry / captured appearance / scale
├── StageScene ── installation / transforms / cameras / video surfaces
├── SimulationModelRef ── behavior version / capabilities / calibration
├── PlotDocument ── pinned revisions / views / symbols / reports
└── SetList ── track / sequence / workspace / preparation action
```

永久 ID、用户显示编号、按钮位置和 DMX 地址是四种不同身份。领域值应区分归一化比例、物理单位、带符号相对值、颜色与离散功能，另设时间类型；精度、舍入与限幅需要契约。灯具领域与输出映射分开，在输出端转换为 DMX 或其他协议；效果和 Cue 不直接写串口字节。当前 u16 原型不作为所有未来值的唯一表示。

PlaybackSurface 保存逻辑页与执行器；现场硬件档案、校准、实际设备绑定和个人布局独立保存。节目可引用逻辑外部设备／监看源，连接地址、凭据、短期会话和 GPU 句柄不写入工程。换扩展翼或媒体服务器时重新绑定，不改 Cue 的业务身份。

## 现场数据流

```text
编辑命令 → 工程修订 → 校验／节目编译 → 不可变执行计划
                                            ↓
控制命令／时钟 → 播放实例 → 跟踪与效果求值 → HTP/LTP/优先级合成
                                                   ↓
                     来源诊断 ← 属性输出快照 → DMX编码 → 输出适配器 → RS485/网络
```

UI 只提交语义命令并订阅快照。实时引擎不访问云数据库、React 状态或任意工程文件。物理 DMX 适配器拥有 Break／MAB／帧发送时序；引擎提供完整帧和时间信息。

## 与现有控台相比要进一步做好的地方

- 默认显示高影响状态：盲编／预演、总控偏离、输出禁用、外部输入、暂停配适或属性冻结、未匹配属性、缺媒体和版本不兼容；不同厂商的相似术语不能直接混用。
- 把 Tracking View 与输出来源合成一条检查链：既看 Cue 从哪里继承，也看最终被哪一路覆盖。
- 将 MA3 Recipe 的可重算能力和 Titan 的快速 Shape／Fan 工作流放在同一个效果模型上。
- 让音频波形从首个专业版本进入工程模型，时间码只是时钟来源之一。
- 所有危险恢复动作先显示范围；“清编程器”“释放播放”“恢复 Master”“停止输出”“设备维护”保持独立。
- Web／移动端能做受能力约束的真实编辑；遇到不支持的高级内容保留原数据，禁止静默降级覆盖。
- 云端分发不直接改现场：先完整下载和校验，再由明确动作激活，可随时看到目标、已下载和运行版本。

## 落地顺序

本节替代旧版 A0／A0b／A1／A2／B／C 的串行排期；历史文档的阶段称呼仍用于解释当时的范围，实际任务与出口以[当前执行计划](development/execution-plan.md)为唯一依据。

| 层次 | 当前基础与下一项结果 |
| --- | --- |
| 已有基础 | A0 静态求值和编码、CORE-001／002 修复及 Tauri 最小灯光编辑／保存重开已完成；其他领域问题见[实现状态](implementation-status.md)，尚无持续播放和真实发送 |
| 当前可见编辑 | 沿用现代创作式组件工作台，将已收敛的选择／属性／撤销接到真实工程；空间、灯具与时间线各在相关契约明确后逐步接入，不等待所有专业功能设计完毕 |
| 首次软件＋播放盒 | 主机编译、Rust 受限执行器、现有微雪 ESP32-S3-RS485-CAN 的 1 路输出、USB 下发、无电脑选场景执行，以及故障／掉电与整机验收；盒子属于首次交付，不放到多端／云端之后 |
| 专业单机扩展 | 引用预设、跟踪编辑、效果／分布、参考波形、排练、节目单、现场覆盖、换灯与诊断；按能力总表逐步扩展，Art-Net／sACN、复杂三维和完整媒体控制不是首条 DMX 链路的前置条件 |
| 多系统、多端与云端 | 外部媒体控制／监看、互动机构、专业预演、实体控台、ARM、iPad／Web 和云端资源分发分别验收；共享工程语义与命令，当前仍在 MacBook 开发，未来 iPad 保留灯光编排主力定位 |

用户已明确的[中转与临时授权](development/decisions/PRODUCT-ADR-004-relayed-device-authorization.md)是相关商业交付的前置验收项；授权服务不能随协作云盘一起笼统后置。开发期工程／参考执行不等待云服务。工程包、设备播放包和已激活版本继续分开。

## 当前不提前锁死的选择

专业预演的分阶段验证见 [Depence R4 对照](depence-r4-assessment.md)、[UE5 预演](ue5-professional-previsualization.md)及[场地采集](venue-capture-design.md)。可先使用静态／录制输入开展原型；专项仿真与真实控台闭环分开验收。

首台样机已经选定上述微雪板卡；工程采用[严格 JSON 正文与三层文件草案](project-format/README.md)，最小灯光子集已能读写。板级驱动、控制面协议／硬件档案、监听接收后端、后续加密封装、数据库迁移工具、对象存储厂商和专业 3D 后端仍需实施或设备证据。先保持接口边界，不把待验证选择写进节目语义。

UI 沿用用户认可的现代创作式方向与按需组件；后续快捷键／工作区模板可照顾专业习惯，仍只有一套底层行为，不以读取两家私有工程文件作为路线。
