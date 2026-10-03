# DEVICE-003：设备运行控制链路

状态：分权、运行应用入口、有界消息、共享客户端、原生运行服务及运行队列增量已验证，完整设备运行链路实施中。2026-10-03；首增量基线 `4d741a4`，第二增量产品基线 `8d13434`，第三增量 `61047e8`，第四增量 `0c832a5`，第五增量 `8d0d093`，main，当前会话单写者。本工单接续框架审查揭示的 AUDIT-001 F01 缺口；FRAMEWORK-001 已于 `f22a611` 收尾，完整 goal 保持 active。

## 当前事实与范围

Runtime／ManagedWorker 已有已安装目录、载入、执行、控制权、回执和维护门；device-channel 已复用 TCP／GATT 承载，安装与运行客户端分别协商。原生 Service／Ble 已有明确运行入口，当前固件与桌面设备页面仍只接诊断／安装。不得再造播放器，也不能把安装成功显示为运行或物理输出。本轮先实现设备运行入口的分权前置，再分增量接通正式操作与可见状态。

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
