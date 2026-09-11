# 跨系统术语与 StageMaster 吸收建议

两套控台共同解决配适、编程、素材复用、节目播放和现场输出，但对象划分与默认行为并不一致。值得吸收的是灯光师完成工作的方式，以及每个操作可预期的结果。不能简单取两家按钮名称的并集，再把它当作一份软件设计。

本模块前半部分归纳已在 M01–M21 核查的术语，链接返回证据所在模块；后半部分是 **StageMaster 的设计建议**，不代表两家产品的内部架构，也不代表用户已批准全部功能进入首版。

## 按行为理解术语

“相近”表示可用于比较同一类工作，不表示所有属性、默认值和失败行为兼容。

| 概念 | grandMA3 | Avolites Titan | 比较时要保留的差异／出处 |
| --- | --- | --- | --- |
| 灯具定义 | Fixture Type | Personality | 包含模式与属性解释；版本、通道功能、几何信息各自检查。[M04](M04-fixtures-patch.md) |
| 配适 | Patch | Patch | 用户界面用“配适”；逻辑身份、地址、档案版本分开。[M04](M04-fixtures-patch.md) |
| 用户可见编号 | FID／CID 等 | User Number | 不用它替代永久对象身份；Titan 还另有 TitanId。[M04](M04-fixtures-patch.md)、[M17](M17-remotes-api.md) |
| 子灯／像素单元 | Subfixture／Geometry | Super Fixture／Sub-fixture | 母灯与 cell 层级、属性继承及选择方式不同。[M04](M04-fixtures-patch.md) |
| 输出资源 | Parameter／Universe | DMX Line／Universe | 参数不等于通道；线路编号不等于授权容量。[M01](M01-products-capacity.md) |
| 同值多地址 | Multipatch | 同 Handle 的多个 Dimmer | 不等于多份可独立编程的完整灯具。[M04](M04-fixtures-patch.md) |
| 暂停配适 | 需按具体配适功能理解 | Park Fixture | Titan Park 移出物理映射但保留编程，不等于 MA Park 输出值。[M04](M04-fixtures-patch.md)、[M16](M16-dmx-network.md) |
| 组 | Group | Group | 保存选择、顺序等；改组未必改变已经录好的 Cue。[M05](M05-selection-groups-layout.md) |
| 编程空间 | Selection Grid | Group Layout | 前者有三维网格，后者二维；都不能与真实舞台坐标混为一谈。[M05](M05-selection-groups-layout.md) |
| 操作画布 | Layout | Workspace／Group Layout 等 | 画布里的按钮、选灯位置、效果坐标应分别判断。[M02](M02-workspace-users.md)、[M05](M05-selection-groups-layout.md) |
| 当前编辑区 | Programmer | Programmer | 选中、已改、激活、输出和将被记录不是同一种状态。[M06](M06-programmer-attributes.md) |
| 清除 | Clear／Off | Clear／Release／Off | Clear 后 LTP 是否恢复，必须看各系统语义与配置。[M06](M06-programmer-attributes.md) |
| 按顺序分布数值 | Align | Fan | 可比较扇形、渐变、分段、曲线和组内分布。[M06](M06-programmer-attributes.md) |
| 复制灯具属性 | At／Clone Programmer | Align | Titan Align 不能直译为 MA Align。[M06](M06-programmer-attributes.md) |
| 灯具便于观察的初始状态 | 按 Default／Highlight 等具体功能比较 | Locate | 不对应 MA3 2.5 的 Locate 对象查找。[M06](M06-programmer-attributes.md)、[M21](M21-version-baseline.md) |
| 素材 | Preset | Palette | 要比较引用和更新传播，不能只比较池按钮。[M07](M07-presets-palettes.md) |
| 按单灯保存素材 | Selective | Normal | 概念相近，但记录／调用优先级仍需核对。[M07](M07-presets-palettes.md) |
| 按灯型复用素材 | Global | Shared | MA Global 不是 Titan Global。[M07](M07-presets-palettes.md) |
| 跨灯型素材 | Universal | Global | Titan Global 的适用属性有范围，不代表所有属性通用。[M07](M07-presets-palettes.md) |
| 引用展开与重连 | Magic／Recast 等不同功能 | Quick Merge／Update 等不同功能 | 复制硬值、维持引用、重建关联是不同动作。[M07](M07-presets-palettes.md) |
| 静态或动态场景 | Cue／Cue Part | Cue | 一个 Cue 的内容、时间和正在运行的状态分开。[M08](M08-cues-tracking.md) |
| 顺序节目 | Sequence | Cue List | 跟踪、触发、时间、释放、暗场预定位分别比较。[M08](M08-cues-tracking.md) |
| 步进追逐 | Sequence 触发／时间组织等 | Chase | Titan Chase 是独立工作流；不强行一对一映射。[M08](M08-cues-tracking.md) |
| Cue 内部分区 | Part | 单属性时间等相关功能 | 不把 Autoload 或单灯时间当作同义 Part。[M08](M08-cues-tracking.md) |
| 继承前值 | Tracking | Tracking | 还涉及硬值、Block、Cue Only、效果继承与删除行为。[M08](M08-cues-tracking.md) |
| 只改这一场 | Cue Only | Cue Only | 需要检查后续恢复点和素材引用更新的实际影响。[M08](M08-cues-tracking.md) |
| 暗场预定位 | MIB | Move In Dark | 时机、Fade、排除属性、目标 Cue 不应省略。[M08](M08-cues-tracking.md) |
| 输出混合 | HTP／LTP／优先级 | HTP／LTP／优先级 | 同名算法仍受来源优先级、释放和效果层影响。[M09](M09-playback-mixing.md) |
| 操作映射 | Executor／Page | Handle／Playback Page | 编程内容不应由推杆位置决定身份。[M09](M09-playback-mixing.md) |
| 抬起／放下推杆 | Executor 功能及序列设置 | Fader Mode／Release Mask | 归零不保证所有属性恢复；需定义激活与释放。[M09](M09-playback-mixing.md) |
| 停止播放 | Off 等 | Kill | 停止运行与释放 LTP 要分别判断；Kill 不等于删除节目。[M09](M09-playback-mixing.md)、[M17](M17-remotes-api.md) |
| 动态属性 | Phaser | Shape／Key Frame Shape | 都可产生动态，但步骤、基值、相位与终止不同。[M10](M10-effects-phasers.md) |
| 形状素材 | Shape | 预制 Shape | MA Shape 可用于 Phaser Recipe；不是 Titan 同名对象的复制品。[M10](M10-effects-phasers.md) |
| 停止／屏蔽效果 | Stomp | Mask FX | 一个偏向收敛到静态步骤，一个按属性／灯具屏蔽效果；需查作用域。[M10](M10-effects-phasers.md) |
| 按规则生成编程 | Recipe／Cook | Group／Palette／效果复用 | 未确认 Titan 有同构 Recipe；Cook 还要规定覆盖手工值的方式。[M11](M11-recipes-reuse.md) |
| 编程复用到更多灯 | Clone | Copy Fixture／Align／Exchange | 创建新灯、复制值、换灯型是三个不同动作。[M11](M11-recipes-reuse.md) |
| 图像驱动灯阵 | Bitmap | Pixel Mapper | 坐标源、图层、素材、动画及输出属性分别比较。[M12](M12-pixel-media.md) |
| 视频系统协作 | Bitmap／媒体输入及网络接口 | Synergy／Ai／Prism | 控制、预览和媒体传输与 DMX 像素输出不是同一能力。[M12](M12-pixel-media.md) |
| 节目时间线 | Timecode Show | Timeline | 记录操作事件不等于记录最终 DMX 帧。[M13](M13-timecode-audio.md) |
| 时间来源 | Timecode Slot／Internal | 内部／外部时间码源 | 时钟来源、显示帧率、丢失策略与事件内容分开。[M13](M13-timecode-audio.md) |
| 节拍主控 | Speed／BPM Master | BPM／Rate／Speed Master | 频率、倍率、拍点和相位不是同一个量。[M09](M09-playback-mixing.md)、[M13](M13-timecode-audio.md) |
| 编辑隔离 | Blind／Preview | Blind／Blind Playback | 改节目数据可能仍影响运行；必须明确隔离什么。[M14](M14-preview-3d.md) |
| 预备现场变化 | Preview 相关流程 | Scene Master | 可比较工作目的，但不可认定对象、提交方式和过渡相同。[M14](M14-preview-3d.md) |
| 三维控制 | XYZ／MArker | Capture 可视化相关操作 | 三维摆灯不等于世界坐标跟随编程。[M14](M14-preview-3d.md) |
| 操作自动化 | Macro／Lua Plugin | Key Macro／脚本 API | 不同录制范围、上下文和版本约束。[M15](M15-macros-automation.md) |
| 浏览器控制 | Web Remote | WebAPI 客户端；移动 Remote 另列 | 网页遥控界面、HTTP API、手机 App 不能混作一个功能。[M17](M17-remotes-api.md) |
| 用户范围 | World／User／Profile | User／Handle World | MA World 涉及灯具／属性范围，Titan Handle World 重点是句柄布局。[M18](M18-sessions-backup.md) |
| 局域网协作 | Session／Ownership | Multi-user Session | 编辑所有权、Show 同步和最终输出责任分别描述。[M18](M18-sessions-backup.md) |
| 备份与接管 | Session 选主及数据同步 | Backup／Take Over | 备份时间点、手动／自动过程和输出恢复分别核查。[M18](M18-sessions-backup.md) |
| 演出曲目管理 | Sequence／Page／Macro 等组合 | Set List／Track | 曲目可以绑定页面、工作区和宏；不是单纯 Cue 列表。[M19](M19-live-show.md) |
| 在线服务 | WorldServer 的具体服务 | 官方软件／档案服务等 | 不据此认定存在完整云端工程存储与分发体系。[M20](M20-maintenance-diagnostics.md) |

