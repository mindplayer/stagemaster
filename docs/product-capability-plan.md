# 产品能力总表与成熟经验吸收规则

更新：2026-10-03，框架能力导航已补 PREVIS-003／TIME-001 软件增量；初始 PLAN-001，PLAN-002 增补内存与宿主音频。这里只维护能力导航与缺口审查，不是完成清单或冻结的接口规范。实现证据看[实现状态](implementation-status.md)，当前顺序看[执行计划](development/execution-plan.md)。

用户要求当前会话主动引入成熟软件的重要、必要能力，不只等待逐项提出需求。已有[控台研究](console-research/README.md)覆盖 22 个模块、304 条对照记录，另有界面、预演、媒体和硬件专项；本表将它们归到统一产品工作流，不另复制一套研究目录。

2026-10-02 框架优先：依 [ADR-097](development/decisions/PRODUCT-ADR-097-integrated-stage-platform.md)，声光电编排／现场控、专业预演、云端商业化、有线／无线解码器与电脑／平板／手机／实体控台构成统一长期产品范围。先验证状态归属、时钟、控制权与可替换适配，不继续以局部界面细化作为当前主线。既有能力证据保留；PLAN-003 不代表各端或新能力已经实现。

## 采用方式

先确定用户要完成的事情，再吸收对应机制：做一个节目、换一批灯、换一个场地、交给另一人、下发到盒子、现场排错。借鉴功能时同时考虑它的依赖、失败处理和撤销，不能只移植按钮。

每项能力区分三个时间点：**边界何时必须确定、最小实现何时需要、专业扩展何时加入**。工程格式允许版本演进；现在用代表性用例验证边界，不试图一次冻结所有未来字段。相同能力只保留一个权威实现，桌面、平板、网页和实体面板复用命令。

表中“首版子集”指首次软件＋单路播放盒所需部分；“专业迭代”不自动成为首版前置条件；“专项接入”须按目标设备与协议单独验收。商业安全按 ADR-101 保留边界，具体加密授权方法在 AUTH-001 后续讨论，暂不考虑限时；不因云端协作后置而遗漏生产保护。

## 能力落位

“原型”仅表示静态核心或独立交互已有局部证据；“设计”包括研究、接口或格式草案，均不表示正式产品可用。模块名称沿用[伪 API](module-api/README.md)，表示职责，不能据此一次新建二十个服务或 crate。

