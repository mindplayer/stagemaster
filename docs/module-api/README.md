# 模块伪 API 方案 0.3

日期：2026-09-11。依据：[架构 v0.5](../architecture.md)、[完整目标再评估](../architecture-evolution-review.md)、[声光视频设计](../audiovisual-stage-design.md)与[硬件控制面](../hardware-control-surfaces.md)。本目录描述计划中的接口、调用和实现约束，**没有服务实现，不表示已能跨设备播放**。

0.3 在 0.2 的外部控制、监看与硬件边界上，拆分操作会话与节目上下文、稳定监看键与连接引用、工程包与执行包，补齐单域编译／激活、空运行状态及服务契约声明。协议草案为 draft-0.3；这是设计阶段的破坏性修订，不宣称兼容旧版或已有迁移实现。

用户要求模块像 class 一样能清楚调用，同时支持跨端和跨设备。为此接口分三层：客户端使用代理类／接口；Rust 模块使用明确构造依赖和方法；平台及硬件实现窄 trait。网络只传有版本的数据、命令、身份和回执，不传类实例、内存地址或驱动句柄。

本目录的 TypeScript 是接口描述与客户端调用草案，不是用 TS 实现灯光引擎。核心语言约束与 C++26 比较见[专项复评](../core-language-rust-vs-cpp.md)；权威语义和输出关键路径继续使用 Rust。

2026-09-24 新增独立的[AI 辅助编辑扩展 draft-1](assisted-editing.md)，依据 [ADR-011](../development/decisions/PRODUCT-ADR-011-assisted-editing.md)。复用现有工程命令，补能力发现、受限上下文、不可变提案、范围内连续编辑及撤销；未改主协议／StageClient，服务和模型尚未接入。

## 直接从哪里看

[应用层安全会话](device-session.md)：有界 Noise IK 核心、双方确认和加密保活已实现并测过 ESP32 板内资源；密钥证明与云端权限分离，正式 GATT／安装接入待完成。

[安全记录字节通道](secure-record-channel.md)：无堆分片重组、持续序号、背压／发送确认、固定期限与关闭；支持 20～244 B 预算，已与真实 Noise 软件组合验证，尚未接无线适配。

[主机设备连接](device-connection.md)：独立原生蓝牙适配、应用级连接服务与正式中文设备面板已接入；取消／陈旧操作／回执和超时保护软件通过，原生实板验收待系统权限完成，诊断不等于节目控制。

已实现的[设备运行应用层](device-runtime.md)：待选／载入／运行分离、单控制权、幂等回执、断线继续和确认静默后维护；无传输／物理驱动依赖，许可调用不等于生产授权。

已实现的[NOR 包存储](nor-package-store.md)：物理擦写粒度、两槽封印／撕裂恢复、租约和受控维护入口；ESP32 驱动构建与预算已核对，实板安装／断电另验收。

已实现的[播放包传输](package-transfer.md)：SMP 应用组、固定缓冲重组、受限安装服务、主机协调、断连续传／取消及意图终态；不把诊断会话当权限，不重放已完成意图。

已实现的[播放包安装事务](package-installation.md)：no_std 状态机、两槽提交／恢复、重传／取消／待确认和稳定读源；文件参考适配与故障验收通过，NOR 适配已补，独立运行层已补，正式 GATT／实板运行仍另行接入。

当前工程的[配灯表导出](patch-report.md)：Rust 快照投影、CSV 成熟编码与冲突保护保存；TS 只发起及展示交接结果。

当前工程的[容量与编码](project-capacity.md)：打开／编辑／保存／恢复统一按完整紧凑内容限制，必要时切换排版；含修订预留与超限保护。

已实现的[独立播放包](playback-package.md)：工程编译适配、no_std 有界 CBOR 容器、逐节目装载、资源报告和桌面原子导出。软件参考包与设备安装／授权分开。

当前已实现的本机接口另见[列表预览](sequence-preview.md)、[场景动态效果](lighting-effects.md)、[灯组与预设](editing-library.md)、[灯具建档与配适](fixture-authoring.md)。以下跨端服务声明仍为设计稿。

