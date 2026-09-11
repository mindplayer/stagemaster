# Rust 内部模块与平台伪接口

这是 Rust 风格的设计伪代码，不参与 Cargo 编译。`new(...)` 展示必须注入的依赖；`&T` 表示只读借用，`&mut T` 表示唯一状态所有者。示例中的命名类型是协议／领域契约占位，布局、具体库和 trait object 形式在实现时确定。

2026-09-11：音视频改为外部系统执行、本机监听／监看；双向硬件控制面独立接入。下述外部与监看 trait 替代旧版内置 MediaBackend／PreparedMedia 播放职责。

同日 0.3 修订：操作会话与 ActiveRun 分离；执行域、外部同步组代次、两类包和节点角色遵守[再评估](../architecture-evolution-review.md)。以下全功能组装只展示依赖关系；实际宿主按能力注册所需服务，纯执行／监看节点不要求装齐编辑器与所有后端。

调用环境标记：**A** 应用／控制线程，可执行有界业务操作；**W** 工作线程，可异步 I/O 或分配；**R** 实时关键路径，禁止无界分配、等待网络／文件／UI。跨进程传输只在外层代理完成，下面的原生引用不能序列化出去。

## 1. 谁负责组装对象

```rust
// A/W：仅宿主入口有权组装模块；不是共享全局 ServiceLocator。
let artifacts = ImmutableArtifacts::new(blob_store, retention_policy);
let receipts = ReceiptStore::new(receipt_repository, retention_policy);
let projects = ProjectService::new(project_store, artifacts.read_port(), receipts.writer());
let sessions = SessionService::new(session_store, projects.snapshot_reader(), artifacts.writer());
let fixtures = FixtureLibrary::new(profile_store, importer_registry);
let bindings = BindingService::new(binding_store, fixtures.reader(), target_probe.reader());
let compiler = ShowCompiler::new(semantic_registry, fixtures.reader());
let builds = BuildService::new(compiler, artifacts.read_port(), artifact_store, worker_pool);
let control = ControlGateway::new(control_authority, plan_index_reader, rt_command_writer, receipts.writer());
let plans = PlanManager::new(artifact_store.reader(), target_probe, external_capability_reader, output_arbiter, activation_queue);
let observe = ObservationService::new(runtime_snapshot_reader, source_map_reader, bounded_history);
let external = ExternalCommandGateway::new(external_registry, external_authority, bounded_external_queue, receipts.writer());
let monitors = MonitorService::new(source_catalog, monitor_receiver_factory, viewer_registry, monitor_budget);
let surfaces = SurfaceCoordinator::new(surface_drivers, surface_profiles, semantic_control_port, feedback_snapshot_reader);

// RPC 注册的是生成的、版本化应用入口；认证上下文由宿主建立。
let application = ApplicationRoutes::new(projects, sessions, fixtures, bindings, builds, control, plans, observe, external, monitors);
let server = HostServer::new(authenticator, protocol_registry, application);
```

这是构造依赖图，未展示所有存储／队列实例的创建。SessionService 读不可变工程快照，ProjectService 读不可变 ProgrammerSnapshot；双方不互调写方法，不同时持有对方的可变锁。工厂、仓库、队列的实现由宿主选择，核心只依赖窄接口。

`reader()`／`writer()`／`snapshot_reader()` 在示例中表示宿主创建的、权限受限的访问端口，不是暴露内部对象引用；实际接口需分别定义可读的快照类型和唯一写入者。

UI 的 StageClient 与这里的服务对象是不同对象。桌面 IPC 和 Web 网关注册同一应用入口，验证后调用 A 层；不会远程暴露 `RuntimeKernel::tick` 或 `OutputPort::try_submit`。

## 2. 工程、会话与领域操作

