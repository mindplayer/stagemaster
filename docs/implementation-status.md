# 实现状态

[EXEC-006](development/tasks/EXEC-006-execution-board-navigation.md) 已补后台执行台导航：类型／状态／常用和未应用输入筛选，筛选外运行提示、固定排序与输入／步骤保留；隐藏旧确认取消，新后台身份清空易失输入。353 UI、类型／格式／最终桌面与原生重开验证通过；保持原后台生命周期和软件输出边界。

[EXEC-005](development/tasks/EXEC-005-source-execution-progress.md) 已将原 Rust 播放器的真实阶段和步骤时间接入后台节目卡片：当前／下一步、延时／渐变／自动等待／人工保持、暂停冻结及循环回首步。557 项去重相关 Rust、349 UI、严格检查、最终桌面打包与原生验收通过；只读观察、错误提示和编辑草稿隔离已验证。当前仍为软件输出，专业执行器布局、实体总控及实际 DMX 交付未因此完成。

[TIME-001](development/tasks/TIME-001-independent-clock-boundaries.md) 正在接独立时间域：[媒体跟随组](module-api/media-source-groups.md) 已有真实编译／Player／统一合成／软件端口实现，准备可在独立工作线程完成。时钟映射、暂停／定位与自主执行隔离、失联及旧代次拒绝通过 17 项新增专项及 877 项全工作区测试、2 个文档示例和严格检查。接续已补正式原生音源的完整帧消费计数、音源实例与原采样时间，读取竞争不刷新缓存；相关验证见工单。[原宿主媒体入口](module-api/media-host-ingress.md)已接固定容量准备／观测槽、原控制权／回执与调度外追赶／回收；原解码源到后台线程的实际通道值通过验证，当前全工作区 895 项测试与 2 个文档示例通过。独立进程／桌面完整音频所有权、暂停健康状态、重启重绑定、循环回跳与真实同步仍未完成。

[PREVIS-003](development/tasks/PREVIS-003-background-observation.md) 已接[后台只读三维观察](module-api/background-previsualization.md)：固定工程／完整软件帧、独立只读客户端、8／16 位还原及内嵌 UE 来源切换。真实进程与原生渲染重启证明执行身份、控制权及输出不变，编辑总控不再叠加后台灯光。392 项相关 Rust、307 UI、7 UE、类型、严格检查及标准桌面打包通过。此项更新下文历史“专业渲染观察尚未接入”的限定缺口；完整专业光学、外部控台输入、客户 UE 打包、多时钟、真实输出及统一云端交付仍未完成。

[HOST-005](development/tasks/HOST-005-desktop-background-execution.md) 已接[桌面后台执行](module-api/desktop-background-execution.md)：独立客户端、已准备目录、明确控制权／回执、来源独立操作和随包进程。原生退出后同一节目继续、重开只读接管及正常关闭通过；112 项相关 Rust、307 UI、严格检查和标准桌面打包通过。编辑预演仍保留既有音频／草稿语义；当前后台只有软件输出，多时钟、专业渲染观察、真实输出及云端交付仍待完成。下列记录为各增量当时的证据，旧记录中的“桌面尚未迁移”由本项限定范围更新，不能据此视为全部迁移。

[HOST-004](development/tasks/HOST-004-multi-source-process.md) 已提供同一程序的 [v2 多来源入口](module-api/multi-source-process.md)：准备目录、受保护场景／列表操作、手动功能值批量、电平和回执，复用原会话与调度；独立客户端被杀、准备文件移除后同一组继续。848 项全工作区测试通过；随后原始截止时刻转发补强后的 33 项专项及 2 个文档示例通过，最终严格检查通过。桌面尚未迁移，当前仍为单域软件采样，无物理输出、跨时钟或商业安全完成声明。

[HOST-003](development/tasks/HOST-003-shared-live-host.md) 已把固定多来源组接入[现有后台宿主](module-api/shared-live-host.md)：共享输入控制权、唯一调度实现、有界属性批量和固定来源快照；旧默认设备调用与受限命令入口保持。6 项新增、842 项全工作区测试及 2 个文档示例、严格 Clippy／fmt 与 Xtensa 编译通过。当前是实际线程的软件适配，HOST-002 的 HTTP／桌面入口和物理端口尚未迁移，未宣称多时钟或硬实时性能验收。