| 编号／能力 | 应纳入的实际功能 | 成熟依据 | 模块与界面位置 | 当前证据／实施层次 |
| --- | --- | --- | --- | --- |
| CAP-01 空间与布置 | 多房间／异形轮廓、标高／净高、舞台／观众区／过道、构件复用、对齐／吸附／阵列、测量、图层与隔离查看 | [Vectorworks／SketchUp／Depence 研究](ui-design/venue-layout-design.md)；[多空间决定](development/decisions/PRODUCT-ADR-010-composable-spaces.md) | 工程空间领域、场地资源与预演；布置工作区 | 空间尺寸、灯位阵列和 STAGE-001 挂接／关联变换已实施；[UX-025](development/tasks/UX-025-stage-organization.md) 已补分层目录、平面显隐、搜索和显示精度。[STAGE-002](development/tasks/STAGE-002-object-edit-locks.md) 已补工程级对象锁、批量保护与二维／三维一致限制。[STAGE-003](development/tasks/STAGE-003-parametric-seating.md) 已补矩形座区、净通道及区域级编辑／预演；UX-030 已补两处平面的灯位能力符号／图例／地址标注；[UX-031](development/tasks/UX-031-shared-stage-overview.md) 步骤／音乐页可共享只读平面与导航。[STAGE-004](development/tasks/STAGE-004-curved-seating.md) 已补同心弧形座区、逐座朝向、净通道和共用参数预览。[STAGE-005](development/tasks/STAGE-005-shared-3d-fixture-movement.md) 已补二维／三维共享选组与整组水平／升降。复杂吊点、门洞／共享墙、裁切／逐座编辑和图层锁继承后续 |
| CAP-02 灯具定义 | 自定义亮度／颜色等能力，模式／年代变体、多单元、功能分段、色盘／图案盘、档案版本、测试记录与个人灯库 | [MA3／Titan／GDTF／OFL 建档研究](ui-design/fixture-definition-design.md) | FixtureLibrary＋工程领域＋DmxEncoder；灯具编辑器／灯库 | FIXTURE-002 已实现工程内调光／RGB 的 8/16 位建档和显式换灯；POSITION-001 已接双轴范围／反向。FIXTURE-003A／B 已贯通单色盘／图案盘／快门频闪／棱镜区间、离散编排和包兼容；[FIXTURE-004](development/tasks/FIXTURE-004-portable-modes.md) 已接模式文件跨工程导入／导出与独立身份；物理光学、关联控制通道、个人库管理、受控试灯及标准导入／多单元后续 |
| CAP-03 配适与选择 | 地址占用、冲突定位、批量编号／配适、有序灯组、选择过滤、二维选择布局、换灯／扩灯／克隆和功能映射 | [M04](console-research/M04-fixtures-patch.md)、[M05](console-research/M05-selection-groups-layout.md)、下方 S1 | ProjectService／BindingService／SessionService；灯具表、舞台与选择组件 | 批量配适、占用图、有序灯组已实现；[UX-026](development/tasks/UX-026-scene-plan-selection.md) 中央平面选灯与统一选择、[UX-027](development/tasks/UX-027-searchable-resources.md) 检索／追加／扣除通过。跨能力换灯和完整克隆后续 |
| CAP-04 预设与复用 | 颜色／位置／图案预设，引用／硬值选择、依赖定位、更新影响范围、配方组合、模板与局部工程导入 | [M07](console-research/M07-presets-palettes.md)、[M11](console-research/M11-recipes-reuse.md) | 工程事务＋编程器＋编译器；资源区、属性区、影响预览 | 逐灯预设、引用／独立值、依赖与三种更新策略已实现；UX-019 固定属性范围与快捷预设、UX-027 搜索复用已验收。配方／通用共享／导入后续 |
| CAP-05 效果编辑 | 曲线／关键帧、分步效果、速度／幅度、相位、灯具顺序与分布、相对／绝对值、可复现随机、共同指向与路径、像素灯阵列及媒体驱动灯光 | [M10](console-research/M10-effects-phasers.md)、[M12](console-research/M12-pixel-media.md)、[效果设计](ui-design/effect-editor-design.md) | 工程效果领域＋ShowCompiler＋RuntimeKernel；舞台、效果组件、时间线 | 亮度／RGB 32 帧曲线、周期／灯序／相位和跨场景复用已实现；UX-019 曲线、UX-020 非模态编辑、[EFFECT-003](development/tasks/EFFECT-003-relative-position-effects.md) 相对双轴幅度／偏移／相位与三维通过；[EFFECT-004](development/tasks/EFFECT-004-live-draft-preview.md) 草稿即时预演／取消恢复；[EFFECT-005](development/tasks/EFFECT-005-reuse-and-order.md) 复用目标预检与整组灯序整理；[EFFECT-006](development/tasks/EFFECT-006-beat-period.md) 作者按拍换算／敲拍／显式采用。[EFFECT-007](development/tasks/EFFECT-007-world-line-effects.md) 共同世界目标直线往返、整线约束／误差认证与两灯 UE；任意空间路径、现场主控／节拍、像素后续；媒体驱动不在实时线程解码 |
| CAP-06 场景与场景列表 | 记录／更新、仅改当前或后续跟踪、分部、阻断继承、渐变／延时、跳转／循环、释放、暗场预定位 | [M08](console-research/M08-cues-tracking.md) | 工程事务＋运行核心；编排列表与现场当前／下一场景 | 静态场景／继承／释放、受限单列表的延时／渐变／人工推进／自动等待／循环已实现；UX-021 当前／下一步／跳转与版本保护已验收。SEQUENCE-002 幕场／台词／备注及运行快照已验收；[SEQUENCE-003](development/tasks/SEQUENCE-003-step-group-editing.md) 成组复制／移动／删除、筛选与历史及 [SEQUENCE-004](development/tasks/SEQUENCE-004-group-timing.md) 整组时间／混合值／原子修改已验收；[SEQUENCE-005](development/tasks/SEQUENCE-005-group-script.md) 三字段稀疏批量提示与时间合成事务已验收；高级跟踪、分部／阻断与完整剧本锚点后续 |
| CAP-07 时间编排 | 无损片段裁切、吸附、波形／标记、节拍、自动化轨、片段／段落复用、时间码、独立同步组 | [M13](console-research/M13-timecode-audio.md)、[创作软件研究](ui-design/interaction-display-research.md) | 工程时间领域＋ClockRegistry／TransportCoordinator；编排时间线 | AUDIO-001／002 已接真实音频、裁切／定位、WaveSurfer 波形与手动卡点；[UX-022](development/tasks/UX-022-audio-lighting-lane.md) 灯光段落边界编辑／撤销／保存通过；[AUDIO-003](development/tasks/AUDIO-003-lighting-transitions.md) 已补确定性进入渐变与任意定位；[AUDIO-004](development/tasks/AUDIO-004-marker-group-editing.md) 成组选择／平移／复制／删除已验收；AUDIO-005 有界原生局部循环已验收；[AUDIO-006](development/tasks/AUDIO-006-lighting-clips.md) 独立灯光片段、真实空隙、移动／复制／锁定和显式兼容转换已验收；[AUDIO-007](development/tasks/AUDIO-007-lighting-clip-groups.md) 片段成组整理与隐藏选择／事务保护已验收；[AUDIO-008](development/tasks/AUDIO-008-lighting-clip-enable.md) 单／组停用、默认值空隙与状态筛选已验收；[AUDIO-009](development/tasks/AUDIO-009-timeline-clip-selection.md) 时间线框选与共享组选择已验收；[AUDIO-010](development/tasks/AUDIO-010-phase-preserving-split.md) 渐变结束后的保相位分割／效果起点重置已验收；[AUDIO-011](development/tasks/AUDIO-011-timeline-group-motion.md) 明确工具／整组拖动／吸附／输入保护已验收。[AUDIO-013](development/tasks/AUDIO-013-phase-preserving-trim.md) 裁切保留效果进度／源范围／精确输入已验收。[AUDIO-014](development/tasks/AUDIO-014-marker-edge-scroll.md) 补齐普通卡点／旧段落边缘滚动与取消。[AUDIO-015](development/tasks/AUDIO-015-group-fade.md) 统一多片段进入渐变与固定身份草稿通过。[AUDIO-018](development/tasks/AUDIO-018-preserved-entry-fades.md) 已补历史过渡内部截取及渐变中分割。[AUDIO-019](development/tasks/AUDIO-019-dynamic-crossfade.md) 双场景动态交叉、保留来源时钟及渐变内剪辑已验收。[AUDIO-020](development/tasks/AUDIO-020-performance-loop-sections.md) 正式演出区段循环实施中，核心／工程、有界原生音源、桌面运行控制与共享运行栏已接通；区段可视编辑、真实音乐／UE 和原生保存重开验收未完成。多轨叠加、自动拍子与跨设备同步后续 |
| CAP-08 排练与预演 | 离线预览、盲编、从指定位置排练、二维／三维、组灯指向、视角收藏、外部控台输入、设备响应仿真 | [M14](console-research/M14-preview-3d.md)、[Depence](depence-r4-assessment.md)、S2 | PreviewService＋仿真＋独立渲染；编排画布、预演视图 | 唯一内嵌 UE 与共享播放进度已由 PREVIS-002 接通，UX-023 处理短时锁竞争；静态共同对焦和 EFFECT-003 相对双轴运动可预演。EFFECT-007 已补共同世界直线路径；[STAGE-005](development/tasks/STAGE-005-shared-3d-fixture-movement.md) 已补共享选组、聚焦与整组水平／升降。[PREVIS-003](development/tasks/PREVIS-003-background-observation.md) 已接后台固定场地／完整软件帧的只读观察，原生 UE 关闭／重开不改变执行与控制权。任意路径、三维组旋转／缩放、外部控台输入、专业光学与独立打包后续 |
| CAP-09 现场执行 | 当前／下一场景、节目单、执行器页、总控／速度控制、灯光熄灭、临时覆盖与归还、属性冻结、互斥／保护、重复触发处理 | [M09](console-research/M09-playback-mixing.md)、[M19](console-research/M19-live-show.md) | ControlGateway／RuntimeKernel／Mixer；现场工作区 | 独立 Rust 单列表执行和 UX-021 专注现场视图已实现；A0 多源混合仍属原型。[EXEC-001](development/tasks/EXEC-001-preview-output-master.md) 已补单机预演总控／熄灯、亮度识别与独立运行状态。[EXEC-002](development/tasks/EXEC-002-preview-rate.md) 已补主机单播放器 25–400% 临时预演速率、暂停保持和载入重置。[EXEC-003](development/tasks/EXEC-003-execution-keyboard.md) 已补显式启用的焦点限定键盘执行、松键／忙与跨页保护。[EXEC-004](development/tasks/EXEC-004-step-navigation.md) 已补当前／下一步定位与可暂停列表跟随、选择及播放位置隔离。[MIX-001](development/tasks/MIX-001-live-contributions.md) 已补有界属性合成，两个实际动态保持场景与手动层的覆盖／归还、亮度与释放分离、快照绑定编码和软件端口验证；[MIX-002](development/tasks/MIX-002-sequence-contributions.md) 已补逐步静态跟踪、延时后释放、明确跳转／顺序推进和循环折叠；[MIX-003](development/tasks/MIX-003-prepared-source-compositor.md) 已补固定来源组统一推进、实际时刻排序、累计容量及完整帧故障边界；[HOST-003](development/tasks/HOST-003-shared-live-host.md) 已接入共用控制权和原后台调度线程；[HOST-004](development/tasks/HOST-004-multi-source-process.md) 已贯通同一独立进程的受保护 v2 来源目录／控制／语义手动操作与回执；[HOST-005](development/tasks/HOST-005-desktop-background-execution.md) 已接桌面后台来源目录、明确控制权和独立操作，关闭编辑窗口后继续、重开只读接管通过；[EXEC-005](development/tasks/EXEC-005-source-execution-progress.md) 已补后台权威步骤／阶段进度；[EXEC-006](development/tasks/EXEC-006-execution-board-navigation.md) 已补类型／状态／常用／草稿导航与固定顺序、浏览输入保持；完整手动编程器、真实输出主控、高级跟踪／释放渐变与专业应急控制后续 |
| CAP-10 音视频与专项设备 | 宿主基础音频／系统音箱路由、近似声光同步；外部播放器／媒体服务器控制、内容绑定、状态回读、监听／监看和专项设备预演 | [多系统设计](audiovisual-stage-design.md)、[ADR-052](development/decisions/PRODUCT-ADR-052-memory-and-host-audio-sync.md) | 独立宿主音频适配＋TransportCoordinator；ExternalCommandGateway／MonitorService；音频轨、播放输出组件、监看面板 | AUDIO-001 真实本机音频与卡点、AUDIO-002 波形已实施；[TIME-001](development/tasks/TIME-001-independent-clock-boundaries.md) 正在验证独立时钟与媒体来源组，已接实际 Player／统一合成和软件端口，正式音源原生消费观测和原宿主有界准备／观测入口已补，独立进程／桌面完整音频所有权、暂停健康状态、重启与循环协调仍待接入；有线／蓝牙延迟校准、跨设备同步、外部播放器控制与视频监看后续。专业混音／视频处理仍外部执行 |
| CAP-11 互动与机构 | 传感器触发、条件／等待／超时、可复用动作组合、场次复位、手动接管、动作结果、机械控制器状态与联锁反馈 | [M15](console-research/M15-macros-automation.md)、[多领域格式](project-format/README.md)、[外部契约](module-api/external-contracts.ts) | 类型化事件／动作与外部网关；编排触发组件、设备诊断 | 格式示例；专项接入。电机运动闭环、限位和安全联锁由合适的现场控制器负责，不能当灯光渐变通道处理 |
| CAP-12 操作面板 | 推子／旋钮／按键、翻页映射、软接管、灯环／小屏反馈、用户可配置的简化现场面板 | [硬件控制面](hardware-control-surfaces.md) | SurfaceCoordinator／ControlGateway；现场及面板配置 | 接口草案；首版盒子本地选择／执行，扩展翼与可配置面板后续，输入均走同一控制入口 |
| CAP-13 工程资源交付 | 收集工程依赖、缺失素材重新定位、同内容去重、模板／局部导入、缩略图／波形缓存、交换包和依赖报告 | [M03](console-research/M03-show-files.md)、S3／S4／S5 | AssetService／PackageService／ProjectRepository；资源管理与工程菜单 | 音乐缓存／随附资源、缺失重定位、摘要及 UX-028 健康检查／保存补齐已实现；工程／设备包分开。多媒体依赖、交换／局部导入与完整资源管理后续 |
| CAP-14 编辑与恢复 | 原子批量编辑、具名撤销、连续拖动单次提交、错误草稿保留、自动恢复、保存冲突、迁移与版本比较 | [风险 H08／H09](architecture-change-risk-review.md)、[UX-010](development/tasks/UX-010-editing-recovery.md) | ProjectService／ProjectRepository；所有工作区共用 | 原子事务／撤销／保存冲突、RECOVERY-001 有效编辑恢复已实现；UX-020 共用非模态草稿事务，UX-032 专注编排保持任务布局／草稿及预演上下文。未应用输入草稿恢复、多版本比较仍后续 |
| CAP-15 编译与自主播放 | 依赖闭合、目标能力／分层内存预算、可重复构建、校验／安装／激活、中断恢复、掉电保护、已安装／运行版本、离线本地控制 | [首版方案](development/decisions/PRODUCT-ADR-002-compiled-playback-and-transfer.md)、[ADR-052](development/decisions/PRODUCT-ADR-052-memory-and-host-audio-sync.md) | TargetService／BuildService／PlanManager／TransferService＋板级内存适配；设备与下发面板 | DEVICE-002 实测加密 GATT 安装，MEMORY-001 实测 8 MB PSRAM 与 2 MB 有界缓存；安装／运行模块独立。真实 DMX、正式运行控制和完整自主播放待验；包上限不因标称内存静默放宽 |
| CAP-16 诊断与预检 | 工程错误定位、缺资源／不支持能力、输出来源／覆盖原因、原始通道值、端口故障、设备反馈新鲜度、触发记录、可导出的诊断包 | [M20](console-research/M20-maintenance-diagnostics.md)、S6 | 编译校验＋ObservationService；状态入口、问题列表、按需诊断 | CHECK-001 配适／编译诊断及导航、UX-028 音乐资源完整性／过期保护已实现；界面区分预览、安装和物理输出。完整来源链、端口实测和诊断包后续 |
| CAP-17 云端与授权 | 工程／素材版本、发布与分发、权限、设备绑定、授权状态、同步冲突与离线副本 | [架构 C05](architecture-evolution-review.md)、[商业安全边界](development/decisions/PRODUCT-ADR-101-commercial-security-boundaries.md) | Publication／Distribution、独立授权边界；项目／资源／设备管理 | 无云服务；AUTH-001 是相关商业交付门槛，协作／云盘后置，不进入输出时钟链 |
| CAP-18 开放接入与跨端 | GDTF／MVR 等交换、Art-Net／sACN／RDM、OSC／MIDI 等协议适配、远程控制、网页／平板编辑、版本化扩展能力和兼容性报告 | [M16](console-research/M16-dmx-network.md)、[M17](console-research/M17-remotes-api.md)、[格式借鉴](development/decisions/PRODUCT-ADR-003-project-data-contract.md) | 导入导出适配、生成契约、宿主／传输；沿用四工作区 | 设计；现在保持接口可移植，协议按软硬件能力逐项验证，不承诺任意格式无损往返或现有板卡支持全部协议 |
| CAP-19 运行连续性 | 断连策略、程序与系统故障恢复、运行记录、备份／恢复、后续主备切换与输出所有权交接 | [M18](console-research/M18-sessions-backup.md)、[架构 C09](architecture-evolution-review.md) | RuntimeKernel／PlanManager／OutputArbiter；现场与设备状态 | 已实现软件运行／安装故障恢复、读源租约、GATT 保活失效／恢复及双槽安装；[HOST-001](development/tasks/HOST-001-independent-runtime-host.md) 已补独立调度宿主库、有界入口／观察和客户端断连继续的软件验证，[HOST-002](development/tasks/HOST-002-local-execution-process.md) 已验证真实独立执行进程；[HOST-005](development/tasks/HOST-005-desktop-background-execution.md) 已接桌面后台入口和原生重连，旧音频／草稿预演仍保留独立生命周期；[OUTPUT-001](development/tasks/OUTPUT-001-port-authority.md) 已以真实 Runtime＋软件驱动验证单端口来源切换、完成回执与静默维护；完整脱机演出、物理输出、长时压力和热备接管未验收，不能仅靠心跳判定接管 |
| CAP-20 现场交付资料 | 灯位图、地址／模式／通道表、设备与资源清单、标签、安装与校准记录、节目备注／检查单、带版本的导出 | [Depence 图纸职责](depence-r4-assessment.md)、[M04](console-research/M04-fixtures-patch.md) | 工程只读投影＋PlotService／导出；布置与工程菜单 | [REPORT-001](development/tasks/REPORT-001-patch-report-export.md) 已实现有界快照 CSV 配灯表与冲突保护，80 灯原生导出通过；[REPORT-002](development/tasks/REPORT-002-sequence-report-export.md) 已补场景列表节目单，六步剧本／时间／来源逐列验证通过；灯位图纸／其他交接资料待补，结构计算不在通用绘图能力内 |
| CAP-21 AI 辅助编辑 | 按任务查询对象／能力、生成修改提案、查看差异／隔离预演、指定范围连续编辑、原子应用／撤销与结果对账 | [ADR-011](development/decisions/PRODUCT-ADR-011-assisted-editing.md)、[接口与 MCP 取舍](module-api/assisted-editing.md) | 编辑自动化窄代理＋ProjectService；编排中的按需助手组件，不独立复制灯光规则 | AI-001 仅可类型检查的接口与例子；先接已有场景参数，后续随核心能力扩展，不是首版前置条件 |