## 应优先吸收哪些工作方法

以下是研究后的设计判断。

| 工作目标 | 从 MA3 重点研究 | 从 Titan 重点研究 | StageMaster 可以形成的能力 |
| --- | --- | --- | --- |
| 工程可扩展且易修改 | 对象引用、Recipe、Selection Grid、跟踪和内容来源 | Palette、Fixture Exchange、Shared 素材 | 稳定对象身份、可检查的依赖、换灯与扩灯后的变化预览 |
| 灯光师快速做现场 | 执行器分配、Masters、强度与优先级控制 | Palette 快调、Shape、Set List、Scene Master | 明确的推杆／按钮语义，快速素材调用，预备后过渡 |
| 同一效果适应不同灯阵 | MAtricks、Phaser、Cook | Fan、Group Layout、Key Frame | 一份效果模型，多种编辑入口；灯序／空间变换单独保存 |
| 排练修改不破坏整场 | 跟踪表、Cue Only、Block、Preview 边界 | Tracking View、Update、Unfold、Include | 修改前显示影响范围，可回看数据来源与恢复点 |
| 音乐卡点与自动播放 | Timecode 事件、Sound、时钟槽 | Timeline 的图形编辑和现场录制 | 音频波形、标记、事件与推杆包络，内部排练及外部同步 |
| 现场可解释 | 来源表格、Masters、输出监视 | Active Playbacks、通道视图、Diagnostics | 从灯具属性追溯节目、编程器、效果、主控、配适及输出状态 |
| 后续多端和盒子 | 独立控制对象与输出站点的工作方法 | Remote 的有限编程和 API 入口 | 共享工程契约与编辑命令，界面和设备输出分开适配 |