[MIX-003](development/tasks/MIX-003-prepared-source-compositor.md) 已提供[固定来源组](module-api/prepared-source-compositor.md)：真实场景／多个列表／手动层统一推进、按激活时刻接管、累计资源预算和完整帧故障保护。11 项新增、836 项全工作区、最终 9 项来源组专项、严格 Clippy／fmt 与 Xtensa 编译验证通过；不改变工程／设备包格式。当前仍为标准宿主软件后端，尚未接独立进程、桌面和物理端口，不代表 ESP32 多来源容量或硬实时性能验证。

[MIX-002](development/tasks/MIX-002-sequence-contributions.md) 已在同一个 Player 上提供[列表贡献](module-api/live-sequence-contributions.md)：逐步继承／释放、延时、明确接管与普通推进、完整循环折叠和可见值渐变起点。修正从手动层接回位置时跳变，补亮度电平反算。12 项新增、825 项全工作区与最终 13 项工程专项通过；严格检查及 Xtensa 无标准库编译通过。独立宿主来源管理／多时钟调度、桌面和物理输出接线仍待完成。

[MIX-001](development/tasks/MIX-001-live-contributions.md) 已提供有界[现场属性贡献](module-api/live-contributions.md)：两个真实动态保持场景与手动控制逐属性合成、明确接管／归还，亮度推子与释放分离，连续采样不抢占；工程快照绑定合成与编码。15 项新增测试（含 448 组参考对照）、813 项全工作区测试通过。当前为软件模块与软件端口组合验证，列表适配见 MIX-002；桌面／独立宿主接线及物理输出仍待实现。

[OUTPUT-001](development/tasks/OUTPUT-001-port-authority.md) 已提供无堆、no_std 的[单端口输出权](module-api/output-port.md)：独占来源、接管前静默、最新完整帧、驱动接纳／完成区别及故障撤权。真实工程编译／安装／Runtime 到软件驱动、控制断连继续和外部接管已验证；19 项专项与 798 项全工作区通过。桌面／独立进程接线与真实输出仍未完成；多来源基础合成见 MIX-001，现有固件 RS485 保持禁用。

[TRANSPORT-001](development/tasks/TRANSPORT-001-record-carriers.md) 已将 BLE 的应用握手／保活／加密消息抽到[独立会话模块](module-api/application-record-carriers.md)，原生适配复用；真实本机 TCP 与 GATT 大小的软件分片通过同一安装器，落盘内容一致。14 项新测试、45 项原设备宿主回归及 779 项全工作区通过。当前仅新增软件字节流证据，通用发现／真实有线驱动／生产安全仍未完成。

[HOST-002](development/tasks/HOST-002-local-execution-process.md) 已贯通实际工程准备与[本机独立执行进程](module-api/local-execution-process.md)：受保护只读／控制入口、服务端会话、有界命令／回执及进程生命周期。真实控制客户端被杀后同一节目继续，新控制者可接管、旧指令拒绝；12 项专项及 765 项全工作区测试通过。当前仅 Mac 软件执行验证，桌面迁移、远程认证、音视频协调及真实输出仍未完成。

[HOST-001](development/tasks/HOST-001-independent-runtime-host.md) 提供上述程序复用的独立 Rust 调度宿主库，不依赖 Tauri／BLE／媒体：有界客户端入口、控制权回执、断连继续、只读观察与独立关闭／故障状态。其 15 项专项和可编译调用示例继续保留。

[STAGE-005](development/tasks/STAGE-005-shared-3d-fixture-movement.md) 已接二维／三维共享选灯、整组水平与升降、混合锁定／取消保护和一次历史；684 Rust／297 UI／7 UE、类型／严格检查／构建及原生高差双灯、单灯、取消、锁定与保存重开通过。完整组旋转／缩放、混合构件与三维框选仍后续。

[AUDIO-018](development/tasks/AUDIO-018-preserved-entry-fades.md) 已补渐变内分割与完整内部截取，保留原起始值、渐变权重与效果进度；678 Rust／291 UI／160 格式及原生越界焦点、取消／历史／保存重开通过。设备包不支持的渐变偏移会显式拒绝，多轨与双动态持续交叉仍后续。