| 文件 | 内容 |
| --- | --- |
| [shared-contracts.ts](shared-contracts.ts) | 身份、时间、错误和运行上下文；不依赖业务 API，供各接口单向引用 |
| [contracts.ts](contracts.ts) | 可类型检查的客户端接口声明；具体参数、返回值、错误、修订与租约 |
| [examples.ts](examples.ts) | 本机／远程调用、编程到播放、影响预览、独立同步组定位、订阅和云端分发 |
| [external-contracts.ts](external-contracts.ts) | 外部系统、动作、控制租约、执行证据、监看源与本地呈现接口 |
| [external-examples.ts](external-examples.ts) | 外部预检／控制、超时对账、独立监看和本地监听的调用 |
| [surface-contracts.ts](surface-contracts.ts) | 硬件推子／旋钮／按键、映射、输入与灯／屏／电动推子反馈；宿主内部接口 |
| [evolution-examples.ts](evolution-examples.ts) | 未载入节目连接面板、独立工程导出、监看重连解析、手动外部控制 |
| [rust-modules.md](rust-modules.md) | 内部服务如何构造、哪些依赖可以注入、纯核心与硬件 trait 怎样调用 |
| [workflows.md](workflows.md) | 跨模块调用顺序、失败与超时处理、关闭和回收 |
| [contract-checks.ts](contract-checks.ts) | 应被类型系统拒绝的调用；不替代运行时验证 |
| [automation-contracts.ts](automation-contracts.ts) | AI／自动化窄编辑入口与可信宿主入口；范围、提案、回执和生命周期 |
| [automation-examples.ts](automation-examples.ts)／[automation-checks.ts](automation-checks.ts) | 修改所选场景亮度的调用、应用／撤销与越权／任意补丁编译期反例 |

专业渲染扩展见 [UE5 预演接口边界](../ue5-professional-previsualization.md)：补充 `PreviewRenderer` 的能力、准备、绑定只读来源、观察与释放生命周期。该扩展当前仅有设计，尚未加入 `contracts.ts` 或其类型检查范围。

[场地采集扩展](../venue-capture-design.md)另列采集导入、后台重建、场地修订准备／提交的伪调用职责，同样尚未加入 TS 声明。场地重建与节目执行分别运行，共享版本化资源和场景契约。

[多设备仿真扩展](../depence-r4-assessment.md)定义 `SimulationCatalog`、`SimulationSession` 和 `PlotService` 的职责及生命周期；设备响应、渲染和物理执行分开，声明和实现仍待补齐。

TS 声明用于当前设计评审。实际实现时，工程／会话／编译／现场控制 DTO 由 Rust 权威契约生成；云端业务契约由 TS 维护。迁移本草案到生成流程后，禁止分别手改两份接口。Rust 文档采用伪代码，省略具体库和部分类型定义，不能直接当成可编译 Rust。

共享类型独立放在 shared-contracts；外部控制与硬件接口不回引聚合的 StageClient 文件。即使只是 TypeScript 类型，也避免模块相互导入；驱动实现依赖窄接口，不能借聚合入口获取其他服务。

## 调用者看到的形状

```ts
// 伪调用：connectStage / transport 在 contracts.ts 中只有声明。
const stage = must(await connectStage(new LocalIpcTransport({ endpoint })));
// 换成 RemoteTransport 后，下面的业务调用面相同。

const edit = must(await stage.projects.edit({ ...meta(ids), base, operations }));
const buildJob = must(await stage.builds.compile({ ...meta(ids), project: edit.project, binding, target, domainId }));
const artifact = await awaitJob(stage.jobs, buildJob);
const prepareJob = must(await stage.deployment.prepare({ ...meta(ids), artifact, expectedRun }));
const ready = await awaitJob(stage.jobs, prepareJob);
const activationJob = must(await stage.deployment.activate({ ...meta(ids), preparedId: ready.preparedId, expectedRun, when }));
const activation = await awaitJob(stage.jobs, activationJob);
// 检查 active、取得相应控制租约后，才调用 control.submit；见完整样例。
```

