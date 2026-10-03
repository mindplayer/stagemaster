# DEVICE-003：设备运行控制链路

2026-10-04 最新：OUTPUT-002 看门狗／逻辑 UART 实板增量已完成，修复栈与发送调度故障后原包 180 秒通过；断线约 40 Hz，UART 无故障，完整证据见[输出工单](OUTPUT-002-dmx-transmission.md)。当前设备保留该逻辑侧实验镜像，节目已停止且无控制者，原 28 节目保持，GPIO21 禁用。下文增量说明是各提交时的历史状态；真实差分波形、最重负载和长期压力仍未完成，完整 DEVICE-003 不关闭。

OUTPUT-002 运行／输出协调增量已完成：原服务的 runtime-dmx-probe 接入原 Port／固定队列，真实静默回执门控 ManagedWorker 维护；128 项相关测试和实际 S3 检查／链接通过。详见[当前输出工单](OUTPUT-002-dmx-transmission.md)。GPIO21 强制关闭、未刷入，正常 runtime-gatt 保持原行为；独立硬件看门狗、实板新组装时序／资源及真实 RS485 仍待验收，完整 DEVICE-003 不关闭。

OUTPUT-002 有界驱动增量已完成原 Port → 队列 → 单帧事务的软件连接和 S3 交叉构建，45 项相关测试通过；见[验收记录](OUTPUT-002-dmx-transmission.md)。正常固件尚未切换，接续 Runtime 快照、输出激活、维护门、引脚和看门狗接线，再实测线路。本轮未刷机或打开设备，完整 DEVICE-003 继续开放。

状态：分权、运行应用入口、有界消息、共享客户端、原生服务／队列及桌面节目操作已完成软件验收；真实 v2 运行固件已受控刷入，28 节目恢复、原生 GATT 与桌面运行控制已通过第一轮实板验证；完整链路接续实际帧时序／压力及物理输出。2026-10-03；首增量基线 `4d741a4`，第二增量产品基线 `8d13434`，第三增量 `61047e8`，第四增量 `0c832a5`，第五增量 `8d0d093`，第七增量 `5688d6a`，第八增量 `c0fc927`，main，当前会话单写者。本工单接续框架审查揭示的 AUDIT-001 F01 缺口；FRAMEWORK-001 已于 `f22a611` 收尾，完整 goal 保持 active。

第十增量已完成[实际帧测量](DEVICE-003-runtime-frame-measurement.md)：基线 `d9cba78`，诊断临界区修正后原工作器单次 180 秒观察通过、断线约 40 Hz／最长 25.810 ms，资源和边界如实记录。ESP32-S3 保留受限独立播放器角色，完整工程仍由主机编译；物理输出和长期验收未完成。

## 当前事实与范围

物理输出前置 [OUTPUT-002](OUTPUT-002-dmx-transmission.md)已补单帧事务／S3 UART 适配准备，真实 Runtime 和原 Port 的完整字节对照、取消／完成软件测试及交叉构建通过；有界交接／原服务接线及逻辑 UART 联测已完成，真实差分线路测量未完成，当前设备仍禁用 RS485。

Runtime／ManagedWorker 已有已安装目录、载入、执行、控制权、回执和维护门；device-channel 已复用 TCP／GATT 承载，安装与运行客户端分别协商。原生 Service／Ble 已有明确运行入口，新 runtime-gatt 固件已刷入本台设备，原生与桌面实际运行控制验证通过；桌面维护入口的软件证据保持，完整维护／物理输出仍须独立验收。不得再造播放器，也不能把安装成功显示为运行或物理输出。接续实板 GATT、自主运行和物理输出证据。

第一增量依 [ADR-127](../decisions/PRODUCT-ADR-127-device-operation-permissions.md)：限定 `device-auth::application`、原 `install-worker::secure`、相关验收和契约。既有安装凭据不提升权限；不修改工程／设备包／配置字节或固件。源码按许可范围、准入、安装适配与测试分文件，目标新增文件低于 300 行。全量基线仍运行时使用项目 `tmp/framework-001-light-target` 隔离构建，不覆盖原测试产物。

## 完整出口