[LIBRARY-003](development/tasks/LIBRARY-003-keyframe-templates.md) 已在本地模板中增加 2–32 帧亮度曲线，保留原有过渡、周期与灯序；明确文件格式 2、额外来源能力和旧格式兼容，导入提供只读曲线／精确值。668 Rust／287 UI／146 格式、严格检查／桌面及真实渐入效果跨场景导入、取消／历史／保存重开通过。云目录、其他属性配方仍后续。

2026-10-02：[LIBRARY-002](development/tasks/LIBRARY-002-local-intensity-templates.md) 已实现独立基础亮度模板、跨工程导入／审阅／导出、来源快照、版本保护和一次历史；664 Rust／287 UI、类型／严格检查／桌面，以及两个独立工程 80／3 台复用、取消／冲突／保存重开通过。云端灯效目录、其他配方与真实新增灯型仍未实现。[FIXTURE-007](development/tasks/FIXTURE-007-custom-wheel-appearance.md)／[FIXTURE-008](development/tasks/FIXTURE-008-color-slot-remap.md) 已补自定义色盘外观、独立变体与通道修订审阅；[POSITION-003](development/tasks/POSITION-003-reference-checks.md) 已补逐灯参考点与模型偏差检查，不能等同实灯自动校准。

2026-10-02：[EFFECT-007](development/tasks/EFFECT-007-world-line-effects.md) 已实现共同世界目标直线往返，整线可达性与固定 32 帧误差认证、草稿即时预演和双灯 UE 通过；626 Rust／274 UI／121 格式、类型／严格检查／桌面与原生历史／取消／保存重开通过。轨迹平面选点、任意多段运动和实灯约束仍待补。

[AUDIO-017](development/tasks/AUDIO-017-marker-selection.md) 已补目录／波形卡点共享组选择、范围与整组视图、目标输入固定及原生一次历史／保存；[APP-001](development/tasks/APP-001-exit-cleanup.md) 修复退出异步清理与 WebKit 回执互等，原生正常关闭／取消保护通过。

[AUDIO-016](development/tasks/AUDIO-016-selection-view.md) 已补所选片段／组／卡点与旧段落视图适应、关闭跟随与草稿保护，不改变播放位置或工程。

[UX-046](development/tasks/UX-046-scene-removal-preflight.md) 已补单／批量场景删除前审阅、引用阻断和使用位置编辑入口；任一被引用即全组保护，成功一次历史，原生保存重开与内容核对通过。

[UX-045](development/tasks/UX-045-scene-usage-navigation.md) 已补场景使用位置、引用搜索与精确编辑导航；可跨步骤执行视图、音乐片段／旧卡点批量模式和筛选找回目标，播放位置保持，错误草稿拒绝。

[UX-044](development/tasks/UX-044-scene-batch-copy.md) 已补场景成组复制、筛选／范围选择及一次历史，复制独立场景和效果身份、保留原引用；共享草稿失败保持选择，保存重开通过。

[UX-043](development/tasks/UX-043-stage-directory-navigation.md) 已补场地目录定位、筛选外计数和内部键盘浏览；保留原选择／显隐／属性草稿，取消无效属性后清理过期校验，原生验收通过。

[AUDIO-015](development/tasks/AUDIO-015-group-fade.md) 已补统一多片段进入渐变，混合值／筛选外选择／锁定与长度原子检查、草稿保护及一次历史，原生保存重开通过。

