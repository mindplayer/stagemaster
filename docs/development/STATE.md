# 当前开发状态

更新：2026-09-28。当前 Astra 会话直接负责架构、实现、测试、审查、集成和状态维护；不再委派 Sol／Qwen。
依据：[DEV-ADR-002](decisions/DEV-ADR-002-astra-direct.md)、[开发方法](README.md)、[当前执行计划](execution-plan.md)。文件统一留在本项目内，见[目录规则](project-files.md)。

已完成 [RECOVERY-001](tasks/RECOVERY-001-project-recovery.md)：基线 `cd7e0e9`，结果为本次 `feat(recovery): add leased project checkpoints and desktop recovery center` 提交。按 [ADR-027](decisions/PRODUCT-ADR-027-project-recovery.md) 增加独立 Rust 检查点、OS 会话租约、中文恢复中心、失败重试、明确丢弃和恢复为副本；不改工程格式。186 Rust／68 UI、最终定向回归、类型／fmt／严格 Clippy、桌面构建与双实例真实 SIGKILL 恢复、原文件摘要、取消／清理、损坏／较早记录、写入失败及窄窗口／Escape 通过。只保护已应用编辑，不含输入草稿／播放状态；原用户 14610／14928 保留。见[接口](../module-api/project-recovery.md)和[首版交付审查](tasks/RECOVERY-001-delivery-audit.md)；下一软件增量为 PLAYER-003A 主机有界执行包，板级 DMX／安装／授权仍须独立验证。

已完成 [CHECK-001](tasks/CHECK-001-project-check.md)：基线 `3c322e6`，结果为本次 `feat(check): add read-only project diagnostics and repair navigation` 提交。按 [ADR-026](decisions/PRODUCT-ADR-026-project-check.md) 接入独立 Rust 检查、真实配适／编译／计划统计、桌面问题定位、旧报告保护、搜索／分页／取消；电脑预览与未开放的设备发布分开。172 Rust／68 UI、后续定向回归、类型／fmt／严格 Clippy／桌面构建与原生修复、1025→1024 步、保存重开、并发编辑／取消、窄窗口通过；2,000 场景检查从 7.779 秒降至 0.428 秒。见[接口](../module-api/project-check.md)。原未保存窗口仍保留。下一项工程恢复，再推进设备执行包／安装。

用户要求使用持续目标模式改进界面，并强调吸收成熟经验。UX-009／010 已完成当前可操作界面设计的两轮迭代与目标验收，见 [UX-010 完成审查](tasks/UX-010-editing-recovery.md)；后续按具体反馈和真实产品接入推进。复用优先已写入根开发规则，不重启大范围选型或把设计夹具接进正式业务。

已完成 [UX-015](tasks/UX-015-editing-monitor.md)：基线 `62ba6f1`，结果为本次 `feat(ui): integrate scene monitoring and persistent workspace surfaces` 提交。编排加入三维同屏、当前场景跟随与显式动态播放监看；选灯、草稿和历史仍共用原核心。WorkspaceSurface 保留 React 状态并移出不可见 DOM，原生 20 次切页均恢复完整辅助树；源切换、错误保护、三维选灯、保存重开、窄窗口与同一 UE 进程复用通过。66 UI、类型、桌面构建和真实文件检查通过；不等同于完整 VoiceOver 或跨平台验收。原用户未保存窗口仍保留。

已完成 [POSITION-001](tasks/POSITION-001-moving-head-workflow.md)：基线 `0d2b47d`，结果为本次 `feat(position): integrate moving-head aiming and articulated preview` 提交。按 [ADR-025](decisions/PRODUCT-ADR-025-moving-head-workflow.md) 接入两轴档案范围／反向、手工单灯零偏、场景角度／默认位置／共同世界点静态对焦、释放／清除和运动定义换灯保护。Rust 求解与 DMX 量化共用，UE 协议 2 显示独立底座／支架／灯头；修复实测旋转浮点分量越 1 的误拒绝。165 Rust／66 UI／78 格式／4 UE、类型／fmt／严格 Clippy／双端构建通过；原生批量原子历史、错误恢复、保存重开、建档与内嵌姿态／暂停恢复已验收。见[运行接口](../module-api/positioning.md)。仍无持续目标跟随、真实光学／非相交轴／轮盘或现场 DMX；原用户未保存窗口保留，辅助树反复切换问题仍需专项修复。

已完成 [FIXTURE-002](tasks/FIXTURE-002-profiles-patch.md)：基线 `a92e5cb`，结果为本次 `feat(fixtures): add profile authoring and safe batch patch workflows` 提交。主动审查后优先补真实灯具接入：工程灯库、8/16 位粗细通道／默认值、使用中模式保护、保留编排与灯位的显式换灯、有序批量配适、占用图与可用地址建议。独立 Rust fixture 模块复用格式／编码／原子历史，见[接口](../module-api/fixture-authoring.md)。160 Rust／62 UI／73 格式、类型／fmt／严格 Clippy／桌面构建与原生错误恢复、撤销、保存重开、内嵌 UE 动态预演通过。当前仅调光／完整 RGB 线性建档；复杂摇头灯、播放盒 DMX、恢复与开演检查按[主动审查](tasks/FIXTURE-002-workflow-audit.md)推进，未将规划当实现。原用户未保存窗口保持运行。

已完成 [STAGE-001](tasks/STAGE-001-rigging-workflow.md)：基线 `9021f7c`，结果为本次 `feat(stage): integrate rigging assembly with embedded venue preview` 提交。按 [ADR-023](decisions/PRODUCT-ADR-023-rigging-assembly.md) 增加水平直桁架／灯杆、批量挂灯、关联平移／旋转／升降、解除和删除保护、按支撑体选灯、平面测距；Rust 管理实体关联与原子历史，UI 复用选灯和尺寸组件。房间、平台、支撑体、挂灯与光束已在内嵌 UE 一起显示，旋转实时同步。153 Rust／56 UI／73 格式、类型／fmt／严格 Clippy／桌面构建及原生保存重开通过。正式用户未保存窗口未操作。复杂吊点／结构载重、门洞、观众区、三维直接操作支撑体和规模性能后续。

已完成 [UX-014](tasks/UX-014-fixture-arrangement.md)：基线 `ec7c434`，结果为本次 `feat(stage): add batch fixture arrangement and group editing` 提交。正式舞台接入有序多选／框选、选择与移动工具、整组拖动、直线／矩阵／圆弧、平移旋转、对齐分布、可选底座朝向和草稿预览；复用 Rust 原子批次／保存，工程格式不变。54 项 UI、8 项 Rust 场地保护测试、类型和桌面构建通过；四灯矩阵、取消／错误定位、隐藏选择、组拖动／一次历史、键盘微调、旋转、保存重开及圆形内嵌三维通过。原用户未保存窗口未操作。桁架挂接已由 STAGE-001 完成；完整三维组变换、门洞与观众区后续；原生弹窗辅助树仍需专项处理。