1. 操作范围、实际设备能力、连接身份、运行控制租约与播放许可分别判断；旧安装入口无隐式运行权限。
2. 复用 Runtime／ManagedWorker 的目录、选择／载入、执行／暂停／继续／下一步／停止；稳定身份与状态／历史回执分开。维护需真实静默确认，外部不能伪造。
3. 同一应用语义经过已有有界承载；取消、过期、断开／重连和回执丢失不重复执行，不停止自主节目，不抢占新控制者。
4. 桌面区分已安装、所选、已载入、运行实例、软件采样和物理发送；未知／未支持如实显示，保留错误定位与操作上下文。
5. 软件用真实包、原安装器和原 Runtime 验证；实际固件／物理输出另按 PLAYER-002／005 取得证据。只完成分权或软件端点不能关闭 F01。

## 第一增量验证

验收范围：真实会话的独立／组合范围和期限拒绝、旧配置兼容、运行范围不能打开安装工作器；原安装、会话及两种软件承载回归。完成后记录确切结果和仍未接通部分。

已接线：`Permissions` 独立 29 行文件，可信 scoped 构造与逐次 `Session::require`；原 Gateway 在打开和工作时要求安装范围。auth／install-worker 64 项通过，包括新 4 项真实会话与 2 项安装网关用例；旧安装、维护、回执、加密分片和恢复断言保持。日志 `logs/device-003-scopes-tests.log`。device-channel／device-host 的 59 项连接承载回归通过，包含真实本机 TCP 和 20 字节 GATT 软件分片、安装落盘、取消、重连／权限拒绝及原连接状态；日志 `logs/device-003-scopes-connection-regression.log`。本增量合计 123 项通过、0 失败、0 忽略。

全工作区全部目标严格 Clippy（启用 install-worker/application）和 ESP32 application-gatt 的 Xtensa 严格检查通过，日志 `logs/device-003-scopes-clippy-final.log`、`logs/device-003-scopes-xtensa.log`。没有刷入或运行板卡。首次主机检查发现文档代码引用缺少反引号，已修正并通过，不改变执行逻辑；初始日志保留。本次涉及的手写代码文件最大 156 行，无第三方／锁文件改动；未改 UI，也未重复桌面打包、原生窗口或物理验收。

## 第一增量时的接线约束

当前 SMAP v1 必须先打开安装工作器，再公告就绪；不能将其直接复用为连接即进入维护的运行通道，否则观察／重连会干扰已播放内容。下一增量先定义独立运行就绪／版本协商，再让目录与运行命令进入原 ManagedWorker；安装仍遵守原维护窗口，读取、控制和安装各自检查范围。复用 device-channel 的记录承载与安全会话，不复制无线收发器。真实执行时间来自设备单调调度，不能靠收到控制消息时才 tick。精确字节协议与固件调度另行评审后实施。

本增量审查结论：上述操作范围出口已满足，按 `feat(device): separate installation and runtime operation permissions` 集成；原配置没有提升权限，现有安装链兼容。格式、差异与 7 个文档的 472 个本地链接通过。完整任务第 2～5 项的运行接线／界面／实物出口仍开放；FRAMEWORK-001 的原完整基线作业单独继续，不把当前 123 项作为全量结果。

## 第二增量：运行应用入口

产品基线 `8d13434`；期间仅框架审查文档以 `f22a611` 集成。依 [ADR-128](../decisions/PRODUCT-ADR-128-device-runtime-application.md) 调整先后顺序：先证明原工作器能承担有界请求和实时撤销，再冻结相匹配的就绪／字节协议，避免无线保活任务执行慢载入。本次限定 install-worker 的 application 模块、实际会话／运行测试、必要测试依赖和文档；未修改无线、固件或桌面行为。

新增 `operations/{mod,model,lifecycle,dispatch}.rs`，创建连接不进入维护或接管；观察目录、步骤与状态，控制租约及原 Action 复用 Runtime。请求期望修订、连续序号、单份历史回执、执行前后活性检查与旧连接释放保护均在同一串行所有者内。每个已准入会话只能保留一个 Connection；可信 live 回调不能缓存 Grant。类型化结果不等于网络字节格式，不承诺可直接装入单个安全记录。详见[调用契约](../../module-api/device-runtime-application.md)。

验证与日志：

- 真实会话／目录／执行／维护／故障等首批 11 项通过，`logs/device-003-runtime-tests-final.log`。
- 工作器与 Runtime 相关回归 56 项通过，包含上述 11 项，不重复计数；`logs/device-003-runtime-regression.log`。
- 随后补齐目录、步骤分页及释放目录后历史文本两项通过，`logs/device-003-runtime-catalog.log`。本增量合计 58 项，其中新增 13 项，0 失败、0 忽略。
- 全工作区全部目标严格 Clippy（含 application）通过，`logs/device-003-runtime-clippy-reviewed.log`；Xtensa application-gatt 严格检查通过，`logs/device-003-runtime-xtensa.log`。最终文档整理后再检查格式与同一固件目标，不运行或刷入板卡。