当前界面能力基线：2026-10-01，[UX-041](development/tasks/UX-041-spatial-fixture-order.md) 已补灯组／效果共享空间排序和草稿隔离，[UX-040](development/tasks/UX-040-common-target-plane.md) 已补共同目标平面选点／精确高度／手势取消及原子求解，[REPORT-002](development/tasks/REPORT-002-sequence-report-export.md) 已补剧本场景列表节目单／快照来源与取消保护，[UX-039](development/tasks/UX-039-group-member-workflow.md) 已补灯组批量成员、共享排序、搜索分页及回车保护，[UX-038](development/tasks/UX-038-copy-values-preflight.md) 已补复制来源预览、目标缺项定位／筛选及取消保护，[UX-037](development/tasks/UX-037-shared-preset-scopes.md) 已统一快捷属性范围、包含色盘并隔离亮度／频闪，[UX-036](development/tasks/UX-036-preset-scope-continuity.md) 已补预设记录／更新范围连续性及镜头中文明细，[UX-035](development/tasks/UX-035-attribute-categories.md) 已补亮度／颜色／图案／镜头分类、草稿保护与窄栏导航，[FIXTURE-005](development/tasks/FIXTURE-005-continuous-optics.md) 已补变焦／调焦／光圈的连续建档、场景预设、渐变及明确预演边界，[UX-034](development/tasks/UX-034-unified-plan-labels.md) 已补场地各类名称统一避让、选中优先与显示模式，[FIXTURE-004](development/tasks/FIXTURE-004-portable-modes.md) 已补模式文件跨工程复用／显式检查与独立身份，[REPORT-001](development/tasks/REPORT-001-patch-report-export.md) 已补快照 CSV 配灯表与冲突保护，[AUDIO-014](development/tasks/AUDIO-014-marker-edge-scroll.md) 已补卡点／旧段落边缘滚动与取消，[UX-033](development/tasks/UX-033-fixture-label-collision.md) 已补密集灯位标签避让与选中优先，[AUDIO-013](development/tasks/AUDIO-013-phase-preserving-trim.md) 已补裁切效果进度／源范围与精确输入，[EFFECT-006](development/tasks/EFFECT-006-beat-period.md) 已补作者按拍换算／有界敲拍，[EFFECT-005](development/tasks/EFFECT-005-reuse-and-order.md) 已补复用来源／目标预检与整组灯序整理，[EXEC-003](development/tasks/EXEC-003-execution-keyboard.md) 已补焦点限定键盘执行／松键保护，[EXEC-002](development/tasks/EXEC-002-preview-rate.md) 已补连续临时预演速率，[STAGE-004](development/tasks/STAGE-004-curved-seating.md) 已补弧形座区；[AUDIO-011](development/tasks/AUDIO-011-timeline-group-motion.md) 已补时间线整组直接移动与输入保护，[AUDIO-010](development/tasks/AUDIO-010-phase-preserving-split.md) 已补保持效果进度的分割／重置，[AUDIO-009](development/tasks/AUDIO-009-timeline-clip-selection.md) 已补时间线框选和目录共用选择，[AUDIO-008](development/tasks/AUDIO-008-lighting-clip-enable.md) 已补片段停用／恢复与默认值空隙，[AUDIO-007](development/tasks/AUDIO-007-lighting-clip-groups.md) 已补片段成组整理与锁定／事务保护，[AUDIO-006](development/tasks/AUDIO-006-lighting-clips.md) 已补独立灯光片段、空隙、移动／复制／锁定，[AUDIO-005](development/tasks/AUDIO-005-local-loop-preview.md) 已补原生有界局部循环，[AUDIO-004](development/tasks/AUDIO-004-marker-group-editing.md) 已补成组卡点整理，[AUDIO-003](development/tasks/AUDIO-003-lighting-transitions.md) 已补音乐灯光段落进入渐变，[FIXTURE-003B](development/tasks/FIXTURE-003B-function-authoring.md) 已贯通功能区间建档、场景／预设和包兼容；[FIXTURE-003A](development/tasks/FIXTURE-003A-discrete-playback.md) 提供直接切换执行基础。本表是当前能力与验证边界的入口；后续增量和当前窗口／实板状态看 [STATE](development/STATE.md)，历史过程看各工单。技术框架为 Rust 核心、Tauri 2＋React／TypeScript 界面；云端 Fastify＋PostgreSQL＋对象存储尚未实施。

已有真实编辑、音频、内嵌预演及设备安装链路，仍是开发版。界面可操作、计划可编码、设备安装成功分别有证据；真实 RS485 输出仍禁用，不能据此宣称可交付演出。