已完成 [EFFECT-002](tasks/EFFECT-002-keyframes-reuse.md)：基线 `3a33ff5`，结果为本次 `feat(effects): add keyframe editing and cross-scene reuse` 提交。按 [ADR-022](decisions/PRODUCT-ADR-022-effect-keyframes.md) 新增 2–32 帧的亮度／RGB 循环、逐段保持／线性／平滑、精确位置／增删重排／均分、原效果转换、周期半速／倍速、跨场景独立复用与换灯范围。选帧条＋单帧属性保持面板紧凑，错误自动选帧并定位；Rust 是唯一动态求值源。145 Rust／46 UI／68 格式、类型／fmt／严格 Clippy／桌面构建通过；原生错序恢复、一次历史、跨场景换灯复用、保存重开与四灯内嵌三维已验收。正式用户未保存窗口未重启；现场速度主控／节拍、相对层、运动与设备效果仍后续。

已完成 [EFFECT-001](tasks/EFFECT-001-basic-effects.md)：基线 `a88c0b5`，结果为本次 `feat(effects): add scene lighting effects and shared preview` 提交。参考 MA／Titan，正式编排接入亮度呼吸、按灯序追逐和双色循环，含周期／相位／反向／范围、灯序编辑、启停／复制／删除、统一撤销和保存。按 [ADR-021](decisions/PRODUCT-ADR-021-lighting-effects.md) 新增能力声明与场景效果，Rust 统一求值；单场景与列表共用播放器，内嵌 UE 消费同一输出。137 Rust／41 UI／63 格式检查、类型／fmt／严格 Clippy与桌面构建通过；四灯原生编辑、冲突恢复、暂停、历史、保存重开与动态三维通过。正式用户窗口仍有未保存编辑，保持运行，更新后的构建需保存后重开。自由关键帧／主控、运动校准与设备效果预算后续；辅助树切换问题仍待专项处理。

已完成 [UX-013](tasks/UX-013-modeling-workflow.md)：基线 `8d49ada`，结果为本次 `feat(stage): streamline modeling workflow and add cutaway view` 提交。正式建模增加矩形／L 形尺寸创建、整体宽深与位置编辑、实时草稿轮廓、标注与八向缩放手柄、键盘微调、房间层级／搜索／折叠和聚焦所选；高级顶点折叠。按 [ADR-020](decisions/PRODUCT-ADR-020-modeling-view-controls.md) 拆分可重建围护投影，三维剖视保留地板／舞台，工程格式与输出语义未改。原生创建、错误恢复、尺寸拖动／单次撤销重做、连续键盘微调、独立保存重开及剖视实测通过；122 Rust／38 UI／4 UE、类型／fmt／严格 Clippy、双端构建通过。用户原窗口存在未保存编辑，本轮以独立验收实例完成，当前正式构建已更新但未强制重启原窗口。HTML 创建弹窗的原生自动化辅助树仍缺失，读屏验收待补；门洞／共享墙／观众区域、物理光学与性能仍属后续。

已完成首个可运行闭环：[PREVIS-001](tasks/PREVIS-001-real-stage-preview.md)。原始基线 `9d49939`，内部视窗提交 `e50cfbd`，本轮基线 `e50cfbd`，结果为本次 `feat(previs): integrate embedded placement editing and playback verification` 提交。Xcode 26.1.1／Metal 17B54／UE 5.8.3 已工作；首个真实固定调光／RGB 预演目标完成，不等同于完整专业预演或客户安装包交付。

“舞台 → 三维预演”在 Tauri 内显示真实 UE 画面，后台离屏运行。房间／舞台／灯位保存、双向选灯与属性、相机导航、工作照明、水平拖动、一次撤销／重做、重开和播放源同步已原生验证。按 [ADR-019](decisions/PRODUCT-ADR-019-embedded-previsualization.md)，UE 只提案，当前窗口队列先处理草稿，Rust 校验精确版本并写入统一历史；没有长期开放 HTTP 编辑资格。修复官方输入的松手端点丢失、越界哨兵误作有效坐标；快速拖动和拖出取消原生复测通过。无效 X 草稿时三维选灯保留原草稿并定位字段。

122 项 Rust、33 项 UI、4 项 UE 自动测试、类型／fmt／严格 Clippy及桌面／UE 构建通过；信令层沿用已通过的 3 项连接测试，本轮未改变。原生 60 秒列表渐变在 14.445 秒暂停、两灯 24% 保持，继续后增亮，停止后归零；渲染器故障隔离沿用 `e50cfbd` 的实测。官方 11 类蓝图、4 种材质、3 个网格加载验证及固定灯模型接入保留。未输出真实 DMX。

后续：光学／色彩定标、帧率／延迟与大规模场景测量、UE／信令运行时独立打包、完整摇头灯／图案盘能力。历史原生自动化切页后子树缺失已在 UX-015 通过显示容器调整及 20 次切页复测；完整读屏与跨平台仍后续。按住拖动时 Esc 未由原生自动化复现，不能借拖出取消通过替代此项。

已完成本轮：[DESKTOP-004](tasks/DESKTOP-004-groups-presets.md)。基线 `8513eee`；结果为本次 `feat(desktop): add ordered groups and reusable preset workflows` 提交。正式工作台已接有序灯组、预设池、引用／独立值、三种更新策略、依赖保护和属性复制；独立 Rust 资源模块复用原子历史／持久化／编译，见[接口](../module-api/editing-library.md)。86 项 Rust、28 项 UI、50 项格式、4 项 Sites、严格 Clippy／fmt／桌面构建与原生保存重开验收通过。逐灯预设增量完成，动态效果、完整现场编程器和通用共享预设仍待实施。

已完成：[PLAYER-002A](tasks/PLAYER-002-esp32-probe.md)。基线 `6fce3db`；用户授权实板测试，补充 GATT 直连／连接层抽象及保活，并明确不保留原固件。共享播放内核已在 ESP32-S3 执行，电脑 BLE 读写／通知、心跳超时和三次重连通过；GPIO21 保持低电平，无 DMX 输出。父任务 PLAYER-002 继续。接口见[设备连接诊断](../module-api/device-link-probe.md)。