不要为了“两家都有”而造两套跟踪引擎、两套效果引擎。可以提供符合不同操作习惯的界面入口，但底层必须采用一套明确的 StageMaster 语义。兼容某种习惯也不意味着读取对方私有工程格式。

## 模块所有权与依赖方向

技术框架沿用已确认的 Rust＋TypeScript；下面是职责建议，不要求每行都拆成独立进程或服务。

| 模块 | 权威数据／职责 | 输入与输出契约 | 应避免的耦合 |
| --- | --- | --- | --- |
| 灯具档案 | 属性、模式、物理范围、cell、DMX 编码定义 | 版本化档案及能力声明 | 核心直接读取厂商下载目录 |
| 配适 | 灯具实例、地址、校准、输出路由绑定 | 受校验的配适命令与只读映射 | UI 直接修改通信设备内存 |
| 工程领域 | Group、素材、Cue、效果、时间线及引用 | 稳定 ID、版本、依赖图 | 用按钮位置或显示编号作为永久 ID |
| 编辑应用层 | 命令校验、事务、撤销、变更影响 | 类型化编辑命令、结果、修订号 | 桌面、Web、移动端各维护一套业务规则 |
| 节目转换 | 将编辑工程解析为可执行内容 | 工程版本＋目标能力 → 执行计划／诊断 | 运行中临时解析任意云文件 |
| 运行与时钟 | 播放位置、Cue 切换、跟踪、效果相位 | 执行计划、时钟、控制命令 → 属性状态 | 音频 UI 或浏览器刷新驱动节目时钟 |
| 输出合成 | 来源优先级、HTP／LTP、主控、覆盖与释放 | 多路属性贡献 → 最终属性值及来源 | 推杆控件自己计算最终 DMX |
| 传输适配 | DMX 编码、网络发送、RS485 设备访问 | 帧／时间及设备状态 | 一个灯具 SDK 渗入所有领域模块 |
| 预览与可视化 | 独立预览实例、二维／三维呈现 | 场景、输出快照、诊断 | 渲染负载阻塞现场输出 |
| 多端界面 | React 组件、交互布局、可访问性 | 共用 TS 契约及宿主接口 | 浏览器页面直接持有物理 DMX 端口 |
| 本地存储 | 工程持久化、快照、恢复和媒体缓存 | 保存／加载／迁移／校验结果 | 保存文件时隐式停止播放 |
| 远程控制 | 身份、权限、会话、去重和状态订阅 | 控制 API 与编辑 API 分开 | 重连重放旧命令而再次触发现场动作 |
| 云端业务 | Fastify 业务 API、PostgreSQL 元数据、对象存储 | 项目修订、素材与不可变发布包 | 把云端数据库事务当作设备激活成功 |
| 设备分发 | 下载、完整性校验、准备、激活、回滚状态 | 目标版本／已下载版本／运行版本 | 收到同步消息便立即替换正在播放的节目 |