| 能力 | 已实现与证据 | 仍未完成／验收边界 |
| --- | --- | --- |
| 编排工作台 | [专注编排](development/tasks/UX-032-focus-editing-layout.md)：任务各自保持、侧栏与音乐控制保留、显式三维恢复；窄步骤区自适应 | 自由停靠／命名布局、完整辅助功能、独立用户测评 |
| 工程编辑与持久化 | Rust 原子事务／撤销重做、严格读取、修订与保存冲突、[容量保护](module-api/project-capacity.md)、[崩溃恢复](module-api/project-recovery.md)、[最近工程](development/tasks/UX-024-recent-projects.md) | 未应用输入草稿恢复、版本迁移／比较、云端协作；其他平台需独立验收 |
| 灯具定义与配适 | [FIXTURE-002](development/tasks/FIXTURE-002-profiles-patch.md)：工程内调光／RGB／双轴、8/16 位任意粗细映射、默认值、使用中模式保护、明确换灯、批量改址／占用图；[FIXTURE-003B](development/tasks/FIXTURE-003B-function-authoring.md) 增加单色盘／图案盘／快门频闪／棱镜的命名区间、类型化选择、8/16 位编码及兼容换灯；[FIXTURE-004](development/tasks/FIXTURE-004-portable-modes.md) 增加可携带模式文件、严格读取及导入草稿；FIXTURE-005 增加全范围线性变焦／调焦／光圈及独立预设范围 | 物理光学元数据、关联／控制通道、个人灯库、GDTF／OFL 导入、多单元和真实试灯 |
| 灯组／预设与选择 | [资源模块](module-api/editing-library.md)、[中央平面选择](development/tasks/UX-026-scene-plan-selection.md)、[编排平面查看](development/tasks/UX-031-shared-stage-overview.md)、[搜索面板](development/tasks/UX-027-searchable-resources.md)：有序选择、追加／扣除、预设引用／独立值、依赖与更新保护 | 通用共享预设、配方、完整克隆／跨能力换灯 |
| 常规场景编排 | [编排流程](development/tasks/UX-018-scene-editing-flow.md)：亮度／颜色／位置属性、批量混合值、场景复制、显式对象／版本预演 | 专业编程器来源追踪、盲编、分部／阻断继承／仅当前更新、暗场预定位 |
| 动态效果 | [效果模块](module-api/lighting-effects.md)：亮度／RGB 曲线、32 帧、三种过渡、灯序／相位；[EFFECT-003](development/tasks/EFFECT-003-relative-position-effects.md) 加入相对物理角度双轴运动；[固定属性编辑](development/tasks/UX-020-docked-effect-editing.md) 保持三维可见；[EFFECT-004](development/tasks/EFFECT-004-live-draft-preview.md) 草稿即时预演／错误保持／取消恢复；[EFFECT-005](development/tasks/EFFECT-005-reuse-and-order.md) 来源／目标预检、分页检索、名称／配适／奇偶／反转排序；[EFFECT-006](development/tasks/EFFECT-006-beat-period.md) 按拍试算／显式采用与有界敲拍 | 功能区间内渐变／命名档位追逐、连续世界目标轨迹、速度／加速度约束、现场速度主控／节拍、像素；当前正弦采用有界采样，详见契约 |
| 摇头位置 | [POSITION-001](development/tasks/POSITION-001-moving-head-workflow.md)：独立两轴范围／反向、零偏、静态共同点、平面选点／精确高度、轴角渐变和关节预演；[POSITION-002](development/tasks/POSITION-002-relative-axis-and-flip.md) 逐灯相对微调、实际通道精度保护／等指向翻转与整批拒绝 | 非相交轴／多头、自动校准、实灯精度／碰撞；相对运动不能等同持续目标跟随 |
| 单列表执行 | [播放核心](module-api/sequence-preview.md)、[专注执行视图](development/tasks/UX-021-execution-view.md)：延时／渐变／自动等待、人工推进、跳转／暂停／循环、当前／下一步／选择分离与旧版本保护；[键盘执行](development/tasks/EXEC-003-execution-keyboard.md) 显式开启、松键／忙保护、焦点／跨页／载入退出；[剧本提示](development/tasks/SEQUENCE-002-script-prompts.md)含幕场、台词／动作、备注及检索；[成组整理](development/tasks/SEQUENCE-003-step-group-editing.md)支持按原序复制／移动／删除和一次历史；[整组时间](development/tasks/SEQUENCE-004-group-timing.md)支持按字段改延时／渐变／推进、混合值与草稿保护 | 多执行器现场混合、真实输出总控、临时覆盖／归还、完整剧本关联及人工／定时混合调度 |
| 预演运行总控 | [EXEC-001](development/tasks/EXEC-001-preview-output-master.md)：常驻亮度、独立熄灯锁存、数值调光／纯 RGB、未覆盖提示、播放不中断、原生三维联动；[预演速率](development/tasks/EXEC-002-preview-rate.md) 25–400% 连续时间缩放、暂停保持／载入复位，不改工程时间 | 真实设备／机构安全联锁、多执行器合成、组主控与独立效果速率 |
| 音乐与灯光卡点 | [音频模块](module-api/audio-editing.md)、[成熟波形](development/tasks/AUDIO-002-professional-waveform.md)、[灯光段落](development/tasks/UX-022-audio-lighting-lane.md)：真实音频、WaveSurfer、裁切／定位／手动标记／场景绑定、边界编辑和统一历史；AUDIO-003 支持确定性进入渐变／任意定位，功能属性保持直接切换；AUDIO-004 支持保留节奏的成组平移／复制／删除；AUDIO-005 临时局部循环共用音频游标；AUDIO-006 独立灯光片段／空隙／移动／复制／锁定、显式兼容转换；AUDIO-007 片段成组移动／复制／删除、隐藏选择与一次历史；AUDIO-008 持久单／组停用和恢复、状态筛选与锁定保护；AUDIO-009 时间线框选／键盘及目录共用组选择，点选不误吸附；AUDIO-010 渐变结束后保相位分割、显式效果起点／重置与保存；AUDIO-011 整组拖动／吸附／键盘及目标草稿互斥；AUDIO-012 片段／裁切／组移动／框选边缘滚动；AUDIO-013 裁切保留动态效果进度、源零点／一小时约束与精确输入；AUDIO-014 卡点／旧段落边缘滚动、点击阈值与取消；AUDIO-018 渐变中分割、历史过渡内部截取与显式重新计算，保留渐变和效果源进度 | 自动拍子、多轨、重叠／双场景动态持续交叉、跨设备时钟和有线／蓝牙延迟校准；音乐不存 ESP32 |
| 舞台与场地 | [装配](development/tasks/STAGE-001-rigging-workflow.md)、[场地目录](development/tasks/UX-025-stage-organization.md)：空间／尺寸、桁架／挂灯、阵列／对齐、测距、显隐／搜索／精确输入及一次历史；[场地锁定](development/tasks/STAGE-002-object-edit-locks.md) 保护单／多选和二维／三维、桁架联动；[参数座区](development/tasks/STAGE-003-parametric-seating.md) 支持排数／净通道、整区编辑及 UE 同源座椅；[灯位显示](development/tasks/UX-030-fixture-plan-symbols.md) 共享能力符号／图例／名称和地址标注；[弧形座区](development/tasks/STAGE-004-curved-seating.md) 同心排、逐座朝向、净通道和即时参数预览；[密集灯位](development/tasks/UX-033-fixture-label-collision.md) 三处平面标签避让／选中优先／长名称截短 | 轮廓裁切／逐座编辑、门洞／共享墙、复杂吊点、完整三维组变换 |
| 程序内三维 | [PREVIS-002](development/tasks/PREVIS-002-single-workspace.md)：唯一 UE 视窗与共享播放进度、跨页保持；[UX-023](development/tasks/UX-023-previs-session-contention.md) 处理短时锁竞争；未建模功能灯具明确提示并保留灯位／姿态，不输出假光束 | 仍依赖本机 UnrealEditor；独立运行时打包、专业光学／轮盘模拟、规模／延迟预算和完整辅助功能 |
| 工程检查与素材交付 | [检查](module-api/project-check.md)、[资源健康](development/tasks/UX-028-project-resource-health.md)：编译／配适定位、资源摘要、缓存／随附文件分别检查、缺失重定位与保存补齐；[配灯表](module-api/patch-report.md)提供全灯具型号／地址／世界安装位置和快照来源 CSV，原生 80 灯实际导出验收；[节目单](module-api/sequence-report.md)导出列表原顺序、剧本提示、场景与人工／自动时间，原生六步逐列核对通过 | 当前资源检查针对已接入音乐文件；完整多媒体依赖、导出图纸、运行来源诊断未实现 |
| 编译／播放包／设备运行层 | [有界包](module-api/playback-package.md)、[安装](module-api/package-installation.md)、[传输](module-api/package-transfer.md)、[NOR](module-api/nor-package-store.md)、[运行模块](module-api/device-runtime.md) 已实施并通过相应故障／重放验证；FIXTURE-003A 已补直接切换属性与执行语义 2 包，旧包仍兼容 | 正式设备运行控制、独立本地面板、物理输出及整链路长期压力仍待验；主机故障注入不是所有板级掉电证明 |
| BLE 与实板存储 | [DEVICE-002 完整验收](development/tasks/DEVICE-002-direct-installation-acceptance.md)：免系统配对加密 GATT、身份／开发权限、下发／取消／续传／结果对账；[MEMORY-001](development/tasks/MEMORY-001-bounded-board-memory.md)：8 MB PSRAM 自检及 2 MB 有界缓存 | 现有开发凭据不等于生产身份、商业许可、安全启动（策略范围见 ADR-101）；容量上限仍按目标预算校验 |
| DMX 与首版播放盒 | 共享内核在 ESP32-S3 验证，支持单路 512 通道数据编码；实板安装已通过 | GPIO21／RS485 发送仍禁用；UART 时序、电气、运行到实灯及脱机操作闭环未通过。ARM／其他盒子尚未验证 |
| 多源混合旧原型 | A0 的 HTP／LTP 与基础跟踪仍独立保留；G0 修复 R01／R02 并保留回归 | 不等同新单列表的完整专业合成器；[架构审查](architecture-review.md)中的 R03／R06／R09 等没有因新模块通过而自动关闭 |
| 扩展方向 | [伪 API](module-api/README.md)、[AI 编辑](module-api/assisted-editing.md)、[音视频／机构](audiovisual-stage-design.md)、[实体控制面](hardware-control-surfaces.md)、[采集重建](venue-capture-design.md)已有设计 | 对应真实服务、外部协议、机械控制适配、硬件面板、采集导入尚未实施；没有以设计稿冒充运行接口 |
| 云端／跨端 | 已明确共享 Rust 语义、TS 界面／云端和可替换宿主边界 | 云服务、生产授权、iPad／网页编辑与其他桌面平台尚未交付 |