测试采用真实工程包、原安装器／ManagedWorker、Noise／Session，实际 512 通道软件输出与独立 Player 对照；授权失效不会被测试伪装为成功回滚。新增 dev-dependency 仅为已有 playback crate，Cargo.lock 只增加该本地测试依赖边；无第三方版本、生产依赖或已有工程／设备包／配置字节变化。首次编译的类型推断和直接测试依赖缺失已修复，首次严格检查的文档问题已修正，未削弱测试断言。新生产文件均低于 200 行，测试文件低于 300 行。

第二增量审查结论：按 `feat(device): expose admitted runtime operations on the existing worker` 集成。本任务的完整出口仍开放：正式运行就绪／版本与有界编解码、承载队列／独立固件调度、桌面目录／操作／状态、实际端口和物理输出。完成本入口不关闭 F01，也不把旧框架轮重新打开。未操作用户工程、output/、窗口、蓝牙、声卡或物理灯具。

最终整理检查：全工作区全部目标严格 Clippy 退出码 0；Xtensa application-gatt 复查退出码 0，`logs/device-003-runtime-xtensa-final.log`；fmt／差异检查通过。8 份文档的 490 个本地文件链接无缺失，`logs/device-003-runtime-doc-links.json`。本增量不重复已经结束的 FRAMEWORK-001 全量基线，也不将相关 58 项称为最新全部产品回归。

## 第三增量：有界消息与协商

基线 `61047e8`，main；上一目标回合已提交实际产品代码，分类为 progress。遵循 [ADR-129](../decisions/PRODUCT-ADR-129-runtime-wire-protocol.md)，新增 no_std 协议 crate；原操作模型移入共享契约，旧 operations 路径重导出。新增工作器 `negotiate`／`process_message`，通过实时授权事实生成独立运行就绪，重用原请求去重与执行，不打开安装工作器。协议数据只能描述权限，不能构造真实 Grant。

写入范围：新 runtime-protocol、原 install-worker 的操作模型／有界消息适配／测试、工作区清单和主机／固件锁文件、相关文档。宿主与固件均只增加本地模块依赖边，不升级第三方。协议严格依赖 1,280 字节单消息预算，单条步骤页极限 1,142 字节；不额外发明分片。具体数组、操作号、回复和错误号见 [SMRT v1](../../module-api/device-runtime-wire.md)。

实际验证：

- `logs/device-003-wire-tests-reviewed.log`：74 项相关 Rust 测试通过，0 失败／忽略；包含原 58 项、13 项协议与 3 项加密工作器新增测试，文档测试 0 项。
- 协议专项覆盖独立固定字节向量、全操作／状态／错误、最大文本／序号、截断／尾随、未知版本／位／类型、超长和错误 UTF-8、无界数组与逐位变异。首次极限测试的手算长度多计 1 字节，复核数组各段后改为独立求和 `8+1+41+42+2+1048=1142`，没有改生产编码来迎合测试，也没有删除上限或完整性断言。
- 加密专项采用真实 Noise＋Session 和原安装包／工作器。协商不抢权或进入维护，目录到选择／载入／执行及暂停／继续／下一步／停止贯通；重复执行保持原实例和历史回执，完整 512 通道帧与独立 Player 一致，断线后原实例自主运行；观察拒绝、过期修订、错误安装消息和重复协商均拒绝。
- `logs/device-003-wire-clippy-reviewed.log`：全工作区全部目标严格 Clippy（application）退出码 0；`logs/device-003-wire-xtensa.log`：application-gatt 的 Xtensa 严格检查退出码 0。初始检查的参数按引用／命名／条件写法已修复，未放宽 lint。

审查结论：本增量按 `feat(device): encode bounded runtime negotiation and operations` 集成，源码／测试新增文件低于 300 行。正式 Channel 仍只有安装入口；接续运行握手和共享客户端，再承载队列、固件独立调度、桌面状态及真实端口。加密内存往返没有冒充 TCP／GATT 实际运行验收。未刷机，未操作真实设备、声卡、UE、用户工程或 output/；完整 DEVICE-003／goal 仍开放。

最终格式／差异检查通过；7 份文档的 490 个本地文件目标无缺失，两个锁文件的外部包版本／校验值与基线逐项一致，`logs/device-003-wire-doc-check.json`。生产依赖树无认证／安全会话／安装工作器／Tokio／蓝牙／桌面框架，`logs/device-003-wire-dependencies.log`，依赖方向未反转。用户 output/ 不纳入提交。