[UX-033](development/tasks/UX-033-fixture-label-collision.md) 已补三处平面灯位标签避让、选中优先与字素缩略；不改变物理位置，其他空间文字统一排布仍后续。

检索、标签、批量属性、多选混合值、键盘／触控板操作、焦点与错误定位是横贯这些能力的基础交互，不另建一个“高级模式”。未来触控与实体面板使用相同语义，操作手势可以不同。

2026-09-30 用户补充：CAP-06 场景列表／步骤、CAP-07 时间编排和 CAP-09 现场执行须共同覆盖按剧本编排，音乐为可选输入。按 [UX-016](ui-design/workspace-framework.md) 增量加入幕／场或段落、台词／动作提示关联、人工等待和局部定时段落；共用场景／资源，分清编辑选择与执行位置。UX-017 已实现新整体工作台布局，SEQUENCE-002 已接步骤剧本提示和检索；完整剧本文档锚点和混合调度尚未实现，现有人工列表能力继续复用，不新增平行工程格式或第二播放器。

## 这次补齐的工作流，而非重新发明架构

以下大部分已有架构依据；这次补的是用户可完成的闭环、归属与验收时机。不能因表里出现一个名词就称当前格式或代码已经支持。

| 缺口 | 已有基础与本次补充 | 必须具备的验收例子 | 最迟落地时机 |
| --- | --- | --- | --- |
| 换场地／换灯后的编排复用 | CAP-02／03／04 已有档案、配适、预设方向；补功能映射差异、未匹配属性、校准和重算影响的统一入口 | 同一节目从 RGB 灯换到带色盘灯，明确哪些颜色不能等价；预览差异后提交，撤销能恢复原档案与引用，不悄悄删失数据 | FIXTURE-002／003B 已落地同属性模式替换、保留身份与引用；FIXTURE-007 将功能控制映射与显示外观分开比较，显式替换允许仅名称／外观不同，允许改变粗细地址；跨能力映射、校准及不等价效果仍后续 |
| 多空间与重复构件 | ADR-010 已定义空间与坐标；补“构件定义、摆放实例、选择灯组、显示图层”不能混为一种组 | 同一桁架构件放到两个不同标高房间，移动其中一个不改另一个；隐藏房间不改变输出；更新定义要能列出受影响实例 | 空间持久化契约建立时；实例化的精确类型另审查，不在本表冻结 |
| 把工程交给另一台电脑 | 资产与两类包已有边界；补收集依赖、缺失定位／替换、缓存可重建、外部系统内容清单 | 搬走原目录后，自包含的编辑资源仍可打开；外部播放器所需内容单列为“待验证”，不能因 JSON 打开成功而宣称可演出 | 首个真实媒体／模型资源接入时，不等云盘上线 |
| 排练、跳时间与现场并存 | 独立预演、同步组和外部动作已有设计；补逐种动作的排练／定位策略与可见输出目标 | 拖时间定位可重算灯光状态；不会补发一次性的机械动作或触发激光；外部设备不支持定位时显示限制，不能假装已同步 | 时间线／外部动作运行契约实施前；首版预览先证明没有真实输出路径 |
| 临时修改与正式版本 | 编辑事务和编译／激活已有边界；补现场覆盖的来源、作用范围、归还入口与修订差异 | 排练时修改预设不改变正在运行的旧计划；临时调暗有明确范围，归还后按既定运行政策恢复；撤销编辑不等同回退现场 | 首次接通正式执行前；高级覆盖工具后续 |
| 开演前检查和出错定位 | 已有领域校验与诊断投影；补统一问题导航和证据等级 | 能从“灯没亮”查看配适、属性来源、被总控抑制、生成帧、端口发送和设备回读；无设备回读时不显示“灯具已亮” | 首版覆盖实际支持的链路；高级来源随功能加入 |
| 操作人员与场次复位 | 已有现场／控制面／外部动作方向；补简化面板的范围、联动触发清单、一次场次结束后的复位职责 | 操作面板只执行指定范围；重新开始密室场次有明确状态，断线重连不重复触发机构；重复按键处理有单独政策 | 面板与互动任务开工时；首版盒子仅落实其选择／执行语义 |