已完成：[DESKTOP-003 / PLAYER-001A](tasks/DESKTOP-003-sequence-preview.md)。用户要求深入 goal 并再次强调模块化；本轮基线 `af3e83f`，结果为本次列表与离线预览提交。独立 Rust 执行器、工程列表编辑／编译和原生预览闭环已接通，接口见[运行模块契约](../module-api/sequence-preview.md)。

已完成：[DESKTOP-002](tasks/DESKTOP-002-editor-workflow.md)，按 ADR-012 接通真实灯具／编排工作区、批量配适、多灯属性／RGB、搜索选择与场景复制；基线 `0cce099`，结果为本次工作台提交。

## 产品基线

- EFFECT-001／002：[场景动态效果](../module-api/lighting-effects.md) 属于独立工程模块，UI 只生成参数；播放器用无逐帧分配的整数曲线，场景／列表和三维一致。效果灯序冻结，启用效果不得争用同一属性；旧静态工程兼容，新效果需 capability。已支持亮度／RGB 绝对曲线、最多 32 个循环关键帧及独立复用，尚无现场速度主控、设备效果包或实灯输出。

- DESKTOP-004：灯组有序成员、召回替换／追加／扣除、奇偶／反选、复制／编辑；预设按属性记录、引用或独立值应用、更新／合并／替换、引用场景与列表查看、解除引用和保留数值删除。选灯和查询归组件，真实语义归 Rust；不引入新语言或运行依赖。原生八灯、两场景联动、独立场景不随更新、撤销重做和重开引用完整通过。

- DESKTOP-003 按 ADR-013 新增列表创建／复制、步骤插入／重排／复制／删除、精确时间和引用保护；纯 Rust 执行器支持延时、渐变、自动等待、暂停／继续、跳转、循环及默认值释放。预览与编辑版本／历史分离，复用 DMX 编码，支持灯值和分页 512 通道监看。70 项 Rust、25 项 UI、50 项格式、4 项 Sites、类型／构建／fmt／严格 Clippy 通过；原生操作与真实文件复核通过。没有真实输出、3D、时间线或设备包；PROJECT-001／PLAYER-001 父任务保持进行中。

- DESKTOP-002 将组件化原则接入正式工作台：多选共同属性、混合值、颜色、批量配适、场景复制、工作区上下文、中文校验与恢复；Rust 批次一次提交／撤销。48 项 Rust、21 项 UI、50 项格式、4 项 Sites 检查通过，原生保存重开与错误恢复通过。完整边界见工单；无空间／播放／设备／AI 服务接入。

- 用户提出 AI 辅助灯光编辑。[AI-001](tasks/AI-001-assisted-editing-api.md) 按 [ADR-011](decisions/PRODUCT-ADR-011-assisted-editing.md) 新增[编辑自动化伪 API](../module-api/assisted-editing.md)：复用 Rust 工程命令，限定上下文与编辑范围，提案／差异／应用／撤销分开；可预授权范围内连续编辑，编辑权不包含现场控制。仅接口草案，未接模型或服务；能力总表增至 21 项。
- 用户要求主动补齐优秀软件的重要、必要能力，不只等待逐项提出。[PLAN-001](tasks/PLAN-001-capability-adoption.md) 将已有研究整理为[20 个能力方向与落地规则](../product-capability-plan.md)：模块／界面归属、真实状态、高返工边界、分阶段验收统一导航；仅规划，不更改公共格式或把所有功能前置到首版。当前可见编辑和软件＋单路播放盒主线继续有效。
- 既有 A0 多源混合器保持静态；新增独立的单列表播放模块按 ADR-013 执行；DESKTOP-001 已新增 Tauri 桌面工作台，接独立 Rust 工程／存储模块，完成最小灯光编辑与保存重开。原时间线交互代码保留待接入，尚无设备输出；详见[实现状态](../implementation-status.md)。
- 用户要求通过组件保持清晰，并接近现代创作软件的操作体验。UX-009 已交付[可交互组件工作台](../ui-design/component-workspace-design.md)：舞台与时间线为主，属性按需展开，布置／灯具／编排往返保留上下文。UX-008 三张静态提案保留参考，未认定用户选中某图；正式入口未替换。UX-006 的[共同指向](../ui-design/workspace-framework.md)与编辑规则继续有效；UE 保留独立后端。依据 [ADR-007](decisions/PRODUCT-ADR-007-effect-editing-first.md)。
- UX-007 已补[舞台与观众区创建方法](../ui-design/venue-layout-design.md)：借鉴 Vectorworks、SketchUp 与 Depence，以平面轮廓／尺寸生成三维舞台和观众区，过道自动避让；用户已认可方向并要求记录，尚无产品建模或座位生成实现。
- UX-011 按用户补充修订为[可组合空间](decisions/PRODUCT-ADR-010-composable-spaces.md)，UX-012 已在独立组件稿验证三空间选择、矩形／L 形尺寸、独立标高／净高、边界拖动与房间／全场查看。正式空间契约、房间建模、共享墙／门洞及工程保存仍未实施；设计稿不能代替真实空间编辑器。
- FIXTURE-001 已整理[灯具定义与个人灯库](../ui-design/fixture-definition-design.md)：参考 MA3／Titan／GDTF／OFL，分开可复用能力、硬件变体／模式／档案修订与工程实例，设计功能分段、色盘／图案盘和受控测试。依据 [ADR-009](decisions/PRODUCT-ADR-009-fixture-definition.md)；FIXTURE-002 已实现调光／RGB 线性建档、显式换灯和配适；完整个人库、功能分段、运动和受控实灯测试仍待后续。
- 用户最新澄清当前没有 iPad，仍按 MacBook 设计、开发和验收；未来 iPad 主力定位只要求提前准备。[ADR-008](decisions/PRODUCT-ADR-008-ipad-primary-authoring.md) 已撤回平板优先布局和前置移动验证，只保留共享核心、输入／布局／宿主与渲染适配边界；无待补设备型号问题。
- 用户确认首版交付软件＋独立播放盒：现有微雪 ESP32-S3-RS485-CAN，1 路 DMX；继续要求无电脑选场景／执行。范围见 [PRODUCT-ADR-001](decisions/PRODUCT-ADR-001-first-software-hardware-delivery.md)；PLAYER-002A 已有 Rust 诊断固件及 GATT 实板通信，真实 DMX 输出和节目持久安装尚未实现。
- 清理前主线：`121efa311855d977364f7ad8707ea729b5c0e367`；仅一个 `main` 工作区，无待合并分支、标签或远程。
- CORE-001 已修复场景编号碰撞，集成 `df64f98603ca28462cf76a515b65fb39dda9b26d`。
- CORE-002 已修复 HTP 默认值下限，集成 `81ed816fff5d8a358d5e1ecf933057a148d02e8f`。
- G0 按开发基础范围结项；已停用工作器的三项残留缺陷没有被认定为修复通过。