## 第四增量：共享运行握手与客户端

基线 `0c832a5`，main；上一目标回合已提交有界消息，分类为 progress。依 [ADR-130](../decisions/PRODUCT-ADR-130-runtime-client-channel.md) 在原 device-channel 内增加运行协商及 RuntimeClient；范围为 Channel／设备描述／主机描述标签、相关测试和文档，未修改固件运行逻辑或桌面控制页面。

运行端点以独立能力位声明；原 Noise 认证抽为共同实现，私有准入分别存储安装和运行事实，原安装 `peer()` 不返回运行权限。客户端拥有一个待确认请求和一份历史回复，禁止把已发送应用消息的 Channel 再包装而重置序号。取消／发送失败保留不确定意图且旧连接失效，业务失败不伪装为断线；显式重试复用原请求、不得续期，迟到重复回复不完成下一个请求。宿主仍负责串行轮询和心跳，客户端不偷偷创建调度器。

验证结果：

- `logs/device-003-client-regression.log`：155 项相关 Rust 回归通过，0 失败／忽略，含 16 项新增（两种承载 2、权限／入口 5、回执 3、生命周期 4、描述 2）；覆盖原安装／连接／工作器／运行与协议回归。该结果不是全工作区全部测试。
- 实际本机 TCP 和 20 字节 GATT 软件分片，经过真实安全会话、真实包安装与原 ManagedWorker，完成目录／选择／载入／执行／暂停／继续／下一步／停止。断线后同一运行实例独立推进，完整 512 通道软件帧与独立 Player 对照；重连查询没有自动取得控制权。
- 重试测试在回复加密发送前留置原结果；不声称能在任意丢失密文后复用原序号流。相同历史回执不重做控制租约；取消、失败、错误关联、观察权限、旧安装回执以及已使用 Channel 重包装均拒绝。
- 虚拟时间专项证明每 2 秒保活和中途重试不延长 30 秒请求期限；相同应用回复仅占一个保活暂存槽，持续重复消息不能延长整个心跳的 5 秒期限。
- `logs/device-003-client-clippy-reviewed-final.log`：全工作区全部目标严格 Clippy（application）退出码 0。`logs/device-003-client-xtensa.log`：Xtensa application-gatt 严格检查退出码 0，未刷机或运行板卡。

初次检查发现测试帮助模块导出、异步测试栈和函数长度问题，已用共享夹具、测试 Future 装箱与按职责拆分修复；未放宽 lint 或删减断言。新增生产和测试文件均低于 300 行。主机 Cargo.lock 只新增 4 条本地依赖边，生产新增 runtime／runtime-protocol，worker／playback 为测试依赖；无第三方升级或固件锁文件变化。契约见[共享客户端](../../module-api/device-runtime-client.md)。

本增量审查结论：软件客户端出口已满足，按 `feat(device): connect runtime clients through shared record channels` 集成。下一项为原生设备服务／生产队列与固件独立调度，再接桌面节目目录／运行操作／状态，实际输出另行验收。完整 DEVICE-003／AUDIT-001／goal 仍开放；已完成框架轮保持关闭。用户请求进度时核对：本轮框架验收已完成，当前设备运行链路粗估 60%～70%，这是剩余工作量判断，不是按测试数推导的商业交付比例。未操作真实蓝牙、声卡、UE、用户窗口／工程或 output/。

最终 fmt／差异检查通过；9 份文档的 503 个本地文件目标无缺失，所涉 Rust 文件最大 276 行，`logs/device-003-client-doc-check.json`。外部锁文件包记录与基线一致、固件锁文件未变；生产依赖树不含安装工作器、蓝牙驱动、Tauri 或音频后端，`logs/device-003-client-dependencies.log`。本次没有重复框架全量基线、桌面打包或原生窗口验收。

## 第五增量：原生运行连接服务

基线 `8d0d093`，main，当前会话单写者；上一回合完成代码验证与提交，分类 progress。依 [ADR-131](../decisions/PRODUCT-ADR-131-native-device-runtime-service.md) 在原 Service／Ble 接入 RuntimeClient；写入限于 device-host、必要本地依赖及文档，未改核心协议字节、固件或桌面页面。

新增显式 `connect_runtime`／`exchange_runtime`／`runtime_snapshot`，原安装连接、扫描、取消、保活和释放路径保留。运行状态区分当前准入、带所属连接代次的历史回复和待确认意图，发送前登记；排队撤回不发送，发送中取消和超时保留不确定性。真实业务失败保留为回复，观察连接本地拒绝控制不会耗用请求号或断线。重连不重放、不接管、不停止节目。