## 提前检查的核心边界

这些是每个相关实施任务必须回答的问题，沿用现有架构和 ADR；本次不修改 JSON Schema 或 API。

1. **身份与引用。** 稳定对象 ID、显示编号、排序、安装实例和 DMX 地址分离；资源引用锁定修订；复制、删除、换灯和导入须说明依赖处理。场景不以通道地址作为业务身份。
2. **能力与值。** 亮度比例、角度、距离、颜色、图案档位和设备动作分开；粗细通道、属性联动及未知能力有明确支持范围。不把所有值都变成 0–255 滑杆，也不用任意 JSON 绕过领域校验。
3. **时间与事件。** 时间单位／时基、同步组、持续状态和一次性动作分开；定位、循环、暂停、重连分别定义。可重算的效果与不可撤销的外部动作不能共用隐式重放规则。
4. **编辑与运行。** 草稿、工程修订、编译产物、已准备版本、运行实例分别存在；编辑撤销、临时覆盖归还、现场停止分别处理。客户端断开与输出停止不是同一事件。
5. **几何与视图。** 局部／世界坐标、单位、安装／校准与指向、空间与连接有明确关系；个人镜头、折叠和隐藏不改变灯光业务。UE 等渲染适配消费中立数据。
6. **资源与端侧裁剪。** 工程包可含编辑资源，目标播放包仅闭合其执行依赖；预算超限拒绝或要求显式简化，不静默丢功能。缩略图和缓存不成为唯一原始数据。
7. **控制与反馈。** 输入意图、已接纳、运行已应用、端口已发送和设备已报告分开；控制权、过期反馈、超时对账和重复命令属于公共契约，不散落在按钮处理函数中。灯光熄灭、停止节目与机构安全停机不能合并成“全部通道归零”。
8. **演进与恢复。** 明确版本、所需能力、迁移、恢复与未知字段政策；遇到不支持的执行语义不发布，不能通过保存抹掉它。云端凭据和本机端口连接信息不混入可携带的编排内容。