Rust 维护权威工程和执行语义；TypeScript 维护界面、云端业务与可共享协议类型。类型／协议生成工具应尽量避免手写两份易漂移的数据结构。脚本扩展如采用 TypeScript，也应经受控命令入口执行，不进入要求稳定时序的输出循环。

## 专业单机优先级建议

以下是依赖顺序，**不是已经承诺的首版排期**。优先级不应把用户重视的音频、波形和卡点长期挤到最后。

| 阶段 | 应形成的完整工作闭环 | 进入下一阶段前需要说明的结果 |
| --- | --- | --- |
| A：语义与输出基础 | 配适 → 选组 → 改属性 → 记录素材／Cue → 播放 → 保存／恢复；内部音频与时间线结构同时确定 | 跟踪、优先级、释放、时间、撤销的行为有规范；独立引擎与模拟输出可验证 |
| B：专业单机可用 | Cue List、效果、推杆／按钮、多页、音频波形卡点、基本时间线、离线预览、输出来源诊断 | 从零编一段完整节目，并能排练修改、现场临时覆盖和恢复 |
| C：提高编程效率 | 可重算配方、扩灯／换灯、批量更新、复杂效果、Set List、宏、像素与外部同步按需求加入 | 模板复用后的差异可检查；工程迁移和现场修改具有可预期结果 |
| D：多端与现场扩展 | Web 遥控、移动简单编排、控制面板、ARM 运行目标与局域网协作 | 权限、断线、重连、能力差异与多入口命令冲突均有明确定义 |
| E：云端存储与分发 | 离线工程同步、素材库、版本发布、设备分发及激活 | 编辑版本、发布版本、下载状态与现场运行状态一致可查；断网仍可执行已准备节目 |