## 验证命令

FIXTURE-005 通过 594 Rust、233 UI、类型／格式／严格检查／桌面及组件／原生建档、取消历史、预设隔离和保存重开，见[记录](development/tasks/FIXTURE-005-continuous-optics.md)。

FIXTURE-004 通过 590 Rust、227 UI、类型／fmt／严格检查／桌面、组件生命周期与原生跨工程／历史／保存重开，见[记录](development/tasks/FIXTURE-004-portable-modes.md)。

REPORT-001 通过 582 Rust、227 UI、类型／fmt／严格检查／桌面及原生导出、逐项内容核对和历史保护，见[记录](development/tasks/REPORT-001-patch-report-export.md)。

AUDIO-014 通过 227 UI、类型／格式／桌面及组件／原生历史保存重开，见[记录](development/tasks/AUDIO-014-marker-edge-scroll.md)。UX-033 通过 223 UI、类型／格式／桌面与真实组件／原生三视图验证，见[记录](development/tasks/UX-033-fixture-label-collision.md)。AUDIO-013 通过 571 Rust＋历史 1、218 UI、113 格式、类型／fmt／严格检查／桌面及组件／原生裁切取消历史保存重开，见[记录](development/tasks/AUDIO-013-phase-preserving-trim.md)。EFFECT-006 通过 214 UI、类型／格式／桌面及组件、原生即时预演／取消／历史／保存重开，见[记录](development/tasks/EFFECT-006-beat-period.md)。EFFECT-005 通过 209 UI、类型／格式／桌面、104 项组件复用及原生排序／取消／历史／保存重开，见[记录](development/tasks/EFFECT-005-reuse-and-order.md)。EXEC-003 通过 205 UI、类型／桌面与原生焦点、搜索、跨页、推进及暂停／停止，见[记录](development/tasks/EXEC-003-execution-keyboard.md)。EXEC-002 通过 566 Rust、201 UI、类型／fmt／严格检查／桌面与原生精确输入／取消／跨页／重载／UE 同源，见[记录](development/tasks/EXEC-002-preview-rate.md)。STAGE-004 通过 559 Rust、200 UI、113 格式、类型／fmt／严格检查／桌面及原生创建／取消／历史／保存重开／内嵌 UE 同步，见[记录](development/tasks/STAGE-004-curved-seating.md)。EFFECT-004 通过 553 Rust／最终 64 桌面、195 UI、严格检查／双端构建、5 UE 自动化与原生编辑／保存重开，见[记录](development/tasks/EFFECT-004-live-draft-preview.md)。AUDIO-012 通过 198 UI、类型／桌面与按住边缘手势、原生保存重开，见[记录](development/tasks/AUDIO-012-timeline-edge-scroll.md)。AUDIO-011 通过 190 UI、类型／桌面、4 Rust 回归和真实手势／原生保存重开，见[记录](development/tasks/AUDIO-011-timeline-group-motion.md)。AUDIO-010 通过 546 Rust、185 UI、112 格式、类型／fmt／严格检查／桌面／Xtensa 和原生连续分割／历史／保存重开，见[记录](development/tasks/AUDIO-010-phase-preserving-split.md)。AUDIO-009 通过 183 UI、类型／桌面、原生框选／复制历史／保存重开，见[记录](development/tasks/AUDIO-009-timeline-clip-selection.md)。AUDIO-008 通过 536 Rust、181 UI、109 格式、类型／fmt／严格检查／桌面与原生状态／历史／保存重开，见[记录](development/tasks/AUDIO-008-lighting-clip-enable.md)。AUDIO-007 通过 532 Rust、179 UI、类型／fmt／严格检查／桌面与原生片段组保存重开，见[记录](development/tasks/AUDIO-007-lighting-clip-groups.md)。AUDIO-006 通过 509 Rust、165 UI、107 格式、类型／fmt／严格检查／桌面与原生片段及 UE 联动，见[记录](development/tasks/AUDIO-006-lighting-clips.md)。AUDIO-005 通过 490 Rust、151 UI、类型／fmt／严格检查／桌面及原生循环与内嵌 UE，见[记录](development/tasks/AUDIO-005-local-loop-preview.md)。AUDIO-004 通过 484 项 Rust、148 项 UI、类型／fmt／严格检查／桌面及原生组编辑验收，见[记录](development/tasks/AUDIO-004-marker-group-editing.md)。AUDIO-003 通过 475 项 Rust、136 项 UI、99 项格式检查、fmt／严格 Clippy／类型、桌面构建；原生渐变／取消撤销／保存重开／音乐和 UE 见[任务验收](development/tasks/AUDIO-003-lighting-transitions.md)。此前 FIXTURE-003B 的功能建档与 Xtensa 检查见[验收](development/tasks/FIXTURE-003B-function-authoring.md)。此前已独立修复验证 [STORE-001](development/tasks/STORE-001-explicit-lock-lifetime.md) 存储锁生命周期。测试数量是该构建的记录，不是商业成熟度评分。现有实板未刷入新的能力声明，仍需兼容检查。