对应详细依据：[架构风险 H01–H10](architecture-change-risk-review.md)、[扩展审查 C01–C10](architecture-evolution-review.md)、[模块调用规则](module-api/README.md)、[工程格式](project-format/README.md)。边界用具体代表性用例推动，不要求先实现完整扩展框架、插件市场或多机系统。

## 界面和实施顺序

主工作区仍为**布置、灯具、编排、现场**，依能力逐步开放。资源、设备、监看、问题列表和工程管理通过按需组件进入；投影、激光、机械装置等在对象／轨道／属性中表达，不为每种设备新增一个顶级页面。详见[组件化工作区](ui-design/modular-workspace-design.md)。

当前优先把已收敛的选择、属性、错误与撤销交互接到真实工程，再按既定工单完善空间／灯具契约、场景列表／时间线、Rust 播放和盒子承接。新能力按以下次序判断，而不是一次点亮总表：

| 层次 | 必须得到的结果 | 可以随后扩展 |
| --- | --- | --- |
| 当前可见编辑增量 | 真实对象、明确选择、事务修改、保存重开；加入空间／灯具时完成其相应契约 | 高精度预演、任意 CAD、完整厂商灯库 |
| 首次软硬件闭环 | 支持范围内的编排／编译、真实设备状态、安装／激活、无电脑选择执行、故障／掉电验收、规定的商业授权 | 网络输出协议全集、热备、多用户协作 |
| 专业编排与现场 | 预设引用、效果与分布、跟踪编辑、换灯、排练、节目单、现场覆盖、工程归档与诊断 | 按真实项目迭代深度，不一次模仿全部控台菜单 |
| 多系统与多端 | 外部媒体控制／监看、互动机构、UE、平板／Web、云端分发各完成自己的闭环 | 每个目标声明能力和限制，不要求最小 ESP32 安装所有模块 |