```rust
struct ProjectService { /* 唯一工程写入者；无公开可变字段 */ }
impl ProjectService {
    fn new(store: ProjectStore, snapshots: SnapshotReader, receipts: ReceiptWriter) -> Self;
    fn edit(&mut self, actor: &VerifiedActor, request: EditRequest) -> Result<EditReceipt>; // A
    fn preview_edit(&self, actor: &VerifiedActor, request: EditRequest) -> Result<ChangePreview>; // A/W
    fn commit_preview(&mut self, actor: &VerifiedActor, request: CommitPreview) -> Result<EditReceipt>; // A
    fn undo(&mut self, actor: &VerifiedActor, request: UndoRequest) -> Result<EditReceipt>; // A
    fn save(&self, actor: &VerifiedActor, revision: ProjectRef) -> Result<JobRef<SaveReceipt>>; // A→W
    fn snapshot(&self, revision: ProjectRef) -> Result<ProjectSnapshotLease>; // A
}

// A：只在事务沙盒内修改，成功才替换权威工程。
impl ShowTransaction {
    fn apply(&mut self, operation: EditOperation, refs: &ResolvedInputs) -> Result<()>;
    fn check_invariants(&self, schema: &SemanticRegistry) -> Result<ImpactReport>;
    fn finish(self) -> ValidatedChange;
}

struct SessionService { /* 每会话选择、Programmer、过滤和绑定状态 */ }
impl SessionService {
    fn new(store: SessionStore, projects: ProjectSnapshotReader, snapshots: SnapshotWriter) -> Self;
    fn open_blind(&mut self, actor: &VerifiedActor, project: ProjectRef) -> Result<SessionRef>; // A
    fn edit(&mut self, actor: &VerifiedActor, command: SessionEdit) -> Result<SessionEditReceipt>; // A
    fn snapshot(&self, actor: &VerifiedActor, expected: SessionRef) -> Result<ProgrammerSnapshotLease>; // A
    fn rebase(&mut self, actor: &VerifiedActor, old: SessionRef, project: ProjectRef) -> Result<SessionRef>; // A
    fn close(&mut self, actor: &VerifiedActor, id: SessionId) -> Result<CloseReceipt>; // A
}
```

调用方式：`projects.edit(actor, EditRequest { base, operations })`；不允许调用者拿到 `project.cues[0]` 后任意修改。Preset／Cue／Effect 等以类型化操作和纯领域函数协作，不另建互相远程调用的对象服务。

Record 的 `ResolvedInputs` 固定 ProgrammerSnapshot、灯具档案与引用版本。失败原子回滚内存事务；保存是另一个明确的 durable 确认点。不同修订的快照不能未经检查混合使用。snapshot lease 退出／到期后由工作层回收，不能让过期客户端 token 保持无限引用。

Live Programmer 使用控制授权绑定运行域；SessionService 校验并构造精简贡献更新，通过 ControlGateway 送入运行核心。只有已提交的会话版本可产生贡献；相同版本不能重复叠加。无法接纳现场贡献时，报告未现场应用状态，而非假装已出光。

## 3. 灯具、绑定、素材和目标能力

```rust
trait FixtureLibraryRead {
    fn profile(&self, reference: ProfileRef) -> Result<ProfileSnapshotLease>; // A/W
    fn validate_address(&self, profile: &ProfileSnapshot, element: ElementId, key: AttributeKey) -> Result<AttributeDescriptor>; // A
}
impl FixtureLibrary {
    fn import(&self, actor: &VerifiedActor, resource: InputResource) -> Result<JobRef<ProfileRef>>; // A→W
}
impl BindingService {
    async fn edit(&mut self, actor: &VerifiedActor, base: BindingRef, project: ProjectRef, routes: RouteChanges) -> Result<BindingRef>; // W，完成配置持久提交
    fn resolve(&self, binding: BindingRef) -> Result<ResolvedBindingLease>; // W
}
impl AssetService {
    fn ingest(&self, actor: &VerifiedActor, source: InputResource) -> Result<JobRef<AssetRef>>; // A→W
    fn attach_reference(&self, asset: AssetRef, resource: InputResource, purpose: ReferencePurpose) -> Result<JobRef<DerivedAssetRef>>; // W，仅编排参考
    fn pin(&self, assets: &[AssetRef], owner: ResourceOwner) -> Result<AssetPinSet>; // W
}
trait TargetProbe {
    fn inspect(&self, target: TargetId) -> Result<CapabilitySnapshot>; // W
    fn admit(&self, requirements: &TargetRequirements, binding: &ResolvedBinding, policy: &AdmissionPolicy) -> Result<AdmissionReport>; // W
}
```

配适以灯具连接段为单位，支持多连接段，而非强制每个灯具只占一个平面地址区。TargetProbe 需要完整格式与组合资源信息；客户端概览不替代该信息。版本或端点已变化时 Prepare 重新检查。

BindingService 的 create/edit 成功返回代表独立绑定版本已持久提交，且不直接更改当前运行路由。其 I/O 位于工作层；修改经计划准备与激活才影响现场。这个确认语义与 ProjectService.edit 的内存事务确认不同，需要在客户端明确区分。