上段展示调用形状，辅助函数与输入上下文见 examples.ts；每次 `meta(ids)` 产生新的命令 ID，重试同一个逻辑操作才复用原 ID。**不能把 compile 的 JobRef 当成 artifact，也不能把 activate 的任务回执当成 ActiveRun。** 完整、可类型检查的调用在 examples.ts；实际项目应以该文件为参考。

调用面相同不代表同步延迟相同。远程调用需要身份验证、版本协商、超时和断线恢复；本机也采用异步应用接口，业务组件不检查自己是否在 Tauri、浏览器或某块主板上。

## 模块与方法索引

灯具定义后续扩展依据 [ADR-009](../development/decisions/PRODUCT-ADR-009-fixture-definition.md) 与[建档设计](../ui-design/fixture-definition-design.md)。档案编辑、修订应用和受控测试须另行形成精确契约；下列已有伪 API 不代表这些新能力已实现，也不授权普通客户端绕过输出仲裁发送原始帧。

| 模块 | 内部类／接口 | 客户端入口／代表方法 | 谁拥有状态 |
| --- | --- | --- | --- |
| 工程编辑 | `ProjectService` | `projects.create/edit/previewEdit/commitPreview/undo/save` | 单写者工程事务服务 |
| 选择与编程器 | `SessionService` | `sessions.openBlind/edit/snapshot/rebase/close` | 每个会话；通过不可变快照与工程服务交接 |
| 灯具档案 | `FixtureLibrary` | `fixtures.importProfile/describe` | 不可变档案版本库 |
| 现场配适 | `BindingService` | `bindings.create/edit` | 独立现场绑定版本 |
| 素材 | `AssetService` | `assets.ingest/inspect/attachReference` | 编排参考与场景内容库；不成为内置节目播放器 |
| 目标能力 | `TargetService` | `targets.domains/inspect` | 按执行域查询能力、游标与验证记录 |
| 逻辑／目标编译 | `ShowCompiler` / `BuildService` | `builds.compile` | 输入快照和不可变编译产物 |
| 准备与激活 | `PlanManager` | `deployment.prepare/activate/discard` | 资源预备、计划与激活协调状态 |
| 现场控制 | `ControlGateway` / `ControlAuthority` | `control.acquire/renew/submit/outcome/relinquish` | 控制租约；运行核心应用控制后的播放状态 |
| 灯光执行 | `RuntimeKernel` / `Mixer` | 不向客户端暴露 `tick` | 指定执行域的一份运行状态 |
| 时钟与同步组 | `ClockRegistry` / `TransportCoordinator` | 通过状态和 `control.submit(seek)` 接入 | 每个时钟映射及每个同步组 |
| 外部系统控制 | `ExternalCommandGateway` / `ExternalDeviceAdapter` | `external.inspect/preflight/acquire/submit/outcome/state` | 控制网关持租约和回执；外部系统持实际播放状态 |
| 监听／监看 | `MonitorService` / `MonitorPresenter` | `monitors.sources/open/state/close`＋客户端本地呈现 | 接收会话与本地耳机／画面分别管理 |
| 硬件控制面 | `SurfaceCoordinator` / `SurfaceDriver` | 宿主 attach/applyMapping/ingest/feedback/detach | 会话、映射／手势与输入反馈；不向普通 UI 开放原始注入 |
| 输出与仲裁 | `DmxEncoder` / `OutputPort` / `OutputArbiter` | 不向普通客户端开放原始帧发送 | 端口所有者与驱动队列 |
| 诊断与订阅 | `ObservationService` | `observe.snapshot/subscribe/explain` | 运行状态的只读投影与有界历史 |
| 预演 | `PreviewService` | `preview.open/seek/close` | 独立的预演执行上下文 |
| 保存与包导出 | `ProjectRepository` / `PackageService` | `projects.save`、`exports.project/deployment` | 工程交换与目标执行包分开 |
| 资源传输 | `TransferService` | `transfers.upload` | 后台数据通路；不占用编辑或运行线程 |
| 云端分发 | `PublicationService` / `DistributionService` | 独立 `CloudClient` 的 upload/publish/assign/inspectDevice | Fastify 业务、数据库、对象存储 |
| 操作追踪 | `ReceiptStore` / `JobRegistry` | `requests.inspect`、`jobs.inspect/wait/requestCancel` | 有明确保留期限的操作／任务记录 |