具体前后依赖继续以[桌面迭代计划](development/desktop-iteration-plan.md)与[首版执行计划](development/execution-plan.md)为准。持续添加能力不等于持续搁置实现；每个实施任务只取一个可用增量，完成后再扩展。

## 新功能进入开发的固定记录

在现有总表和对应研究条目中先去重；属于同一能力的细节补回原项。只在开工时创建具体实施工单，至少记录：用户任务、参考机制与链接、采纳／舍弃原因、数据和状态所有者、界面入口、依赖与目标能力、失败／撤销、可验证出口。涉及公共格式／运行语义则先补架构决定。

“已列入规划”“契约通过”“代码原型”“正式软件验证”“设备实测”分别记录。正式界面只出现已接通的操作；保留独立设计稿作交互验证，不把它的样例或占位能力塞进正式产品。

## 本次补查的官方依据

核对日期：2026-09-24。MA3／Titan 的其他对照沿用[原研究基线与证据索引](console-research/README.md)，不把下面的局部复核宣称为全产品重新实测。StageMaster 的取舍是本项目判断，并非这些厂商的保证。

| 来源 | 已核对的机制 | 本项目取舍 |
| --- | --- | --- |
| S1 [Titan 19.0：修改配适／灯具替换](https://manual.avolites.com/docs/patching/changing-the-patch/) | 换灯复用编排，借助预设修正；属性功能可手工映射，自动对应有局限 | 按语义映射并显示不等价项；不承诺任意灯具交换后效果完全相同 |
| S2 [QLab 5：试听与试演](https://qlab.app/docs/v5/tools/auditioning-cues/) | 可按输出类型另设试演路由；普通预览仍可走正常输出，部分外部控制不受总试演选项影响 | 吸收独立试演用途；StageMaster 以执行上下文和输出权限隔离，不能只靠“预览”按钮名称 |
| S3 [QLab 5：保存和移动工作区](https://qlab.app/docs/v5/fundamentals/managing-workspaces/) | 可把引用媒体收集到工程文件夹，存在不能自动收集的外部依赖 | 收集本系统可管理的资源；外部内容、环境和授权依赖另列，不能把“打包成功”等同“现场就绪” |
| S4 [Ableton：收集并保存](https://help.ableton.com/hc/en-us/articles/209775645-Collect-All-and-Save) | 普通保存可仅存引用，收集操作复制所需资源；插件须另行安装 | 工程保存与可携带归档分开；资源清单和缺失状态对用户可见，不内置媒体工作站 |
| S5 [Ableton：查找和替换缺失媒体](https://help.ableton.com/hc/en-us/articles/360000346779-How-to-find-and-replace-missing-media-files) | 支持目录搜索、候选选择与手工重新定位 | 同名不同内容不能静默替代；区分找回原资源和主动换内容，显示影响并可撤销 |
| S6 [QLab 5：工作区状态](https://qlab.app/docs/v5/tools/workspace-status-window/) | 问题集中展示并定位对象，也能查看触发与相关日志 | 统一问题导航与诊断入口；故障是否阻断由本系统的能力、编译和执行政策决定 |

尚需在具体任务中验证：换灯保真范围、资源打包规模、几何编辑性能、跨端交互、协议延迟、端侧容量与物理输出。公开功能资料只证明可参考的机制，不证明 StageMaster 已实现或性能达标。

2026-09-27 主动审查：[FIXTURE-002 工作流缺口与顺序](development/tasks/FIXTURE-002-workflow-audit.md)。本轮以实际灯具接入打通建档、配适和预演；下一软件核心为运动档案／校准／共同指向，首版硬出口仍是独立播放盒真实 DMX，同时安排工程恢复和开演问题导航。

2026-10-01 F02 增量：[FIXTURE-005](development/tasks/FIXTURE-005-continuous-optics.md) 贯通全范围变焦／调焦／光圈建档、场景预设、渐变及包；属性语义由 Rust 维护，界面独立镜头建档区和预设范围，依赖既有通用连续求值。原生历史、预设隔离和保存重开通过；物理光学、关联通道和真实试灯仍后续。

2026-10-01 位置编排增量：[UX-040](development/tasks/UX-040-common-target-plane.md) 已接静态共同点的场地选点／精确高度／取消保护，状态属于位置编辑草稿，界面在场景位置属性，依赖现有场地投影与 Rust aim；24 灯原子求解／失败／历史／保存重开通过。连续目标轨迹和物理输出精度仍未实现或未验收。

2026-10-01 选灯与效果增量：[UX-041](development/tasks/UX-041-spatial-fixture-order.md) 已补灯组／效果共用的单轴世界位置排序，明确升降方向／同坐标稳定／未布置置后；状态属于现有成员草稿，依赖场地灯位和既有保存事务。组件与原生保存重开验收通过；完整空间网格／镜像相位后续。

2026-10-01 资源效率增量：[UX-042](development/tasks/UX-042-pinned-resources.md) 已补场景编辑常用灯组／预设固定栏，状态归本机有界偏好（工程身份＋资源身份），复用原召回／选择／明确应用；项目隔离、改名／恢复、容量／存储失败、原生重启通过，不改工程或播放。

场地目录增量 [UX-043](development/tasks/UX-043-stage-directory-navigation.md)：共享内部滚动、筛选外选择计数／找回、对象焦点导航与原选择保护；本机视图状态，不改工程与平面显隐。

场景资源增量 [UX-044](development/tasks/UX-044-scene-batch-copy.md)：显式多选／筛选外数量、范围与键盘选择、原序不重名整组副本，复用原子事务和嵌套效果独立身份；本机选择不改工程，引用原对象不迁移，原生历史／重开通过。后续补使用位置导航，再评估批量删除。

2026-10-02 引用检查增量 [UX-045](development/tasks/UX-045-scene-usage-navigation.md)：场景属性显示当前工程的直接使用位置，依据稳定身份派生；有界分页／搜索、经共享草稿保护精确打开对应步骤／片段／卡点，解除筛选并聚焦。依赖原 ProjectView 和编辑器，不改播放快照或工程内容；新旧音频工程原生验收通过，间接跟踪图／引用替换仍后续。

2026-10-02 [UX-046](development/tasks/UX-046-scene-removal-preflight.md)：场景属性／批量整理共用有界删除审阅，显示所有对象与使用位置，有引用则全组保护，可直接导航处理；沿用 Rust 原子批次与核心最终校验，取消／失败保持、一次历史／原生保存重开通过。

2026-10-02 [AUDIO-016](development/tasks/AUDIO-016-selection-view.md)：时间线共享视口适应单片段／组、卡点与旧段落，使用裁切后时间、明确退出跟随，键盘焦点／空组／草稿保护；270 UI 和真实组件／原生通过，无工程或播放位置变化。

2026-10-02 [AUDIO-017](development/tasks/AUDIO-017-marker-selection.md)：工作区拥有卡点组身份，目录与波形双向选择、范围追加、整组视图与固定目标输入，原生复制／历史／保存重开通过；不改持久格式，复用有序选择模型与核心批次。

2026-10-02 空间效果增量：[EFFECT-007](development/tasks/EFFECT-007-world-line-effects.md) 已接共同目标直线往返。作者路径归工程效果，空间数学归 stagemaster-spatial，有限关键帧归编译器、执行端复用既有播放器；界面为场景效果属性，依赖灯位／轴模型／零偏。两灯原生三维、精度拒绝、取消／历史／保存重开通过；任意多段路径、实灯运动约束和几何遮挡仍后续。

2026-10-02 摇头灯操作增量：[POSITION-002](development/tasks/POSITION-002-relative-axis-and-flip.md) 按成熟控台的逐轴调整和翻转工作流，补相对草稿／独立轴保持／粗细精度、同射线另一姿态、整组原子拒绝与历史。状态归工程场景，物理映射／求解归 Rust，界面为位置属性；原生两灯及保存重开通过。实测参考点／自动拟合、实例输出补偿和操作反向按 ADR-085 独立建设，不能把手工零偏称为自动校准。

2026-10-02 CAP-02／03 补充：[ADR-088](development/decisions/PRODUCT-ADR-088-custom-wheel-appearances.md) 已对照 MA／Titan／GDTF，明确色盘实测、外观与物理数据分离、变体隔离、受控试灯、功能依赖与更新影响审阅；FIXTURE-007 先实施档位外观／批量建档和显式应用，其余继续作为未完成项。

2026-10-02 CAP-02／CAP-05／云分发补充：[ADR-090](development/decisions/PRODUCT-ADR-090-reusable-effect-library.md) 将灯具映射、灯效模板、工程实例和目标编译分层，先本地模板闭环再接云目录。类型化接口为设计稿；星空／激光／多单元 LED／新说明书按能力扩展，不能按外形或名称声称兼容。验收覆盖跨模式绑定、缺能力拒绝、离线固定版本和更新审阅，详见决策。

2026-10-02 CAP-04／CAP-05／CAP-13 增量：[LIBRARY-002](development/tasks/LIBRARY-002-local-intensity-templates.md) 已完成基础亮度曲线的本地模板闭环。Rust 管配方／绑定／来源验证，存储管有界文件及原子写入，动态效果区提供导入／审阅／导出；两个独立工程 80／3 台绑定、静态属性保持、取消／冲突拒绝、历史与离线重开通过。依赖既有 dimmer 能力、效果编译与工程事务；云目录、其他曲线与跨能力颜色角色继续按独立工单推进。

2026-10-02 CAP-04／CAP-05／CAP-13 接续：[LIBRARY-003](development/tasks/LIBRARY-003-keyframe-templates.md) 已支持既有单亮度关键帧效果的格式 2 模板，复用原求值器和文件工作流；只读曲线、精确帧值、同名导出区分、原生取消／历史／保存重开通过。新旧模板来源可并存，云端目录仍未实施。

2026-10-02 时间线剪辑增量：[AUDIO-018](development/tasks/AUDIO-018-preserved-entry-fades.md) 已支持渐变内分割、完整内部截取和显式重新计算；历史连续值属于工程片段，渐变与效果源时间分别由 Rust Plan 求值，界面只给编辑反馈。参考 OTIO 源时间／父时间分离，复用现有混合求值和事务，依赖稳定灯具属性身份；逐采样逻辑值／DMX 帧、容量／引用拒绝及原生取消／历史／保存重开通过。多轨、双动态交叉及设备承接继续独立推进。

2026-10-02 三维布置增量：[STAGE-005](development/tasks/STAGE-005-shared-3d-fixture-movement.md) 选择由工作区持有、位移由 Rust 原子校验、UE 只保留可取消提案；复用成熟编辑器的有序选组与活动对象、整组聚焦机制。正式场地／三维入口支持水平及升降，同高差两灯、锁定／取消／一次历史及保存重开通过；未新增设备协议或第二场地。完整旋转／缩放、混合构件和框选继续独立推进。
