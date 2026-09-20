# 当前开发状态

更新：2026-09-21。当前 Astra 会话直接负责架构、实现、测试、审查、集成和状态维护；不再委派 Sol／Qwen。
依据：[DEV-ADR-002](decisions/DEV-ADR-002-astra-direct.md)、[开发方法](README.md)、[当前执行计划](execution-plan.md)。文件统一留在本项目内，见[目录规则](project-files.md)。

## 产品基线

- Rust 内核仍为 A0 静态原型；新增独立 React 交互原型 UX-003，未连接内核或设备，能力和限制见[实现状态](../implementation-status.md)。
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
| [PLAYER-001](tasks/PLAYER-001-software-playback-foundation.md) | planned（已细化） | 首先在电脑贯通最小工程、受限编译、虚拟时间播放与 512 通道输出核对；精确契约及实现待开始 |
| PLAYER-002–005 | planned | 板卡风险验证尽早交错，再贯通输出、USB／持久包、UI／本地操作与整机验收；尚未开工 |

## 最新验证

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

下一项按 [PLAYER-001 工单](tasks/PLAYER-001-software-playback-foundation.md) 实施：先形成精确行为契约与架构变更说明，再在电脑完成虚拟时间参考执行、受限编译和可核对的 DMX 逻辑输出；渐变中断、跳转、循环、释放、片段入口、依赖闭合及资源拒绝分别验收。按[执行计划](execution-plan.md)尽早交错验证现有 ESP32 的工具链和物理输出风险，再接传输无关的包安装、持久化及现有 UI。USB 先调通，局域网和蓝牙按各自适用角色分阶段验证。盒子应独立供电、自主播放，最终具备本地选场景的操作面。演示适配器不能成为正式内核语义，R03–R09 等既有问题仍待处理。
旧独立项目 `yunwei-ma` 有未提交源码及未跟踪文件，删除范围尚待用户明确；当前不改动该仓库。