表中逻辑类不要求逐项创建 crate、进程或微服务。`StageClient` 仅是客户端代理的组合，不是服务端共享所有状态的大对象。

## 公共调用规则

| 事项 | 约定 |
| --- | --- |
| 身份 | 对象 ID、命令 ID、服务实例 ID 分开；大整数计数器以字符串传输 |
| 时间 | `ClockInstant` 为指定 clockId／epoch 的单调纳秒；`MediaTime` 带时基；不能直接比较不同域 |
| 授权 | principal 由验证后的连接上下文派生，不信任客户端填写 actor；品牌类型不构成权限保护 |
| 基准修订 | 编辑带 ProjectRef 或 SessionRef；过期返回冲突，不静默覆盖 |
| 幂等 | 同作用域 commandId＋相同负载只接纳一次；同 ID 不同负载拒绝；保留期限明确 |
| 超时 | 表示未及时得到结果；可能已经执行，先 `requests.inspect` 对账 |
| 后台任务 | JobRef 只表示登记；读取 JobState 才知道成功／失败；取消请求不等于已经取消 |
| 现场控制 | ControlTicket 只表示接纳；`outcome` 返回 applied 才表示运行核心已应用，仍非硬件实测反馈 |
| 版本代次 | 每个方法校验自己需要的运行域／计划／播放／时钟／租约代次，不依赖一个全局 epoch |
| 错误 | 使用稳定 code、参数和重试政策；UI 根据 messageKey 中文显示 |
| 订阅 | 先快照再按游标续订；缺口显式要求重同步；关闭客户端订阅不停止现场 |
| 传输 | WireRequest 只装 JSON DTO；字段必须经过生成 schema 验证；方法来自封闭注册表 |
| 资源 | input/read-resource、prepared-plan 等 token 由拥有它的服务验证权限、节点作用域和有效期；跨节点不能直接解引用 |

`RpcTransport.request` 是客户端基础设施接口，业务组件不自行拼 method 字符串。RPC 边界负责 schema、大小限制和协议协商；trait 调用、IPC、网络适配只改变实现位置。媒体 PCM／视频帧走独立数据通路，不逐帧塞入 WireRequest。

`Subscription`、Promise、认证回调、JobRef 的类型参数只存在于本地代码；序列化时只保留规定的 DTO 字段。网络不会传递函数、迭代器或泛型类型。收到结果后按原操作的结果 schema 验证，再构造成客户端返回值。

`TargetSnapshot` 是执行能力概览，编译和准备服务按 targetId／domainId／executionCapabilityRevision 取权威完整能力；不能信任客户端的 evidence 或端点列表。监看和控制面能力独立版本化。详细格式与组合资源预算属于目标能力契约，未压缩成“支持视频”布尔值。

## 编程语义与状态交接

`ProjectService` 负责 Group、Preset、Cue、Effect、Recipe、Timeline 等对象的事务修改。它们是类型化操作／领域对象，不分别成为有独立数据库的远程服务。这样 Record、Cue Only、克隆等跨对象变更仍能原子校验和提交。

`SessionService` 不直接调用 ProjectService 的写方法；Record 使用服务签发的不可变 ProgrammerSnapshot。提交时检查 token、会话来源修订、当前工程和依赖是否兼容；否则先 rebase 再重新取快照。快照服务有期限和显式持有规则，不允许客户端伪造 token 内容。

`sessions.edit` 返回 SessionEditReceipt，分开会话已应用与 Live 贡献接纳结果：盲编、已接纳的 ControlTicket，或未现场应用及原因。不能仅返回新会话版本就宣称真实输出已改变；已接纳时仍需查询 applied。

计划按“逻辑计划 → 目标产物 → 已准备资源 → ActiveRun”推进。Prepare 不开始演出，Activate 不自动 Go。激活可以更换已有运行计划，因而需要相应现场权限和状态迁移政策；调用者不能把它当作无副作用查询。