本服务只管理执行绑定；监看布局与硬件档案独立存储，稳定逻辑键由各目录解析到临时连接。它们的重连不修改灯光计划，除非节目明确将对应反馈能力列为执行依赖。

资源 token 由所属节点解析；手机本地文件需要通过授权传输导入该服务，不能把手机文件路径传到盒子直接打开。AssetRef 是内容身份，InputResource 是临时访问能力，两者用途不同。

## 4. 编译器与计划管理

```rust
impl ShowCompiler {
    fn compile_logical(&self, show: &ProjectSnapshot, profiles: &PinnedProfiles, policy: &SemanticPolicy) -> Result<LogicalPlan>; // W，纯语义
    fn lower_target(&self, logical: &LogicalPlan, binding: &ResolvedBinding, caps: &CapabilitySnapshot) -> Result<TargetPlan>; // W
}
impl BuildService {
    fn compile(&self, actor: &VerifiedActor, input: PinnedCompileInput) -> Result<JobRef<BuildArtifact>>; // A→W
}
impl PlanManager {
    fn prepare(&mut self, actor: &VerifiedActor, build: BuildRef, expected: RunCursor) -> Result<JobRef<PreparedPlanToken>>; // A→W
    fn activate(&mut self, actor: &VerifiedActor, token: PreparedPlanToken, expected: RunCursor, when: ActivationTime) -> Result<JobRef<ActivationOutcome>>; // A
    fn discard(&mut self, actor: &VerifiedActor, token: PreparedPlanToken) -> Result<DiscardReceipt>; // A→W
    fn collect_retired(&mut self, acknowledgements: &[RetirementAck]) -> Result<ReclamationReport>; // W
}
```

编译固定工程／绑定／档案、外部对象引用与能力版本，输出 sourceMap。Prepare 完成本地容量与灯光输出预留，并检查外部对象／能力；对端的资源就绪单列证据，不能将“可发送命令”当作“媒体已预读”。预检不改变外部播放，token 固定版本与失效时间。

Activate 做比较并交换：只允许 expectedRun 仍匹配时排入切换边界，R 路径再检查必要代次以消除入队后竞争。准备失败／过期保留旧计划；跨执行域出现部分成功时返回各域实况，不声称全硬件原子切换。丢弃／过期由工作层释放准备资源，已激活计划不能被 `discard` 停掉。

## 5. 控制网关、时钟与实时核心

```rust
impl ControlAuthority {
    fn acquire(&mut self, actor: &VerifiedActor, run: ActiveRun, scope: ControlScope, policy: TakeoverPolicy) -> Result<ControlLease>; // A
    fn renew(&mut self, actor: &VerifiedActor, lease: ControlLease) -> Result<ControlLease>; // A
    fn relinquish(&mut self, actor: &VerifiedActor, lease: ControlLease) -> Result<()>; // A
}
impl ControlGateway {
    fn submit(&mut self, actor: &VerifiedActor, command: ControlRequest) -> Result<ControlTicket>; // A
    fn outcome(&self, actor: &VerifiedActor, ticket: ControlTicket) -> Result<ControlOutcome>; // A
}
trait ClockSource {
    fn observe(&self) -> Result<ClockObservation>; // 平台适配，调用频率与成本受契约限制
}
impl ClockRegistry {
    fn update_observation(&mut self, source: ClockId, sample: ClockObservation) -> Result<ClockMapping>; // A
    fn map_time(&self, time: ClockInstant, target: ClockId) -> Result<ClockInstant>; // A
}
impl TransportCoordinator {
    fn prepare_seek(&self, group: SyncGroupRef, position: MediaTime) -> Result<PreparedSeek>; // W
    fn schedule_seek(&mut self, seek: PreparedSeek, expected: TransportCursor) -> Result<ControlTicket>; // A
}

// A/W：构造时固定容量并预热，R 路径不处理 JSON、字符串 ID 或文件引用。
let kernel = RuntimeKernel::new(prepared_runtime_plan, preallocated_buffers);
impl RuntimeKernel {
    fn tick(&mut self, input: &TickInput, output: &mut TickOutput); // R
}
struct TickInput<'a> {
    clocks: &'a [MappedClockSample],
    commands: &'a [ValidatedRtCommand], // 有界；含紧凑索引、作用域代次、授权过期检查信息
}
struct TickOutput<'a> {
    values: &'a mut [ResolvedValue],
    receipts: &'a mut FixedReceiptBuffer,
    diagnostics: &'a mut FixedDiagnosticBuffer,
}
impl Mixer {
    fn resolve(&self, contributions: &[CompiledContribution], state: &MixState, output: &mut [ResolvedValue]); // R
}
```

