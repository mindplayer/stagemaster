# 实现状态

当前界面能力基线：2026-10-01，[AUDIO-005](development/tasks/AUDIO-005-local-loop-preview.md) 已补原生有界局部循环，[AUDIO-004](development/tasks/AUDIO-004-marker-group-editing.md) 已补成组卡点整理，[AUDIO-003](development/tasks/AUDIO-003-lighting-transitions.md) 已补音乐灯光段落进入渐变，[FIXTURE-003B](development/tasks/FIXTURE-003B-function-authoring.md) 已贯通功能区间建档、场景／预设和包兼容；[FIXTURE-003A](development/tasks/FIXTURE-003A-discrete-playback.md) 提供直接切换执行基础。本表是当前能力与验证边界的入口；后续增量和当前窗口／实板状态看 [STATE](development/STATE.md)，历史过程看各工单。技术框架为 Rust 核心、Tauri 2＋React／TypeScript 界面；云端 Fastify＋PostgreSQL＋对象存储尚未实施。

已有真实编辑、音频、内嵌预演及设备安装链路，仍是开发版。界面可操作、计划可编码、设备安装成功分别有证据；真实 RS485 输出仍禁用，不能据此宣称可交付演出。

| 能力 | 已实现与证据 | 仍未完成／验收边界 |
| --- | --- | --- |
| 工程编辑与持久化 | Rust 原子事务／撤销重做、严格读取、修订与保存冲突、[容量保护](module-api/project-capacity.md)、[崩溃恢复](module-api/project-recovery.md)、[最近工程](development/tasks/UX-024-recent-projects.md) | 未应用输入草稿恢复、版本迁移／比较、云端协作；其他平台需独立验收 |
| 灯具定义与配适 | [FIXTURE-002](development/tasks/FIXTURE-002-profiles-patch.md)：工程内调光／RGB／双轴、8/16 位任意粗细映射、默认值、使用中模式保护、明确换灯、批量改址／占用图；[FIXTURE-003B](development/tasks/FIXTURE-003B-function-authoring.md) 增加单色盘／图案盘／快门频闪／棱镜的命名区间、类型化选择、8/16 位编码及兼容换灯 | 物理光学元数据、关联／控制通道、个人灯库、GDTF／OFL 导入、多单元和真实试灯 |
| 灯组／预设与选择 | [资源模块](module-api/editing-library.md)、[中央平面选择](development/tasks/UX-026-scene-plan-selection.md)、[搜索面板](development/tasks/UX-027-searchable-resources.md)：有序选择、追加／扣除、预设引用／独立值、依赖与更新保护 | 通用共享预设、配方、完整克隆／跨能力换灯 |
| 常规场景编排 | [编排流程](development/tasks/UX-018-scene-editing-flow.md)：亮度／颜色／位置属性、批量混合值、场景复制、显式对象／版本预演 | 专业编程器来源追踪、盲编、分部／阻断继承／仅当前更新、暗场预定位 |
| 动态效果 | [效果模块](module-api/lighting-effects.md)：亮度／RGB 曲线、32 帧、三种过渡、灯序／相位；[EFFECT-003](development/tasks/EFFECT-003-relative-position-effects.md) 加入相对物理角度双轴运动；[固定属性编辑](development/tasks/UX-020-docked-effect-editing.md) 保持三维可见 | 功能区间内渐变／命名档位追逐、连续世界目标轨迹、速度／加速度约束、现场速度主控／节拍、像素；当前正弦采用有界采样，详见契约 |
| 摇头位置 | [POSITION-001](development/tasks/POSITION-001-moving-head-workflow.md)：独立两轴范围／反向、零偏、静态共同点、轴角渐变和关节预演 | 非相交轴／多头、自动校准、实灯精度／碰撞；相对运动不能等同持续目标跟随 |
| 单列表执行 | [播放核心](module-api/sequence-preview.md)、[专注执行视图](development/tasks/UX-021-execution-view.md)：延时／渐变／自动等待、人工推进、跳转／暂停／循环、当前／下一步／选择分离与旧版本保护；[剧本提示](development/tasks/SEQUENCE-002-script-prompts.md)含幕场、台词／动作、备注及检索 | 多执行器现场混合、总控、临时覆盖／归还、完整剧本关联及人工／定时混合调度 |
| 音乐与灯光卡点 | [音频模块](module-api/audio-editing.md)、[成熟波形](development/tasks/AUDIO-002-professional-waveform.md)、[灯光段落](development/tasks/UX-022-audio-lighting-lane.md)：真实音频、WaveSurfer、裁切／定位／手动标记／场景绑定、边界编辑和统一历史；AUDIO-003 支持确定性进入渐变／任意定位，功能属性保持直接切换；AUDIO-004 支持保留节奏的成组平移／复制／删除；AUDIO-005 临时局部循环共用音频游标 | 自动拍子、多轨、重叠／双场景动态持续交叉、跨设备时钟和有线／蓝牙延迟校准；音乐不存 ESP32 |
| 舞台与场地 | [装配](development/tasks/STAGE-001-rigging-workflow.md)、[场地目录](development/tasks/UX-025-stage-organization.md)：空间／尺寸、桁架／挂灯、阵列／对齐、测距、显隐／搜索／精确输入及一次历史；[场地锁定](development/tasks/STAGE-002-object-edit-locks.md) 保护单／多选和二维／三维、桁架联动 | 座区／座椅复合业务对象、门洞／共享墙、复杂吊点、完整三维组变换 |
| 程序内三维 | [PREVIS-002](development/tasks/PREVIS-002-single-workspace.md)：唯一 UE 视窗与共享播放进度、跨页保持；[UX-023](development/tasks/UX-023-previs-session-contention.md) 处理短时锁竞争；未建模功能灯具明确提示并保留灯位／姿态，不输出假光束 | 仍依赖本机 UnrealEditor；独立运行时打包、专业光学／轮盘模拟、规模／延迟预算和完整辅助功能 |
| 工程检查与素材交付 | [检查](module-api/project-check.md)、[资源健康](development/tasks/UX-028-project-resource-health.md)：编译／配适定位、资源摘要、缓存／随附文件分别检查、缺失重定位与保存补齐 | 当前资源检查针对已接入音乐文件；完整多媒体依赖、导出图纸、运行来源诊断未实现 |
| 编译／播放包／设备运行层 | [有界包](module-api/playback-package.md)、[安装](module-api/package-installation.md)、[传输](module-api/package-transfer.md)、[NOR](module-api/nor-package-store.md)、[运行模块](module-api/device-runtime.md) 已实施并通过相应故障／重放验证；FIXTURE-003A 已补直接切换属性与执行语义 2 包，旧包仍兼容 | 正式设备运行控制、独立本地面板、物理输出及整链路长期压力仍待验；主机故障注入不是所有板级掉电证明 |
| BLE 与实板存储 | [DEVICE-002 完整验收](development/tasks/DEVICE-002-direct-installation-acceptance.md)：免系统配对加密 GATT、身份／开发权限、下发／取消／续传／结果对账；[MEMORY-001](development/tasks/MEMORY-001-bounded-board-memory.md)：8 MB PSRAM 自检及 2 MB 有界缓存 | 现有开发凭据不等于生产身份、24 小时文件许可、安全启动；容量上限仍按目标预算校验 |
| DMX 与首版播放盒 | 共享内核在 ESP32-S3 验证，支持单路 512 通道数据编码；实板安装已通过 | GPIO21／RS485 发送仍禁用；UART 时序、电气、运行到实灯及脱机操作闭环未通过。ARM／其他盒子尚未验证 |
| 多源混合旧原型 | A0 的 HTP／LTP 与基础跟踪仍独立保留；G0 修复 R01／R02 并保留回归 | 不等同新单列表的完整专业合成器；[架构审查](architecture-review.md)中的 R03／R06／R09 等没有因新模块通过而自动关闭 |
| 扩展方向 | [伪 API](module-api/README.md)、[AI 编辑](module-api/assisted-editing.md)、[音视频／机构](audiovisual-stage-design.md)、[实体控制面](hardware-control-surfaces.md)、[采集重建](venue-capture-design.md)已有设计 | 对应真实服务、外部协议、机械控制适配、硬件面板、采集导入尚未实施；没有以设计稿冒充运行接口 |
| 云端／跨端 | 已明确共享 Rust 语义、TS 界面／云端和可替换宿主边界 | 云服务、生产授权、iPad／网页编辑与其他桌面平台尚未交付 |

## 验证命令

AUDIO-005 通过 490 Rust、151 UI、类型／fmt／严格检查／桌面及原生循环与内嵌 UE，见[记录](development/tasks/AUDIO-005-local-loop-preview.md)。AUDIO-004 通过 484 项 Rust、148 项 UI、类型／fmt／严格检查／桌面及原生组编辑验收，见[记录](development/tasks/AUDIO-004-marker-group-editing.md)。AUDIO-003 通过 475 项 Rust、136 项 UI、99 项格式检查、fmt／严格 Clippy／类型、桌面构建；原生渐变／取消撤销／保存重开／音乐和 UE 见[任务验收](development/tasks/AUDIO-003-lighting-transitions.md)。此前 FIXTURE-003B 的功能建档与 Xtensa 检查见[验收](development/tasks/FIXTURE-003B-function-authoring.md)。此前已独立修复验证 [STORE-001](development/tasks/STORE-001-explicit-lock-lifetime.md) 存储锁生命周期。测试数量是该构建的记录，不是商业成熟度评分。现有实板未刷入新的能力声明，仍需兼容检查。

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