接线审查发现旧安装服务在安全确认后主动进入安装工作器，不能与运行入口复用相同服务并猜测意图；因此保留原安装服务，运行使用独立 edb0／edb2／edb3，底层 GATT 记录代码共用。通知按服务和特征双重筛选，缺少运行声明／端点拒绝，不降级安装。设备声明与握手事实匹配仍由本机和设备各自核验；没有给 SMDV v1 配置提升权限。

实际验证：

- `logs/device-003-host-regression-accepted.log`：device-host／device-channel／device-upload 合计 96 项测试通过，0 失败／忽略，含 9 项新增。不是全工作区全部测试。
- 8 项集成用真实 RuntimeClient、安全会话、软件字节流、安装包与原 ManagedWorker 驱动 Service，覆盖目录到执行、暂停继续／下一步／停止、互斥、旧 epoch、观察拒绝、确认业务失败、排队撤回、发送中取消、未确认执行后断线与不重播、持续保活且 30 秒总期限不延期、描述不匹配。断线后同一设备运行实例和释放控制者分别断言；历史记录在新搜索中仍标记原连接代次。
- 原生通知层增加 1 项运行／安装／错误特征隔离测试，原排序／溢出／片段超时测试保持；两种共享记录承载和旧安装／上传回归保留。
- `logs/device-003-host-clippy-accepted.log`：全工作区全部目标严格 Clippy（application）退出码 0，包含桌面 Rust 调用方。未修改 UI／固件，因此未重复 UI 打包、原生窗口、Xtensa 或实板测试。

首次集成夹具缺少诊断协议要求的内部堆容量、虚拟时间未推进握手轮询，已按既有协议补齐并让定时任务真实推进；保持全部状态与期限断言。测试引用参数、异步锁生命周期和已有 net 测试特性缺失也已修复，未放宽 lint。一个测试文件初次路径误写到项目同级新目录，发现后立即移入本项目，空目录已删除，无外部残留产物。测试共用既有 device-channel 的真实安全／安装夹具，没有另写协议模拟器。

本增量审查结论：原生运行服务的软件出口满足，按 `feat(device): integrate native runtime connection services` 集成。新增生产依赖只有已有 runtime-protocol，worker／runtime／playback 和 Tokio net 为测试依赖；Cargo.lock 只增 4 条本地边，无第三方升级或固件锁文件变化。下一步生产设备队列／运行 GATT 端点与固件独立调度，再接桌面操作和真实输出；完整 DEVICE-003／AUDIT-001／goal 保持 active，框架轮保持已完成。用户 output/、工程、窗口及物理设备未操作。

最终 fmt／差异检查通过，10 份文档的 518 个本地文件链接无缺失；所涉手写 Rust 文件最大 266 行。生产依赖树不含安装工作器，外部包锁定记录不变、整个固件目录未修改，路径误写产生的空目录已核对不存在。日志 `logs/device-003-host-doc-check.json`、`logs/device-003-host-dependencies.log`。

## 第六增量：运行队列与独立有效性

基线 `9f63a80`，main，当前会话单写者。上一回合按用户要求核对进度，无产品变更，分类 no progress；本回合重新核对现状后接续实际实现，无等待中的进程或外部阻塞。依 [ADR-132](../decisions/PRODUCT-ADR-132-runtime-worker-queue.md) 与[队列契约](../../module-api/device-runtime-queue.md)，范围为 device-session／auth 的最小截止读取、install-worker 的运行队列／网关、真实会话与工作器验证，追加 device-channel 的实际承载集成测试。未更改工程／包／协议字节、固件行为或桌面页面。

通信侧 Gateway 持有真实 Session，生成一个带 epoch／本地票号／固定期限的类型化命令；工作侧 Endpoint 跨连接保留原 Connection、调用唯一 ManagedWorker。Live 从真实权限、接收期限和当前网关截止点取交集，由平台独立同步发布并在清理时撤销；工作前后重读，不把入队快照当永久许可。5 秒协商／回复／保活与 30 秒普通工作期限固定；相同未完成请求不重复入队，后续重试由原运行历史返回。发送密文在承载确认前不可变，旧完成不能完成下一请求。

实际验证：