网关处理鉴权、DTO、幂等、序号、索引转换与入队；没有“拿到 Rust trait 就绕过检查”的旁路。RtCommand 中不能残留对 UI／工程对象的引用。核心应用前检查必要的计划／控制权／时钟代次；控制权限在排队期间失效时拒绝并产生回执。

只有运行宿主调用 tick。音频回调、媒体解码、异步网络循环和 UI 刷新都不调用它。队列满由网关立即返回明确错误；连续电平可按对象合并，离散 Go 不能被覆盖。循环预算固定，不能无界消费直到队列为空。

Seek 的本地重算与外部能力检查在工作层准备，R 只应用已准备本地状态并投递有界动作；外部定位结果逐目标查询，不混同本地 applied。新 TransportGeneration 只影响目标组。长时间操作按有效期续租，断线不自动续租。

PinnedCompileInput 固定 domainId；PlanManager 只在该执行域切换逻辑计划。跨域／跨节点协调收集独立结果，不能将本地句柄交换解释为硬件或外部动作的原子提交。

## 6. 外部控制、监听／监看与输出 trait

```rust
trait ExternalDeviceAdapter {
    async fn describe(&self) -> Result<ExternalCapabilitySnapshot>; // W
    async fn preflight(&self, request: ExternalPreflight) -> Result<ExternalReadiness>; // W，只读
    async fn dispatch(&mut self, command: ValidatedExternalCommand) -> Result<ExternalSendReceipt>; // W
    async fn read_state(&self, target: ExternalTargetRef) -> Result<ExternalObservation>; // W
}
trait MonitorReceiverFactory {
    fn capabilities(&self) -> MonitorCapabilities; // W
    async fn open(&self, source: AuthorizedMonitorSource, viewer: VerifiedViewer, budget: MonitorBudget) -> Result<MonitorReceiver>; // W
}
impl MonitorReceiver {
    fn try_take(&mut self) -> Option<MonitorFrameLease>; // 仅监看数据，有界
    fn state(&self) -> MonitorState; // 接收状态，不冒充客户端已呈现
    async fn close(self) -> Result<MonitorCloseReceipt>; // W
}
struct FrameLease { /* 内存类型、平面布局、时戳、所有者、消费完成栅栏；不可序列化裸指针 */ }
trait MediaPreviewTap {
    fn try_take(&mut self) -> Option<PreviewFrameLease>; // 有界、可丢过期预览
}
trait OutputArbiter {
    fn reserve(&mut self, actor: &VerifiedActor, request: OutputReservation) -> Result<ReservationToken>; // W
    fn commit_handover(&mut self, reservation: ReservationToken, expected: OutputOwner) -> Result<OutputLease>; // 受控切换
}
impl DmxEncoder {
    fn encode(&self, mapping: &CompiledDmxMapping, values: &[ResolvedValue], frames: &mut [DmxPayload]); // 纯函数，可置于 R
}
trait OutputPort {
    fn try_submit(&mut self, frame: OwnedOutputFrame) -> Result<OutputAcceptance, RejectedFrame>; // 有界入队
    fn state(&self) -> OutputState; // 最近状态，不阻塞等待硬件
    async fn stop(&mut self, policy: DeviceStopPolicy) -> Result<StopReceipt>; // W
}
```

`try_submit` 的成功仅表示端口队列已接纳；写入接口和设备反馈另上报。失败返回 RejectedFrame，明确原帧所有权仍归调用方，避免泄漏或重复释放。此接口不能把阻塞系统发送藏在“try”名称下面；真实驱动发送由端口工作层执行。

灯光输出帧带运行域、计划代次、OutputLeaseGeneration、绑定版本和截止时间，端口检查所有权。监看帧带源身份／代次、时间与接收会话，不取得 OutputLease。外部控制租约、监看授权和灯光输出所有权不能互换。

本地监听使用受控采样缓冲，视频监看用协商的缓冲／纹理；零复制和回退复制都计入预算。回收不进入监听回调或灯光 tick。正式媒体播放和处理在外部软件，适配器不承担主节目混音／合成。ExternalCommandGateway 在真正发出前重查租约、上下文和截止时间；已发送但失去应答时报告未知，不盲目重发触发。