## 任务状态

| 任务 | 状态 | 说明 |
| --- | --- | --- |
| [CHECK-001](tasks/CHECK-001-project-check.md) | done（工程检查闭环） | 只读编译与容量报告、错误定位／修复／重查、版本与取消保护、保存重开；设备包及安装尚未实现 |
| [POSITION-001](tasks/POSITION-001-moving-head-workflow.md) | done（两轴位置闭环） | 档案／零偏、静态共同对焦、原子历史、量化编码、UE 独立关节、保存重开与播放暂停恢复通过；空间轨迹／复杂关节／实灯后续 |
| [FIXTURE-002](tasks/FIXTURE-002-profiles-patch.md) | done（灯库／配适） | 三种线性属性组合、粗细映射、模式保护／换灯、批量改址与占用、原子历史／保存重开、内嵌动态预演通过；复杂档案与实灯后续 |
| [STAGE-001](tasks/STAGE-001-rigging-workflow.md) | done（场地装配） | 六项工作流、统一历史／保存、内嵌 UE 场地与光束同步通过；复杂吊点、承载计算与三维直接组操作后续 |
| [UX-014](tasks/UX-014-fixture-arrangement.md) | done（灯位布置） | 多选／组拖动、五类排列调整、灯序／搜索、一次历史与保存重开通过；三维接已应用灯位，直接拖动仍单灯 |
| [EFFECT-002](tasks/EFFECT-002-keyframes-reuse.md) | done（关键帧／复用） | 最多 32 帧、三种过渡、错误选帧定位、跨场景独立复用、保存历史与三维通过；现场速度主控及运动后续 |
| [EFFECT-001](tasks/EFFECT-001-basic-effects.md) | done（基础效果） | 呼吸／追逐／双色、灯序／相位、原子历史／保存、Rust 播放与嵌入三维通过；关键帧已由 EFFECT-002 补齐，主控、运动与设备效果预算后续 |
| [UX-013](tasks/UX-013-modeling-workflow.md) | done（交互增量） | 正式尺寸建模、对象层级、缩放手柄／历史／保存重开与三维剖视通过；完整 CAD／读屏／专业光学后续 |
| [PREVIS-001](tasks/PREVIS-001-real-stage-preview.md) | done（首个闭环） | 应用内真实 UE、双向选择／拖动／历史／保存与列表播放联动已验收；专业光学、规模性能和客户独立打包后续推进 |
| [DEV-001](tasks/DEV-001-delivery-foundation.md) | done | Git 与验证基线保留 |
| [CORE-001](tasks/CORE-001-cue-uniqueness.md) | done | 实现与保护验收保留 |
| [CORE-002](tasks/CORE-002-htp-fallback.md) | done | 实现与保护验收保留 |
| DEV-002／003／004 | retired / 历史结项 / cancelled | 旧工作器路线退出，专属文件转由 Git 历史追溯，不重启补修 |
| [DEV-005](tasks/DEV-005-current-version-cleanup.md) | done | 当前项目清理完成；源码重建、24 项测试、fmt 与严格 Clippy 通过 |
| [UX-001](../ui-design/interaction-display-research.md) | 研究讨论稿已整理 | 7 个专业软件／工业 HMI 参考对象；原则用于 UX-003，尚无用户可用性实测 |
| [UX-002](../ui-design/visual-directions.md) | 用户已选第一张 | 选择舞台画布；补充剪映式编辑建议后生成时间线修订图，作为 UX-003 基准 |
| [UX-003](tasks/UX-003-stage-canvas-prototype.md) | done | 本机交互原型：片段移动／裁切、参数、预览、音频参考、保存和独立执行模拟；10 项测试、类型、构建与视觉验收通过 |
| [UX-004](tasks/UX-004-timeline-drag.md) | done | 修复时间尺不能连续拖动与片段半秒跳格；新增可拖播放头、邻近吸附、每帧合并、取消与边缘滚动；15 项测试及浏览器复查通过 |
| [UX-005](tasks/UX-005-chinese-terminology.md) | done | 统一“场景／执行”等中文操作术语，同步界面、辅助功能标签及当前说明；类型检查、构建与浏览器保存／执行核验通过 |
| [HW-001](tasks/HW-001-first-player-baseline.md) | done（范围／资料） | 已核对官方板卡资料与原理图，记录首版边界和实施顺序；发现隔离侧公共地需补引出方案，无刷机／接灯 |
| [HW-002](tasks/HW-002-project-segments.md) | done（研究／取舍） | 核对控台工程内容，确定主机编译与设备播放包边界；比较传输方案，优先有线验证、局域网传包，蓝牙保留配网／控制候选；只读识别到乐鑫 USB 接口 |
| [PROJECT-001A](tasks/PROJECT-001A-format-design.md) | done（格式设计） | 声光电与机构工程 `0.1.0-draft.1`、4 份 Schema、5 份样例、49 项开发期测试；未接入产品运行 |
| [DESKTOP-001](tasks/DESKTOP-001-visible-workbench.md) | done | Tauri 工作台、最小灯光工程编辑／保存／重开、撤销及退出保护；无样例入口或模拟执行 |
| [DESKTOP-002](tasks/DESKTOP-002-editor-workflow.md) | done | 组件式真实编排、多灯／RGB、批量配适、复制、搜索与恢复；原子事务、保存重开和原生验收通过 |
| [DESKTOP-004](tasks/DESKTOP-004-groups-presets.md) | done（本轮增量） | 灯组与预设真实编辑、依赖／更新策略、属性复制、保存重开和预览核对；后续效果／编程器独立迭代 |
| [DESKTOP-003 / PLAYER-001A](tasks/DESKTOP-003-sequence-preview.md) | done（本轮增量） | 真实场景列表、独立编译／时间执行、离线预览、DMX 数值监看、上下文与恢复；无设备接入 |
| [UX-006](tasks/UX-006-effect-editor-design.md) | done（设计交付） | 总体工作区、效果编辑、共同指向可拖设计稿与 UE 边界；已验证理想指向交互，未实施产品业务／真实求解 |
| [UX-007](tasks/UX-007-venue-layout-design.md) | done（研究／设计） | 舞台、观众区、过道创建及二维／三维联动；已核对成熟软件资料，尚无产品实现 |
| [FIXTURE-001](tasks/FIXTURE-001-definition-design.md) | done（研究／设计） | 复用灯具能力、自定义档案、模式／变体／版本、建档交互与核心扩展边界；尚无编辑器、导入器或实灯测试 |
| [UX-008](tasks/UX-008-modular-workspace-design.md) | done（静态提案） | 功能落位与三张工作区布局；后续组件化交互由 UX-009 继续，未认定选中某图 |
| [UX-009](tasks/UX-009-component-workspace.md) | done（独立交互设计） | 按需属性组件、工作区往返、指向拖动和灯具草稿子集验证；未替换正式 UI 或接入业务 |
| [UX-010](tasks/UX-010-editing-recovery.md) | done（交互改进／目标验收） | 输入草稿／错误定位、分范围具名撤销、连续点击和快捷键修复；两轮可操作界面目标验收通过，正式产品接入仍待实施 |
| [UX-011](tasks/UX-011-multi-space-layout.md) | done（需求／架构方向） | 异形室内、多空间、独立标高／净高、共享构件与连接、房间查看；更新场地闭环顺序，契约与产品实现待办 |
| [UX-012](tasks/UX-012-space-workspace.md) | done（独立交互优化） | 多空间布置、参数编辑、平面边界拖动／取消／撤销、立体和当前空间查看；已验证，未接正式工程 |
| [PLAN-001](tasks/PLAN-001-capability-adoption.md) | done（能力整合与审查） | 20 个能力方向、7 类闭环缺口、8 项高返工边界；主动吸收规则与产品蓝图对齐，不改变首版门槛 |
| [AI-001](tasks/AI-001-assisted-editing-api.md) | done（伪接口设计） | 受限编辑代理、宿主授权、提案／应用／撤销与冲突对账；11 个 TS 输入及 11 个新增错误调用反例检查通过，无模型或服务实现 |
| [PROJECT-001](tasks/PROJECT-001-project-contract.md) | in progress | 灯光编辑子集读取／领域校验及保存重开已随 DESKTOP-001 完成；场景列表及单路编译已由 DESKTOP-003 补齐；时间线、迁移恢复和其他领域待实施 |
| [PLAYER-001](tasks/PLAYER-001-software-playback-foundation.md) | in progress | PLAYER-001A 已实现独立有界执行、正式工程编译和 512 通道预览；设备预算实测、片段包、模拟发送确认／故障与完整出口待办 |
| [PLAYER-002](tasks/PLAYER-002-esp32-probe.md) | in progress；002A done | 工具链／共享内核与 BLE GATT 实板验证通过；UART DMX、时序／电气仍待办 |
| PLAYER-003–005 | planned | 持久包与传输无关安装／控制、GATT 优先接入、UI／本地操作及整机验收；未实施 |
| AUTH-001 | planned | 手机／电脑中转正式授权，离线时当前文件可生成 24 小时临时包；到期收尾后禁止新播放。商业验收前须补服务、离线签发、可信时间和生产保护；依据 [PRODUCT-ADR-004](decisions/PRODUCT-ADR-004-relayed-device-authorization.md) |

