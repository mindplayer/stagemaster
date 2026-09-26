# 当前开发状态

更新：2026-09-26。当前 Astra 会话直接负责架构、实现、测试、审查、集成和状态维护；不再委派 Sol／Qwen。
依据：[DEV-ADR-002](decisions/DEV-ADR-002-astra-direct.md)、[开发方法](README.md)、[当前执行计划](execution-plan.md)。文件统一留在本项目内，见[目录规则](project-files.md)。

用户要求使用持续目标模式改进界面，并强调吸收成熟经验。UX-009／010 已完成当前可操作界面设计的两轮迭代与目标验收，见 [UX-010 完成审查](tasks/UX-010-editing-recovery.md)；后续按具体反馈和真实产品接入推进。复用优先已写入根开发规则，不重启大范围选型或把设计夹具接进正式业务。

已完成：[DESKTOP-003 / PLAYER-001A](tasks/DESKTOP-003-sequence-preview.md)。用户要求深入 goal 并再次强调模块化；本轮基线 `af3e83f`，结果为本次列表与离线预览提交。独立 Rust 执行器、工程列表编辑／编译和原生预览闭环已接通，接口见[运行模块契约](../module-api/sequence-preview.md)。

已完成：[DESKTOP-002](tasks/DESKTOP-002-editor-workflow.md)，按 ADR-012 接通真实灯具／编排工作区、批量配适、多灯属性／RGB、搜索选择与场景复制；基线 `0cce099`，结果为本次工作台提交。

## 产品基线

- DESKTOP-003 按 ADR-013 新增列表创建／复制、步骤插入／重排／复制／删除、精确时间和引用保护；纯 Rust 执行器支持延时、渐变、自动等待、暂停／继续、跳转、循环及默认值释放。预览与编辑版本／历史分离，复用 DMX 编码，支持灯值和分页 512 通道监看。70 项 Rust、25 项 UI、50 项格式、4 项 Sites、类型／构建／fmt／严格 Clippy 通过；原生操作与真实文件复核通过。没有真实输出、3D、时间线或设备包；PROJECT-001／PLAYER-001 父任务保持进行中。

- DESKTOP-002 将组件化原则接入正式工作台：多选共同属性、混合值、颜色、批量配适、场景复制、工作区上下文、中文校验与恢复；Rust 批次一次提交／撤销。48 项 Rust、21 项 UI、50 项格式、4 项 Sites 检查通过，原生保存重开与错误恢复通过。完整边界见工单；无空间／播放／设备／AI 服务接入。

- 用户提出 AI 辅助灯光编辑。[AI-001](tasks/AI-001-assisted-editing-api.md) 按 [ADR-011](decisions/PRODUCT-ADR-011-assisted-editing.md) 新增[编辑自动化伪 API](../module-api/assisted-editing.md)：复用 Rust 工程命令，限定上下文与编辑范围，提案／差异／应用／撤销分开；可预授权范围内连续编辑，编辑权不包含现场控制。仅接口草案，未接模型或服务；能力总表增至 21 项。
- 用户要求主动补齐优秀软件的重要、必要能力，不只等待逐项提出。[PLAN-001](tasks/PLAN-001-capability-adoption.md) 将已有研究整理为[20 个能力方向与落地规则](../product-capability-plan.md)：模块／界面归属、真实状态、高返工边界、分阶段验收统一导航；仅规划，不更改公共格式或把所有功能前置到首版。当前可见编辑和软件＋单路播放盒主线继续有效。
- 既有 A0 多源混合器保持静态；新增独立的单列表播放模块按 ADR-013 执行；DESKTOP-001 已新增 Tauri 桌面工作台，接独立 Rust 工程／存储模块，完成最小灯光编辑与保存重开。原时间线交互代码保留待接入，尚无设备输出；详见[实现状态](../implementation-status.md)。
- 用户要求通过组件保持清晰，并接近现代创作软件的操作体验。UX-009 已交付[可交互组件工作台](../ui-design/component-workspace-design.md)：舞台与时间线为主，属性按需展开，布置／灯具／编排往返保留上下文。UX-008 三张静态提案保留参考，未认定用户选中某图；正式入口未替换。UX-006 的[共同指向](../ui-design/workspace-framework.md)与编辑规则继续有效；UE 保留独立后端。依据 [ADR-007](decisions/PRODUCT-ADR-007-effect-editing-first.md)。
- UX-007 已补[舞台与观众区创建方法](../ui-design/venue-layout-design.md)：借鉴 Vectorworks、SketchUp 与 Depence，以平面轮廓／尺寸生成三维舞台和观众区，过道自动避让；用户已认可方向并要求记录，尚无产品建模或座位生成实现。
- UX-011 按用户补充修订为[可组合空间](decisions/PRODUCT-ADR-010-composable-spaces.md)，UX-012 已在独立组件稿验证三空间选择、矩形／L 形尺寸、独立标高／净高、边界拖动与房间／全场查看。正式空间契约、房间建模、共享墙／门洞及工程保存仍未实施；设计稿不能代替真实空间编辑器。
- FIXTURE-001 已整理[灯具定义与个人灯库](../ui-design/fixture-definition-design.md)：参考 MA3／Titan／GDTF／OFL，分开可复用能力、硬件变体／模式／档案修订与工程实例，设计功能分段、色盘／图案盘和受控测试。依据 [ADR-009](decisions/PRODUCT-ADR-009-fixture-definition.md)；当前线性映射不足以承载全部语义，具体契约与编辑器未实施。
- 用户最新澄清当前没有 iPad，仍按 MacBook 设计、开发和验收；未来 iPad 主力定位只要求提前准备。[ADR-008](decisions/PRODUCT-ADR-008-ipad-primary-authoring.md) 已撤回平板优先布局和前置移动验证，只保留共享核心、输入／布局／宿主与渲染适配边界；无待补设备型号问题。
- 用户确认首版交付软件＋独立播放盒：现有微雪 ESP32-S3-RS485-CAN，1 路 DMX；继续要求无电脑选场景／执行。范围见 [PRODUCT-ADR-001](decisions/PRODUCT-ADR-001-first-software-hardware-delivery.md)，尚无固件、设备通信或真实输出。
- 清理前主线：`121efa311855d977364f7ad8707ea729b5c0e367`；仅一个 `main` 工作区，无待合并分支、标签或远程。
- CORE-001 已修复场景编号碰撞，集成 `df64f98603ca28462cf76a515b65fb39dda9b26d`。
- CORE-002 已修复 HTP 默认值下限，集成 `81ed816fff5d8a358d5e1ecf933057a148d02e8f`。
- G0 按开发基础范围结项；已停用工作器的三项残留缺陷没有被认定为修复通过。