- `logs/device-003-queue-regression-accepted.log`：221 项相关 Rust 回归通过，0 失败／忽略；20 项新增为 2 项权限截止、16 项队列／撤销／错误／容量和 2 项实际承载。覆盖 device-session、auth、install-worker、runtime-protocol、runtime、device-channel、device-host；不是全工作区全部测试。
- 真实安装包经过队列进入原播放器，目录／选择／载入／执行／暂停／继续／下一步／停止、完整 512 通道软件帧与参考 Player 一致。断线后同一实例继续推进，重连只读取、不自动抢权。
- 工作留置 8 秒仍正常保活，相同待处理请求不增加工作；30 秒精确截止不因心跳或重试刷新。实际原包 Reader 内触发撤销，加载可以完成，但旧输入释放、完成拒绝，未假称回滚。旧 epoch／旧票号、观察权限、固定期限及在途心跳与回复先后均覆盖。
- 共享 RuntimeClient 通过实际本机 TCP 和 20 字节 GATT 软件分片，进入新 Gateway、真实单格同步队列及独立阻塞工作线程，调用原 ManagedWorker；每次操作也经过心跳往返。该软件线程适配验证队列边界，不替代 ESP32 的核心调度／射频实测。
- 复查发现工作器先采样时间、通信侧随后发布新心跳时，以发布时间作 Live 下界会误撤销有效连接；改用固定准入时刻，Gateway／Connection 各自保持计时回退检查。专门竞态测试同时确认更新后精确接收截止仍生效。
- `logs/device-003-queue-clippy-verified.log`：全工作区全部目标严格 Clippy（application）通过；`logs/device-003-queue-xtensa-integrated.log`：Xtensa application-gatt 严格检查通过。fmt 与差异检查通过，无第三方／清单／锁文件变更。

初次严格检查发现命令值传递、条件写法、长测试函数和测试分号问题，已通过可复制的小命令、条件整理和共用验证帮助函数修复，未放宽 lint；追加承载帮助模块首次导入路径错误已修正。首次补丁中的错误更新路径被工具原子拒绝，未写出项目。既有测试断言保留，没有隐藏失败用例；初始日志保留。

审查结论：共享运行队列出口满足，按 `feat(device): dispatch runtime work through bounded authenticated queues` 集成。生产与测试按职责拆分，新增文件低于 300 行；Gateway／Endpoint 固定容量只是类型预算，不能视作整机峰值。下一项是固件独立运行 GATT、原工作器非阻塞持续推进与 Live 同步槽，再接桌面操作和实际端口。没有刷机、连接蓝牙、操作声卡／UE／用户窗口或 output/；完整 DEVICE-003／AUDIT-001／goal 仍开放，已完成 FRAMEWORK-001 不重开。

最终本地检查：6 份变更文档的 444 个本地文件链接均有效，本次涉及 Rust 文件最大 219 行；清单、锁文件及固件目录无变化，记录 `logs/device-003-queue-doc-check.json`。用户 output/ 未纳入提交。

## 第七增量：固件运行服务与独立调度

基线 `5688d6a`，main，当前会话单写者。上一进度查询仅核对状态，分类 no progress；本回合确认所有旧验证进程已结束，接续未提交实现，最终验证并集成。依 [ADR-133](../decisions/PRODUCT-ADR-133-firmware-runtime-gatt.md) 和[固件契约](../../module-api/firmware-runtime-gatt.md)，限定固件接线、显式开发权限配置、必要主机测试及工具文档；未改桌面或核心协议／工程／设备包字节。

`ble/application/{gate,protocol,port,mod}` 共用安装和运行的安全会话／记录，首个入口冻结；`runtime_io` 独立同步整个 Live，工作前后重查。`installation/dispatch` 保留唯一 ManagedWorker，以 25 ms 节拍独立推进，回复队列背压不能等待阻塞完成发送。上电只绑定目录、失败保留维护恢复；普通观察不会抢权／触发维护，实际 OutputDisabled 仍由 main 全程持有。

SMDV v2 仅在可信本地配置中显式表示安装／观察／控制范围，v1 不升级；生成工具可用 `--runtime` 新建随机密钥对，不能覆盖。项目 data/DEVICE-003/runtime-build-validation 使用虚构设备编号，仅构建验证；不修改实板和已有 DEVICE-002 凭据，不能用于刷机。

验证与证据：