## 最新验证

FIXTURE-002：160 Rust、62 UI、73 格式、类型／fmt／严格 Clippy和桌面构建通过。原生自定义六通道／16 位反序映射、重复通道定位、换灯冲突整批拒绝、超 512 修正、反向灯序、隐藏选择、取消、一次撤销重做、保存重开、占用图定位和无效线路、无效草稿拦截导航／Esc 恢复通过。真实文件确认全部场景效果、灯组／预设／列表与场地／挂接保留，内嵌 UE 静态及动态灯光通过。证据 `data/FIXTURE-002/`，日志 `logs/FIXTURE-002-*.log`；用户原窗口未操作，无设备输出。具体边界见工单。

STAGE-001：153 Rust、56 UI、73 格式、类型／fmt／严格 Clippy与桌面构建通过。原生四灯反序均布、余量错误保留定位、一次撤销重做、90° 旋转／升高、整体拖动／撤销、解除／删除保持灯位及恢复、复制不复制挂灯、4 米平面测距、重开后测距方向键无修改均通过。最终构建重开和 UE 内房间／舞台／桁架／灯杆／四灯光束同场、剖视、30° 旋转同步通过；格式检查和文件对比确认灯具／配适／场景／灯组及原场地未误改。证据 `data/STAGE-001/`、日志 `logs/STAGE-001-*.log`。读屏、按住拖动 Esc、触屏／其他系统／规模性能未补验，原用户未保存窗口保持。

UX-014：54 项 UI（新增 8 项排列／变换／选择边界）、8 项 Rust 场地保护测试、类型和桌面构建通过；无 Rust／Schema 修改，未重复全工作区及 UE 编译。原生四灯反序矩阵、非法列数保留定位、搜索隐藏选择、取消无历史、一次撤销恢复未布置状态、组拖动保镜头／灯序、框选、两次微调、90° 旋转和升高、保存重开、满圆四灯三维同步通过。真实文件核对配适、场景、列表、空间和底座角度未误改。证据在 `data/UX-014/`，日志 `logs/UX-014-*.log`；没有设备输出，用户原未保存工程未操作。读屏、按住拖动时 Esc、触屏与大规模性能未宣称通过。

EFFECT-002：145 Rust、46 UI、68 格式、类型／fmt／严格 Clippy和桌面构建通过。原生关键帧错序自动选中字段、均分恢复、增删重排、半速／倍速、一次撤销重做、跨场景搜索／单灯复用、保存重开及四灯关键帧三维变化通过。完整证据与边界见工单；验收数据／截图在 `data/EFFECT-002/`，日志在项目 `logs/`。无真实输出和固件变化，原用户未保存内容未操作。

EFFECT-001：137 项 Rust（新增 8 内核／5 工程／2 宿主）、41 项 UI、63 项格式检查、类型、fmt、严格 Clippy、桌面构建通过。实际四灯追逐、顺序重排、非法周期定位／修改、暂停保持、颜色叠加、属性冲突草稿保留、撤销重做、复制停用／删除、保存重开及应用内三维动态光束／落点通过。迟到帧与连续推进、延时／渐变、有限结束、预算拒绝和场景／列表切换有核心保护测试；原生列表动态过渡未单独复测。验收数据在 `data/EFFECT-001/`，日志在 `logs/EFFECT-001-*.log`；未操作正式窗口编辑、真实 DMX 或固件。原生辅助树切换缺失仍未解决，不能认定读屏通过。