云端数据边界应从 A 阶段保留，但上线云服务不成为 A／B 的前置条件。ARM 与桌面可复用 Rust 模块；ESP32 是否使用较小的执行格式，需要按真实容量和硬件测试决定。

## 后续需求和验收场景

以下场景是建议，不是已完成测试。每个场景都应先写出本系统的预期行为，再做自动化验证；涉及物理输出的部分另做设备验证。

| 场景编号 | 场景 | 要观察的结果 |
| --- | --- | --- |
| S01 | 普通灯、16-bit 摇头、多 cell 灯具混合配适，制造地址冲突 | 地址／参数／cell 身份一致；冲突被发现，粗细通道正确 |
| S02 | 选灯、Locate 类操作、修改一个属性、Clear、记录 | 哪些属性进编程器、参与输出和被记录可解释 |
| S03 | 创建位置／颜色素材，录入多个 Cue，再更新素材 | 引用随预期传播，硬值保持独立，缺属性明确提示 |
| S04 | 跟踪序列中插入、删除、Cue Only 修改并更改 Block | 后续 Cue 的结果符合定义，变化范围可预览 |
| S05 | 两路播放和编程器共同控制同一属性，再依次释放 | HTP／LTP、优先级、恢复来源及 Fade 有确定结果 |
| S06 | 推杆换页后原节目仍在运行，返回原页再推 | 对象身份稳定，是否接管／匹配电平符合设置 |
| S07 | 制作位置相对效果与颜色绝对效果，改变基值和速度 | 幅度、基值、相位、同步、一次性结束行为一致 |
| S08 | 灯阵从 12 台扩到 20 台，再交换为另一灯型 | 选择顺序、效果分布和素材映射结果可检查，不静默丢属性 |
| S09 | 现场运行时盲编；修改被当前节目引用的素材 | 编辑隔离、提交与送入现场的边界可见 |
| S10 | 音频卡点播放，从中间开始、后退、循环和改偏移 | 声音、Cue、推杆包络及效果相位按规定恢复 |
| S11 | 外部时间码断开、跳时、恢复；UI 长时间卡住 | 引擎按时钟策略运行，状态与输出责任明确 |
| S12 | 文件保存中断、载入缺媒体／旧档案工程 | 恢复点、缺失项及迁移结果可见，原版本可追溯 |
| S13 | Web 断线重连并重试同一控制请求 | 不重复触发过期操作，权限与状态版本一致 |
| S14 | 云端上传不完整、下载失败、现场正在播放时收到新版 | 不把残缺包激活；运行版本与待激活版本分开 |
| S15 | 同一节目在桌面预览和 ARM 目标运行 | 支持能力内的求值一致，超出能力有明确拒绝或转换报告 |
| S16 | 误将维护指令作为日常 Cue，或输出设备失联 | 根据设备类别和明确策略处理，不能统一假定所有通道归零合适 |

## 如何把研究项转成开发任务

从[统一功能索引](feature-matrix.md)选定条目，先写清“灯光师在什么情况下要完成什么动作”，再决定 StageMaster 的行为。任务至少记录：关联研究 ID、对象与状态、默认值、影响范围、依赖、异常结果、验收场景、是否需要真实设备。此后再讨论实现顺序和 UI。

研究库记录的是竞争产品的公开行为；它不替代 StageMaster 自己的规格。最终应形成一套中文、明确且一致的专业语义，而不是让用户在两套不同的默认行为之间猜测。