- `logs/device-003-firmware-regression.log`：device-auth／device-session／install-worker／device-channel／device-host／runtime 的 214 项相关测试通过，0 失败／忽略。随后新增保活、畸形消息／时钟回退、观察权限三项；最终 `logs/device-003-firmware-protocol-final.log` 的 7 项固件协议测试全部通过。去重合计 217 项，其中新增 9 项（配置 2、固件 7），不是全工作区测试。
- 7 项直接包含实际固件 gate／protocol 源码，使用真实 Noise、原包和工作器：v1／v2 安装完整传输、运行连接不维护／不抢权、断线后同实例推进、旧配置拒绝运行、发布后排队、队列失败／过期撤销、8 秒工作留置期间保活、错误消息／计时和观察权限拒绝控制。队列与独立主机线程的物理承载边界沿用第六增量，未将主机执行冒充 ESP32 射频或第二核实测。
- `logs/device-003-firmware-clippy-final.log`：全工作区全部目标严格 Clippy（application）通过。`logs/device-003-firmware-install-check-final.log`、`logs/device-003-firmware-runtime-check-final.log`：旧安装和新运行 Xtensa 严格检查通过；`logs/device-003-firmware-runtime-build-final.log` 最终完整构建通过。
- `logs/device-003-firmware-image.log`：espflash save-image 离线生成应用镜像，808,704 字节，占 3 MiB ota_0 的 25.71%。无端口连接或刷写。保留原工具链 LOAD RWX 告警，段记录 `logs/device-003-firmware-segments.log`，未放宽检查掩盖。
- 最终链接段与符号记录 `logs/device-003-firmware-size.log`／`logs/device-003-firmware-symbol-sizes.log`：`.bss` 192,468、`.data` 13,256、`.data.wifi` 284 字节；bss 已含堆 131,072、第二核栈对象 32,784（栈体 32,768）、主任务槽 14,112、工作器任务槽 7,792。新增运行请求／完成队列和 Live 槽为 120／1,344／128。主核 `.stack` 链接预留 93,436。以上是静态包含关系，不相互重复加总、不表示峰值。PSRAM 缓存仍为既有 2 MiB，未宣称实板内存验收。

初始检查的 base feature 未用分支、测试误读 Installed 字段与 Settings 参数 lint 已修复；最终追加“仅观察许可”真实固件测试，未删断言或放宽规则。新增手写文件最大 212 行；所涉 ble.rs 为 323 行，保留 GATT 声明／平台连接组装，应用协议已抽取。无文件超过 500 行，无新第三方依赖或锁文件变化。

本增量软件出口按 `feat(esp32): integrate authenticated runtime scheduling and GATT` 集成。完整 DEVICE-003、AUDIT-001 与 goal 保持 active；下一项桌面设备目录／操作／状态和维护切换，再实板无线、自主运行、时序／内存与 UART DMX 验收。用户 output/、工程、窗口、设备均未操作；FRAMEWORK-001 保持已完成。


## 第八增量：桌面设备节目操作

基线 `c0fc927`，main，当前会话单写者。上一回合仅答复用户进度，分类 no progress；本回合核对未提交源码、真实存活的 Vite 验收服务和已有终止作业，接续原增量。依 [ADR-134](../decisions/PRODUCT-ADR-134-desktop-device-runtime.md)，新增[本机 JSON 投影与桌面接口](../../module-api/desktop-device-runtime.md)，复用原 Service、控制租约、协议和设备工作器；Tauri 只转发，界面只协调命令和观察。

运行连接与原诊断／安装连接明确选择。目录搜索、载入、步骤选择／执行、暂停、继续、下一步、停止、取得／归还／确认接管、维护切换均已接原生运行端口。面板收起保留草稿，外部接管撤销本界面的续期资格；发送或读取失败立即禁用操作，历史状态与未确认结果保留，重连不自动重播／抢权。目录取消等待当前请求结束并丢弃结果；未读完整与真实空目录分开。未知通信耗时不显示空单位，断线后不展示旧诊断为当前事实。

原生连接的期望权限只来自本机已加载配置，旧安装配置不升级。全部 u64 以十进制字符串投影，严格拒绝额外字段和非法标识。续期使用原生剩余准入期限并保留传输余量，不允许前端任意延长；这不是商业授权限时方案。

实际验证：