DESKTOP-004：86 项 Rust（新增 8 项资源领域和 1 项 Session 原子历史／预览失效保护）、28 项 UI、50 项格式、4 项 Sites、类型、桌面打包、fmt 和严格 Clippy 通过。原生检验灯组反序／精确重排、空名定位、颜色掩码 24 项、两场景引用更新、取消不提交、解除后保持旧色、删除依赖错误／固化／撤销重做、来源灯亮度复制、搜索保留 7 台隐藏选择、窗口关闭编辑保护、复制后自动选中和保存重开。DMX 离线预览通道 1–4 为 204/16/160/224，对应 80% 亮度和 #10A0E0；无物理输出。约 1440×940 与 1100×800 窗口视觉检查通过。验收文件／日志保留项目内，退出验收工程后重新打开空白欢迎页。

PLAYER-002A：基线 `6fce3db`，结果为本次设备探针提交。实读 ESP32-S3 v0.2、16 MB Flash，刷入诊断固件；按用户要求停止并删除旧固件读取文件。Xtensa Rust 1.97.0.0、esp-hal 1.2.2 与依赖锁定；同份 no_std 播放内核实板自检通过，512 属性／10000 次推进用时 4,089,015 微秒，过程中堆使用不增加。GATT 实际协商 MTU 247，短写／错误版本／错误会话／重复／乱序拒绝、16 次有效心跳、约 5.913 秒无有效心跳断开、三次重连和断线内核持续推进通过；128 KiB 配置堆的已用／剩余为 41020／90052 字节，连接前后相同。77 项 Rust、主工作区及固件 fmt／严格 Clippy、独立固件离线构建通过。保留裸机 ELF RWX 段告警，esp-radio 依赖为 beta；没有 DMX、PSRAM、节目文件上传、身份授权、独立供电／拔 USB、手机后台或 8 小时验收。详细限制见工单。

DESKTOP-003 / PLAYER-001A：基线 `af3e83f`；结果为本次 `feat(playback): deliver modular sequence editing and offline preview` 提交。ADR-013 先于公共接口实现。70 项 Rust、25 项 UI、50 项格式和 4 项 Sites 检查通过，fmt／严格 Clippy／UI 类型与桌面构建通过。原生验证两步自动衔接、渐变中暂停 0.872 秒冻结、继续、末步控制边界、停止恢复默认值；DMX 实测页面数值 204/57/121/255 与 80% 和 RGB 目标一致。非法精度／编号冲突聚焦字段、取消、精确重排选中保持、复制／撤销／重做、保存不使预览过期、内容修改禁用陈旧执行、保存重开通过。通道分页及 509–512 范围通过。应用已更新并恢复原空白工程；所有验收文件与日志留项目内。没有真实总线输出、触屏／其他系统或板卡预算实测。

DESKTOP-002：基线 `0cce099`；结果为本次 `feat(desktop): expand real editing workspaces and batch workflows` 提交。ADR-012 先于公共命令扩展；48 项 Rust、fmt／严格 Clippy、UI 类型／21 项测试、50 项格式与 4 项 Sites、构建通过。原生八灯批量配适、成组撤销、颜色／混合值、复制不污染原场景、搜索选择保持、中文错误／取消、引用删除保护、滑块键盘焦点与真实保存重开已验收。数据／日志留项目内，最终应用恢复原工程；长拖性能与跨端验收未覆盖，没有操作设备或部署。

AI-001：基线 `ba613d8`；结果为本次 `docs(api): reserve scoped AI-assisted editing contracts` 提交。ADR-011 先审查再新增辅助编辑扩展，不修改主协议／工程格式。项目本地 TypeScript 7.0.2 对全部 11 个接口／样例文件检查通过，新增 11 个编译期反例有效；文档引用、围栏、能力编号及差异检查通过。仅设计资料，未实现服务、调用模型、发送工程数据、改正式界面或操作设备；运行权限、事务恢复与模型效果待后续验收。

PLAN-001：基线 `9638236`；结果为本次 `docs(product): consolidate capability adoption and delivery boundaries` 提交。复用已有 22 模块／304 条对照，补查 6 个官方页面，将重要能力落到模块、界面、实际状态与实施层次。修正旧蓝图中播放盒、首台硬件、格式与授权顺序的过时表述。文档引用／围栏、20 个能力编号、来源数量、研究计数与差异检查通过；纯文档，无公共契约、产品代码、设备或部署变化，未运行无关产品测试。

UX-012：基线 `0b08e01`；结果为本次 `feat(ui-design): add multi-space layout interactions` 提交。验证独立标高／净高、L 形参数、画布选房间、输入错误跨空间／工作区定位、刷新恢复；拖宽 6→7→单次撤销 6、拖深 6→7→Esc 6、方向键微调至 6.1。与基线同状态截图对照编排布局，回归颜色、片段／时间保持及灯具草稿独立撤销；多种窗口宽度检查无横向溢出。脚本语法、片段约束、文档引用与差异检查通过，浏览器无脚本错误。仅设计稿及说明，未改正式产品代码、Schema、硬件或部署。

UX-011：基线 `cf13e72`；结果为本次 `docs(spaces): plan irregular multi-room venues` 提交。核对 Vectorworks 2026 与 IFC4 ADD2 TC1 官方空间／边界机制，形成 ADR-010，修订场地创建和多房间验收范围。与现有坐标、目标附着、输出及端侧裁剪边界核对，本地文档链接与差异检查通过。仅文档变更，未修改 Schema／核心／交互稿，未运行产品测试或声称已实现建模、遮挡与性能验证。

UX-010：基线 `2c878ef`；结果为本次 `fix(ui-design): preserve invalid input and isolate undo` 提交。浏览器验证空值／越界保留与恢复、错误跨片段定位、独立历史与选择保持、快捷键重做、通道冲突／区间错误、紧接编辑后的点击；目标点 1.5→3.2→单次撤销 1.5、时间定位 20.8→取消 15.3，约 1440／1024／736／305 像素布局复查通过，无脚本错误。脚本语法、片段约束、本地引用和差异检查通过；未改产品代码／公共格式，无设备／部署操作。当前设计目标逐项审查通过，产品边界见工单。

UX-009：基线 `76f1598`；结果为本次 `docs(ui): add interactive component workspace` 提交。完成现代创作式可交互工作台，验证组件展开、颜色修改、工作区往返、片段／时间位置保持、通道与区间错误、草稿还原、目标拖动／取消／单次撤销和时间尺定位；补标准快捷键、展开语义及焦点保持，时间尺 Esc 取消复查通过。约 1024／736／305 像素宽度无横向溢出，修复窄窗口资源区无法展开。浏览器无脚本错误，脚本语法、本地引用和差异检查通过。仅独立设计资料，无正式产品代码／格式变更、真实保存／播放、UE 或设备测试；未运行无关产品测试。