ValidatedExternalCommand 区分 manual／show，show 必须携带同步组和 TransportGeneration；驱动不能自行把过期自动动作转换成手动操作。数值合并与编码器增量不能越过离散命令屏障。

## 7. 换硬件时如何替换

```rust
// 由 build target 和宿主配置选择实现，不在 Cue / Effect 内判断 macOS 或某个主板。
let platform = LinuxPlatformServices::new(platform_config);
let monitors = BoardMonitorBackend::new(platform.device_catalog(), monitor_config);
let outputs = OutputPortFactory::new(platform.io_catalog(), port_registry);
let surfaces = BoardSurfaceDrivers::new(platform.input_catalog(), surface_config);
let host = RuntimeHost::assemble(core_modules, platform, monitors, surfaces, outputs, target_profile);

// 测试用相同 trait；能力明确来自测试场景，不冒充实机证据。
let host = RuntimeHost::assemble(core_modules, VirtualPlatform, FakeMonitorBackend, VirtualSurfaceDrivers, MemoryOutputPorts, simulated_profile);
```

这些类型是示意名称，不是已实现驱动。平台、监看、硬件控制面、输出与能力验证分别替换，未配置监看的宿主无需媒体后端。硬件控制面的输入处理和状态反馈见 surface-contracts.ts，不通过 UI 更新定时器推进。受限目标超出能力则拒绝或选择已定义产物。

## 8. 诊断、预演、存储与云端

```rust
impl ObservationService {
    fn snapshot(&self, actor: &VerifiedActor, run: RunCursor) -> Result<RuntimeSnapshot>; // A
    fn subscribe(&self, actor: &VerifiedActor, filter: SubscriptionFilter, cursor: Cursor) -> Result<SubscriptionToken>; // A
    fn explain(&self, actor: &VerifiedActor, frame: FrameRef, address: AttributeAddress) -> Result<Explanation>; // W
}
impl PreviewService {
    fn open(&mut self, project: ProjectRef) -> Result<PreviewToken>; // W
    fn seek(&mut self, token: PreviewToken, group: SyncGroupId, position: MediaTime) -> Result<PreviewState>; // W
    fn close(&mut self, token: PreviewToken) -> Result<()>; // W
}
trait ProjectRepository {
    async fn commit_snapshot(&self, snapshot: &ProjectSnapshot, expected_head: SavedHead) -> Result<DurableReceipt>; // W
    async fn recover(&self, project: ProjectId) -> Result<RecoveryReport>; // W
}
impl PackageService {
    fn export_project(&self, project: DurableProjectRef) -> Result<JobRef<LocalProjectPackage>>; // W
    fn export_deployment(&self, project: DurableProjectRef, build: BuildRef) -> Result<JobRef<LocalDeploymentPackage>>; // W
}
impl TransferService {
    fn upload(&self, source: AuthorizedReadResource, destination: UploadGrant) -> Result<JobRef<TransferReceipt>>; // W
}
```

PreviewService 构造时没有 OutputArbiter 或物理输出工厂，不能因调用了播放方法而获得真实输出权限。监视现场与重放预演是两个上下文。诊断历史已淘汰时返回 NOT_RETAINED，不编造旧帧来源。

仓库成功返回 DurableReceipt 才标记已保存；旧保存不能覆盖新 head。PackageService 固定控制计划、本地必需资源及外部依赖清单；参考代理不代表外部系统已经部署正式素材。跨系统资源交付分别确认，传输 grant 不写入节目或公开事件。

工程交换包只固定可编辑内容与资源；上段控制计划和现场绑定要求仅适用于执行包。通用资源的几何／外观／文档元数据按 schema 读取，不把所有输入当作媒体文件。分支同步及资源依赖清单仍需持久化原型细化。

Fastify 的云业务使用相同的明确构造方式：

```ts
const publications = new PublicationService(packageRepository, objectStore, rustValidationJobs);
const distribution = new DistributionService(assignmentRepository, deviceRegistry, publications.readPort());
// openUpload → 数据传输 → verifyUpload → publish → assign。
// 云端请求不调用 RuntimeKernel.tick，设备 Active 状态由设备上报。
```

云端类名也是伪接口。工作任务复用 Rust 校验／编译核心；项目引用或本地资源 token 不会让云端自动获得本机文件。具体调用链见 workflows.md。