## 任务状态

| 任务 | 状态 | 说明 |
| --- | --- | --- |
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
| PLAYER-002–005 | planned | 板卡风险验证尽早交错，再贯通输出、USB／持久包、UI／本地操作与整机验收；尚未开工 |
| AUTH-001 | planned | 手机／电脑中转正式授权，离线时当前文件可生成 24 小时临时包；到期收尾后禁止新播放。商业验收前须补服务、离线签发、可信时间和生产保护；依据 [PRODUCT-ADR-004](decisions/PRODUCT-ADR-004-relayed-device-authorization.md) |

## 最新验证

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

DESKTOP-003 已形成可运行的桌面编排增量，下一轮按真实使用反馈继续。PLAYER-001 后续优先补执行包／片段依赖闭合与模拟输出确认／故障，再与板卡工具链风险验证交错；当前不自动刷机或接灯。编排端仍需灯组／预设编辑、自定义灯具档案、场地／3D 和真正的时间线，各自依契约逐步实现；不要为“丰满”把这些领域塞进 UI 或同一个播放模块。

DESKTOP-002 已完成组件式真实工作台接入；继续依据 [UX-009](../ui-design/component-workspace-design.md) 收集操作反馈，并按成熟软件细节清单迭代；延续现代创作式布局与组件按需展开，不再以静态三图选型阻塞设计迭代。设计中的草稿、场地与片段是交互夹具，正式入口仍接真实工程与命令。UX-006／007／FIXTURE-001 的业务边界继续有效。

场地新增约束按 UX-011／ADR-010 执行：正式空间契约从开始覆盖多个异形房间、各自标高／净高与连接，先完成基础空间编辑和保存闭环，再加入舞台／观众区；不用当前稿的全局宽深代替正式模型。复杂坡顶、多层细节、完整建筑导入与 UE 继续分步实施。

授权规则已明确，AUTH-001 列为相关商业交付的前置验收项；离线许可、断电计时和有限收尾还需实现及验证，不阻塞下述工程标准工作，也不把云端授权作为当前开发原型的运行前提。

结合 [UX-006 总体框架](../ui-design/workspace-framework.md) 与 [UX-007 场地创建](../ui-design/venue-layout-design.md) 收集 MacBook 工作区、布置与目标点跟随的实际反馈，为真实界面接入制定小范围工单；先空间契约与基本搭建／摆位，再接校准与 Rust 指向求解，普通拖动不自动录制路径。[未来 iPad 准备](decisions/PRODUCT-ADR-008-ipad-primary-authoring.md) 不改变当前推进顺序，不等待设备或前置移动宿主测试。既有配适、场景编辑和保存继续复用；场景列表与 Rust 离线预览已由 DESKTOP-003 实现；时间线契约及 UE 接入按相应依赖后续实施，不用假业务填满新布局。父任务 PROJECT-001 不结项；板卡风险与 AUTH 商业门槛仍保留，当前不自动推进硬件。详见[编排端迭代计划](desktop-iteration-plan.md)。

灯具能力按 FIXTURE-001 单独建立契约／编码与可见编辑实施工单，先自定义亮度／RGB 和档案保存复用，再接功能分段与常见摇头灯。真实复杂灯具、共同指向和输出接入以经过验证的档案为前提，不等待完整云端灯库或 UE；受控测试需先具备输出控制权和停止机制，当前不自动接灯。

旧独立项目 `yunwei-ma` 有未提交源码及未跟踪文件，删除范围尚待用户明确；当前不改动该仓库。