项目使用本地离线依赖缓存；终端从根目录运行：

```sh
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo fmt --all -- --check
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo test --workspace --locked --offline
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo clippy --workspace --all-targets --locked --offline -- -D warnings
npm --prefix apps/ui-prototype run check
npm --prefix apps/ui-prototype run test
npm --prefix apps/ui-prototype run desktop:build
```

格式与伪接口的验证说明分别见[工程格式](project-format/README.md)和[模块接口](module-api/README.md)。仅通过 Schema／类型检查不能证明服务、鉴权、实时性或物理输出已实现。设备相关构建、真实试验、失败记录与资源限制保留在对应工单，不以本机软件测试代替。

2026-10-01：[SEQUENCE-005](development/tasks/SEQUENCE-005-group-script.md) 已交付批量幕场／台词／备注，保留原值、统一填写和明确清空；与批量时间合成一次事务，含混合值、错误定位、独立取消及隐藏选择保护。606 Rust、252 UI、严格检查／类型／桌面与原生历史／保存重开通过；运行提示快照保持不变。

2026-10-01：[EXEC-004](development/tasks/EXEC-004-step-navigation.md) 执行列表独立浏览区、定位下一步和显式运行跟随已完成；手动／搜索／跨页／载入版本变化退出，选择与播放不变。255 UI、类型／格式／桌面及长列表组件／原生验证通过，工程未改。

2026-10-01：[UX-042](development/tasks/UX-042-pinned-resources.md) 常用灯组与预设可固定，按工程身份在本机保持，各类最多 8 项／最多 20 工程；复用有序召回和明确预设应用，坏存储／容量／删除恢复保护。258 UI、类型／格式／桌面及原生重启、项目隔离通过，工程未改。