UX-008 组件化反馈：基线 `3909d1f`；结果为本次 `docs(ui): record functional component composition principle` 提交。补记用户希望通过组件封装功能来保持页面清晰，细化功能组件按需展开、明确编辑目标、共享状态与关闭生命周期。文档引用与差异检查通过；无产品代码／公共契约变更，未运行产品测试，三张视觉提案仍待选型。

UX-008：基线 `d5890c6`；结果为本次 `docs(ui): explore scalable modular workspaces` 提交。保留已选视觉风格，生成三张不同信息结构的静态图；完整提示词、功能归属、上下文和限制均入库。逐张检查，PNG 尺寸／副本哈希、本地引用及差异检查通过；第三图九光源与八台标签不符已记录。未改产品代码／格式或运行产品测试，未启动服务、操作设备或部署。

FIXTURE-001：基线 `9c76803`；结果为本次 `docs(fixtures): define reusable capabilities and profile editing` 提交。核对 MA3 类型编辑／功能档位、Titan 19 个人档案／虚拟调光、GDTF 规范和 OFL 模式，并检查现有 Rust 映射／工程校验的不足；形成 ADR-009 和交互设计。补记用户确认 UX-007 方向。本地文档引用与差异检查通过；未修改代码、Schema 或公共 API，未运行产品测试、连接串口或输出。

UX-007：基线 `69189bf`；结果为本次 `docs(ui): design stage and audience layout workflow` 提交。核对 Vectorworks 2026 舞台／座位区／过道、SketchUp 推拉／标尺和 Depence 3 观众生成资料；设计“平面定义、三维同步”的创建流程及分步交付。与现有坐标、场地修订和共同指向边界核对，本地引用与差异检查通过；纯文档变更，未改产品代码、公共契约或格式，未运行建模／性能／硬件测试。

UX-006 总体框架补充：基线 `fe48904`；结果为本次 `docs(ui): define workspace and interactive group aiming` 提交。整体布局与“共同指向”操作收敛，分开普通拖动／轨迹记录、安装位置／指向目标和渲染／核心求解。独立设计稿经浏览器核验拖动中的八灯同点、固定安装点、单次撤销、取消、数值输入、视角与窄屏重排；修复状态回显清空撤销历史。脚本语法、本地引用及差异检查通过，无产品代码／格式变更、UE 接入或硬件测试。

UX-006 当前平台澄清：基线 `2c64d0f`；结果为本次 `docs: keep MacBook development primary and defer iPad work` 提交。修正将未来 iPad 目标提前为当前任务的误读，恢复 MacBook 布局、键鼠／触控板与效果编辑优先，移除前置移动宿主验收；未来适配边界保留。文档引用和差异检查通过，未改代码或运行产品测试，无当前 iPad 验收阻塞。

UX-006 iPad 定位补充：基线 `4f3cc4b`；结果为本次 `docs: prioritize iPad as primary authoring device` 提交。新增 ADR-008，修订平板角色、触控／Pencil、移动宿主验证顺序、独立播放盒和 UE 移动边界；核对 Apple、Tauri 与 Epic 官方资料。文档本地引用与差异检查通过，未改代码、公共契约或工程格式，未运行产品测试或安装／接入 iOS 与 UE。

UX-006 三维拖动补充：基线 `d8b94ba`；结果为本次 `docs(ui): specify direct 3D stage manipulation` 提交。补充镜头导航、触控板、物体约束拖动、精确摆位、撤销和多舞台观察规则；光束指向另依赖校准和核心求解。文档引用与差异检查通过；尚未实现或测试三维交互，不修改应用／引擎。

UX-006：基线 `9d81cc3`；结果为本次 `docs(ui): define effect-first workspace and UE stage integration` 提交。主界面围绕选灯／效果／时间编排，记录选择与修改范围、灯具顺序、连续拖动、跨端和任务验收；补 UE 空间搭建与渲染边界。三张静态图已检查并记录数值／光束等生成瑕疵，PNG 尺寸、副本哈希、本地引用和差异检查通过。只改设计资料、执行顺序和界面偏好；未改应用、Schema、公共接口或引擎，未运行产品测试、安装 UE 或操作设备。

DESKTOP-001：基线 `603f40f`；结果为本次 `feat(desktop): deliver real lighting project editing and persistence` 提交。43 项 Rust 测试、fmt 与严格 Clippy，UI 类型／15 项交互回归、50 项格式检查和 4 项 Sites 测试通过；本机 .app 原生窗口验收新建、灯具／场景属性、真实保存重开、地址冲突、撤销重做和未保存取消。修复无父窗口的原生提示无法正常显示，复测通过。Schema 允许空白工程零节目入口，发布语义未放宽。限制与详细记录见工单；未操作设备、固件或云端。


PROJECT-001A：基线 `fc14472`；结果为本次 `feat(project): define modular audiovisual and motion JSON format` 提交。三层文件覆盖灯光、外部媒体、机构／输入输出、时间线联动、监看、空间与面板；4 份 Schema、5 份设计样例及 49 项检查通过。样例素材、程序和授权为明确占位，不可部署。只读重新识别到乐鑫 `303A:1001`、`/dev/cu.usbmodem2101`；未打开串口、刷机或输出。未改 Rust／UI，未重复其测试；持久化、编译、动作运行、设备预算及授权实现仍待办。

设备授权方案：基线 `2f681ca`；结果为本次 `docs: define temporary relayed playback authorization` 提交。用户明确手机／电脑中转加密包，手机离线时当前文件可生成 24 小时临时包；提前提示，到期允许已开始的节目结束后禁止新播放。[PRODUCT-ADR-004](decisions/PRODUCT-ADR-004-relayed-device-authorization.md) 补充受限离线签发、首次设备接受起算、重复安装不延期、可信时间及有限收尾的设计约束。官方资料、本地引用及差异检查完成；没有实施授权、访问设备或修改播放行为。

PROJECT-001 加密封装预留：基线 `2628ea1`；结果为本次 `docs: reserve encrypted project container boundary` 提交。核对成熟文件加密资料，补充 JSON 正文与外层加密容器、版本、密钥和设备包的分工。本地文档引用及差异检查通过；仅文档变更，未引入加密依赖或实现读写功能。

PROJECT-001 研究与规划：基线 `9768f32`；结果为本次 `docs: define JSON project contract direction` 提交。核对官方格式／版本规范及当前代码，形成 [PRODUCT-ADR-003](decisions/PRODUCT-ADR-003-project-data-contract.md)；本地文档引用与差异检查通过。仅文档变更，未实现或测试工程读写、未测格式性能、未操作设备。

PLAYER-001 规划：基线 `e461112`；结果为本次 `docs: sequence software playback before hardware integration` 提交。细化软件先行、硬件反馈能力的职责与分步验收；仅文档变更，本地引用与差异检查通过，未运行产品测试或操作设备。播放实现仍为待办。