0.3 的 compile／prepare／activate 仅覆盖明确的一个执行域；跨节点协调另收集各域结果。OperatorSessionRef 是独立身份会话，硬件 attach 和手动外部控制可在没有 ActiveRun 时发生；它本身不授予业务或输出权限。RuntimeSnapshot 允许 idle，空节点不必伪造运行计划。

持久 MonitorBindingSpec 保存稳定 MonitorSourceKey，使用时 resolve 为带代次的 MonitorSourceRef；监看布局不参与灯光执行 Route。工程包可直接供其他编辑端读取；执行包带指定域 BuildArtifact，只有 PublishedDeployment 可 assign，服务端再次核验真实包清单。

ControlLease 是用户／控制器操作某个范围的授权；OutputLease 是运行服务占有物理端口的授权。归还前者不等于停止节目或交接物理接口。租约到期／断连处理由对应控制模式规定，不能由网络适配器随意 Release 全场。

## 伪 API 的覆盖范围

本版给所有主要模块提供调用边界，并以首批类型化操作串通调用。`EffectDraft` 当前展示 steps 类型；颜色空间、完整 Phaser 节点图、Part／Block、MIB、更多功能值、复杂屏幕映射等按独立 schema 扩展，不声称在本版已经全部定义。

新增能力至少补：请求／返回 DTO、权限、作用域、依赖、版本、失败政策、时间含义及正反例。不能用 `any`、任意 JSON patch 或 `execute(string)` 绕过未定义语义。合法扩展走受校验的 schema 和能力表，未知必需执行语义拒绝发布。

外部动作的 content／preset／parameter ID 来自服务端目录，绑定系统上下文；参数 schema 与目标能力运行时核验。`preflight` 仅只读查询；peer-acknowledged 与 state-reported 不等于现场已出声／出画。监看 token 只可接收，MonitorPresenter 是客户端本地对象，不能序列化成可操作远端扬声器的接口。硬件控件／目标／behavior 的组合也须语义校验，类型检查本身不证明可以实际绑定。

节目外部动作的租约绑定 TransportCursor，发送前检查同步组代次和截止时间；manual 仅用于经过授权的独立手动操作。连接的 services 是实际可用服务及其契约版本，聚合 StageClient 不要求所有节点实现全部服务；缺服务返回明确能力错误。

仍未完整声明：时钟映射与核心控制命令 deadline、分支同步／资源依赖清单、监看布局存储、场景／仿真 DTO、跨节点部署协调。它们有明确的原型／格式冻结门槛，见再评估 C01—C10；本目录不是覆盖全部未来能力的最终 SDK。

在数据契约稳定后，再生成实际 Rust／TS 代码和传输注册表。本目录不会在根 workspace 中预建一堆未实现服务或加入播放器依赖。

## 检查与下一步

类型检查命令（复用项目已安装的开发编译器，不新增应用依赖；包含独立辅助编辑扩展）：

```sh
apps/ui-prototype/node_modules/.bin/tsc -p docs/module-api/tsconfig.json
```

该检查验证接口与样例能匹配，并拒绝列出的错误调用。它不能证明 RPC 可连接、鉴权正确、真实帧输出、运行无分配或商业可用。Rust 伪接口也尚未编译或实现。

实现顺序建议：公共值／身份与错误 → 工程／会话及事务 → 模拟适配与编译／准备 → 现场命令和状态 → 持久化／恢复 → UI 代理 → 外部系统联调、监听／监看及真实控制面 → 云端传输。各阶段沿着相同调用链替换模拟实现，并覆盖架构 A/B 场景；跨端和硬件契约现在保留，不等待全部功能实现才补接口。

- [真实场地编辑模块](stage-spaces.md)：空间／构件／灯位及原子命令（PREVIS-001 实施中）。
- [摇头灯指向与安装变换](positioning.md)：已接入档案、静态共同对焦、角度／零偏、量化编码与 UE 姿态的运行接口；连续轨迹仍为后续边界。
