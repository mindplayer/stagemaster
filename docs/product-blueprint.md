# StageMaster 产品蓝图：取长补短后的统一系统

状态：产品方向 v0.3，2026-09-11；已加入专业预演、场地采集、外部音视频控制／监看及实体控台方向。它把 [grandMA3／Titan 功能研究](console-research/README.md)与 [Depence R4 重点对照](depence-r4-assessment.md)转成 StageMaster 自己的设计方向。详细边界以[架构设计 v0.5](architecture.md)及专项补充为准；阶段建议随实操反馈调整，实际完成范围见[实现状态](implementation-status.md)。

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

- 默认显示高影响状态：Blind／Preview、总控偏离、输出禁用、外部输入、Park、未匹配属性、缺媒体和版本不兼容。
- 把 Tracking View 与输出来源合成一条检查链：既看 Cue 从哪里继承，也看最终被哪一路覆盖。
- 将 MA3 Recipe 的可重算能力和 Titan 的快速 Shape／Fan 工作流放在同一个效果模型上。
- 让音频波形从首个专业版本进入工程模型，时间码只是时钟来源之一。
- 所有危险恢复动作先显示范围；“清编程器”“释放播放”“恢复 Master”“停止输出”“设备维护”保持独立。
- Web／移动端能做受能力约束的真实编辑；遇到不支持的高级内容保留原数据，禁止静默降级覆盖。
- 云端分发不直接改现场：先完整下载和校验，再由明确动作激活，可随时看到目标、已下载和运行版本。

## 落地顺序

### A0：静态语义验证原型

简单配适、组顺序、编程器、Selective Preset 引用、Cue 静态跟踪、基础合成和 DMX 数据编码已形成可运行链路。当前存在[审查所列问题](architecture-review.md)，来源只到贡献标识，没有持续时钟或真实发送。

### A0b：架构与契约收敛

补齐状态归属、身份与值、灯具多单元、编辑命令、编译计划、现场激活、时间与输出契约；修正明确缺陷，用代表性节目和失败场景验证边界。该阶段完成后再固定永久工程格式和扩大 UI 实现。

### A1：可编辑单机闭环

加入工程持久化、撤销／恢复、Cue List 播放状态、Fade／Delay、基础 Phaser、参考音频波形／标记、外部播放控制和 Tauri 桌面壳。灯光师能从零编一小段音乐节目并保存重开；真实声音来自外部播放器，监听单独接入。

### A2：真实输出与现场可靠性

实现一个缓冲式 USB／RS485 DMX 适配器及 Art-Net／sACN，测量抖动、断连、睡眠恢复、UI 卡顿、保存和日志压力。把来源诊断接到实际帧；明确设备失联策略。

### B：专业单机可用

补齐多 Cue 跟踪编辑、效果曲线与空间分布、播放页面／执行器／Masters、盲编和独立预览、Set List、换灯／扩灯影响预览、备份恢复和演出视图。

### C：多端、盒子与云

用相同命令模型实现 Web Remote、平板简单编排和 ARM 执行目标；随后接入 Fastify＋PostgreSQL＋对象存储的工程版本、素材和发布包分发。

[廉价独立 Cue 播放盒](standalone-cue-player.md)是明确使用方式：提前导入编好的节目，没有电脑时在盒子选 Cue 播放。先验证 ESP32 受限档位，复杂节目使用 ARM；本地文件导入与自主播放可先于云端完成，云端以后复用同一交付流程。

## 当前不提前锁死的选择

专业预演的分阶段验证见 [Depence R4 对照](depence-r4-assessment.md)、[UE5 预演](ue5-professional-previsualization.md)及[场地采集](venue-capture-design.md)。可先使用静态／录制输入开展原型；专项仿真与真实控台闭环分开验收。

第一款 RS485／USB 设备、控制面协议／硬件档案、监听接收后端、工程容器格式、数据库迁移工具、对象存储厂商和完整 3D 引擎仍需原型或设备证据。先保持接口边界，不把待验证选择写进节目语义。

UI 可提供“MA 式”“Titan 式”快捷键或工作区模板，但 StageMaster 只有一套底层行为。产品兼容的是专业工作习惯，不以读取两家私有工程文件作为路线。