HW-002：基线 `f986737`；结果为本次 `docs: assess compiled playback and transfer options` 提交。核对官方资料、脚本计算存储算例、文档引用和差异检查通过；系统枚举到 `303A:1001` 与 `/dev/cu.usbmodem2101`，未打开串口、复位、读写固件或输出。未验证实际闪存分区、容量和无线性能；详见 [PRODUCT-ADR-002](decisions/PRODUCT-ADR-002-compiled-playback-and-transfer.md)。

UX-005：基线 `e10d532`；结果为本次 `fix(ui): unify Chinese product terminology` 提交。界面与当前产品说明统一中文，长期规范写入根目录及原型开发规则。类型检查、生产构建、文档引用和差异检查通过；浏览器核对保存／执行提示、场景列表及当前窗口排版。仅改文案，不新增或重跑内核行为测试；软硬件主线仍按 PLAYER-001 接续。

HW-001：基线 `8877846`；结果为本次 `docs: define first software and DMX player delivery` 提交。官方文档及原理图渲染核对、本次本地 Markdown 文件引用与差异空白检查完成；非代码变更，未重复运行 Rust／UI 测试。未安装工具链、访问串口、刷机或操作灯具；完整 DMX 标准正文、接口适配和实物时序仍待验证。

UX-004：基线 `92089f6`；结果为本次 `fix(ui): make timeline gestures continuous` 提交。类型检查、15 项测试和构建通过；浏览器核验连续定位、3 px 微移、释放位置、一次撤销、Esc 取消、3× 边缘滚动与现场状态隔离。原型输入保留毫秒精度，未改变 Rust 或正式时间契约；详见工单。

UX-003：基线 `58c6f676959c896f0d34dd1aca2606f858c25357`；结果为本次 `feat(ui): add stage canvas timeline prototype` 提交所含源码与记录。React 原型 `apps/ui-prototype/` 的类型检查、10 项测试和生产构建通过；浏览器完成编辑／保存／执行隔离、移动／裁切、音频参考及较小视口检查，详见[视觉验收](../../design-qa.md)和[使用说明](../ui-design/interaction-prototype.md)。Rust 与公共契约未改动，原型刷新即重置，尚无真实调度、持久化或设备输出。

DEV-005 清理结果已集成：`5a15e97a1ecedf0a4c821fe38936089c9f2dc2de`；后续状态记录提交不改变产品代码。

- 已删除 35 个受版本管理的旧工具／流程文件、846 个旧试验及迁移文件，清空旧构建缓存后只生成当前源码的验证产物。Git 历史保留。
- 产品源码、Cargo 清单、保护测试、模块 API 和控台研究与清理前基线一致。
- `cargo fmt --all -- --check`、`cargo test --workspace --locked --offline`（24 项）、严格 Clippy 及文档引用检查通过。
- 真实灯具、固件、云端及共享模型环境未操作。

## 下一步

按 [FIXTURE-002 主动审查](tasks/FIXTURE-002-workflow-audit.md) 继续，不等待用户逐项提醒。UX-014／STAGE-001 已完成灯位阵列、组编辑、水平支撑体／挂接和内嵌场地；FIXTURE-002 已补线性建档与换灯。下一软件核心依 ADR-009／016 接运动档案、两轴物理范围／反向／零位、单灯安装校准、共同目标和 UE 关节；随后补快门／轮盘等分段功能。现场编程器／速度主控需明确控制权和时间合成，再逐步推进真实时间线。场地后续为隐藏／锁定／空间隔离、门洞／共享墙与观众区；崩溃恢复、问题导航和开演检查列为商业必要增量，辅助功能问题专项补验。

首次软硬件交付主线仍需 UART DMX 发送适配／时序与完整隔离接口、版本化执行包、分块传输／原子安装及正式连接界面。PLAYER-002A 仅完成共享内核／GATT 实板增量；当前诊断固件不能播放用户工程或驱动灯具。GATT 先接业务，USB 用于开发恢复；新的动态效果另做设备资源与时序测量。

DESKTOP-002 已完成组件式真实工作台接入；继续依据 [UX-009](../ui-design/component-workspace-design.md) 收集操作反馈，并按成熟软件细节清单迭代；延续现代创作式布局与组件按需展开，不再以静态三图选型阻塞设计迭代。设计中的草稿、场地与片段是交互夹具，正式入口仍接真实工程与命令。UX-006／007／FIXTURE-001 的业务边界继续有效。

场地新增约束按 UX-011／ADR-010 执行：正式空间契约从开始覆盖多个异形房间、各自标高／净高与连接，先完成基础空间编辑和保存闭环，再加入舞台／观众区；不用当前稿的全局宽深代替正式模型。复杂坡顶、多层细节、完整建筑导入与 UE 继续分步实施。

授权规则已明确，AUTH-001 列为相关商业交付的前置验收项；离线许可、断电计时和有限收尾还需实现及验证，不阻塞下述工程标准工作，也不把云端授权作为当前开发原型的运行前提。

结合 [UX-006 总体框架](../ui-design/workspace-framework.md) 与 [UX-007 场地创建](../ui-design/venue-layout-design.md) 收集 MacBook 工作区、布置与目标点跟随的实际反馈，为真实界面接入制定小范围工单；先空间契约与基本搭建／摆位，再接校准与 Rust 指向求解，普通拖动不自动录制路径。[未来 iPad 准备](decisions/PRODUCT-ADR-008-ipad-primary-authoring.md) 不改变当前推进顺序，不等待设备或前置移动宿主测试。既有配适、场景编辑和保存继续复用；场景列表与 Rust 离线预览已由 DESKTOP-003 实现；内嵌 UE 已完成固定灯首个闭环，真实时间线与专业预演增量按相应依赖推进，不用假业务填满新布局。父任务 PROJECT-001 不结项；板卡风险与 AUTH 商业门槛仍保留，硬件按 PLAYER-002 的当前授权和已验证边界推进。详见[编排端迭代计划](desktop-iteration-plan.md)。

灯具完整能力继续沿 FIXTURE-001 设计分层；调光／RGB 工程内建档已由 FIXTURE-002 完成，下一步功能分段与常见摇头灯另立契约和可见编辑工单。真实复杂灯具、共同指向和输出接入以经过验证的档案为前提，不等待完整云端灯库或 UE；受控测试需先具备输出控制权和停止机制，当前不自动接灯。

旧独立项目 `yunwei-ma` 有未提交源码及未跟踪文件，删除范围尚待用户明确；当前不改动该仓库。