- `logs/device-003-desktop-runtime-regression.log`：device-host／device-channel／device-upload 共 102 项通过，0 失败／忽略，含新增 JSON 操作、权限拒绝、维护／期限和全宽计数投影 4 项。随后追加可信配置实际连接，`logs/device-003-desktop-runtime-native-final.log` 中 13 项原生运行测试全通过。去重合计 103 项，新增 5 项；不是全工作区全部测试。
- 新原生测试经过真实安全会话、原安装包和 Runtime，验证无自动控制／执行、目录到暂停继续、旧修订拒绝、断线历史／自主运行、维护取消、租约期限、非法 JSON 未派发、u64::MAX 精确字符串、v1 拒绝及 v2 观察／控制范围实际握手。
- `logs/device-003-desktop-runtime-clippy-final-reviewed.log`：全工作区全部目标严格 Clippy（application）通过。初次检查的值传递、长字面量已修复；状态枚举与维护测试权限夹具按原真实契约修正，未放宽规则。Serde 空动作改用空结构体变体后，额外 duration 字段确实被拒绝。
- `logs/device-003-desktop-runtime-ui-tests.log`：314 项 UI 逻辑通过，含新增 3 项回复／精度、租约／续期、状态标签测试。最终类型检查 `logs/device-003-desktop-runtime-ui-check-final.log` 通过。前端没有另建节目计时器或权限判定器。
- 隔离页面使用正式 DeviceCenter／DeviceRuntime，人工交互验证连接仅观察、显式取得／接管取消与确认、选择／载入／执行／暂停／继续／下一步／停止、当前步骤、筛选隐藏选择、收起保持、维护执行禁用、目录取消与刷新恢复、读取失败即时禁用、发送后断线提示及重连无新增控制请求。旧在途目录在取消后没有被发布；最终全新页面控制台无错误。原热更新测试页曾出现夹具 createRoot 重执行告警，清洁页面复验无此问题，未将开发热更新混作正式运行。桌面 1440×940 DOM 尺寸检查无横向溢出，默认窄面板亦可滚动操作。
- 界面记录 `logs/device-003-desktop-runtime-ui-acceptance.txt`，截图 `data/DEVICE-003/ui-validation/program-controls.png` 与 `uncertain-disconnect.png`；均是隔离夹具，不是实板。临时标签和本轮 Vite 服务已正常关闭，测试视口已恢复。
- `logs/device-003-desktop-runtime-build.log`：正式桌面 Tauri 构建／应用包通过，产物在 `tmp/framework-001-light-target/debug/bundle/macos/舞台大师.app`。原 Vite 主包大于 500 kB 提示保留，未调整阈值掩盖。本轮未启动此应用连接实板，不声称完成真实原生 GATT 交互；固件源码未变化，没有重跑 Xtensa 或刷机。

审查结论：桌面软件入口增量按 `feat(device): add desktop program runtime controls` 集成。Rust 投影、类型、状态协调、显示组件、发现组件、测试分别组织；三个前端手写文件约 318～328 行，评估后保留各自完整状态／视图职责，发现列表已抽出，无文件超过 500 行。两条原有本地测试依赖移为显式生产依赖，没有新增第三方或锁文件变化；核心不依赖 UI。用户 output/、工程、正式应用窗口、UE、声卡与物理设备未操作。

完整 DEVICE-003／AUDIT-001／goal 保持 active。下一步核对实板和原凭据，准备对应运行固件与受控实板验证，再贯通 UART DMX；必须分别证明无线保活、独立调度、峰值内存和真实输出。FRAMEWORK-001 已完成的框架轮不重开，不能将本次打包／隔离界面证据当成硬件已经可交付。

## 第九增量：实板与正式桌面联合验收

基线 `3af3e98`，结果为本次 `fix(device): validate board runtime and preserve controls during polling` 提交。独立[实板工单](DEVICE-003-runtime-board-acceptance.md)记录真实配置、固定分区刷写、原 28 场景恢复与原生实际 GATT 全流程；断线重连后同一启动／运行实例进度继续，未自动取得控制权。新增可复现命令沿同一个 Service／JSON／Runtime，非协议模拟器。

正式桌面搜索、连接、目录、中文筛选、载入、开始／暂停／继续／停止、收起保持和运行中重连验证通过。实测发现周期刷新把按钮短暂禁用，已抽出 35 行串行协调器修复：观察与交互忙状态分开，最多一个操作等待、断线取消、观察失败不派发；不复制核心或放宽权限。103 Rust、318 UI／类型、全工作区全部目标严格 Clippy、实际固件检查／构建及最终桌面打包通过，fmt／差异检查通过；不是最新全工作区全部测试。

600 秒观察中记录堆峰 53,140 字节、栈采样 20,525 字节，不能替代完整峰值或长时压力。诊断 LIVE tick 不是节目实际帧生成计数，UART DMX 全程禁用。最后停止且归还控制，用户正在查看的隔离窗口保留。接续实际帧时序与资源采样、UART／物理灯具；完整 DEVICE-003、AUDIT-001／goal 仍未完成。
