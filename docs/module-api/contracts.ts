/**
 * 伪 API 草案 0.3：用于调用设计和类型检查，没有服务实现。
 * Rust 领域／控制接口在实现阶段成为权威来源并生成客户端 DTO；
 * 本文件不作为另一套独立维护的工程语义，不规定最终传输编码。
 */
import type { ExternalAction, ExternalApi, ExternalTargetRef, ExternalTicket, MonitorApi } from "./external-contracts";
import type { ActiveRun, BindingRef, ClockInstant, Counter, Decimal, Id, Json,
  MediaTime, Meta, OperatorSessionRef, Problem, ProjectRef, Result, Rpc, RunCursor,
  ServiceContractRef, TransportCursor } from "./shared-contracts";
export * from "./shared-contracts";
export interface SessionRef {
  readonly sessionId: Id<"session">;
  readonly revision: Counter;
  readonly project: ProjectRef;
}
export interface ProfileRef {
  readonly profileId: Id<"profile">;
  readonly revisionId: Id<"profile-revision">;
  readonly modeId: Id<"mode">;
}
export interface AttributeAddress {
  readonly fixtureId: Id<"fixture">;
  readonly elementId: Id<"element">;
  readonly attribute: string; // 有命名空间的注册属性键；禁止任意未注册键进入计划。
}
export type Value =
  | { readonly kind: "normalized"; readonly value: Decimal }
  | { readonly kind: "physical"; readonly unit: "degree" | "kelvin"; readonly value: Decimal }
  | { readonly kind: "relative"; readonly unit: "degree" | "normalized"; readonly value: Decimal }
  | { readonly kind: "discrete"; readonly functionId: Id<"function">; readonly setId: Id<"set"> };
export type ValueSource = { readonly kind: "literal"; readonly value: Value } |
  { readonly kind: "preset"; readonly presetId: Id<"preset"> };
export interface Assignment { readonly address: AttributeAddress; readonly source: ValueSource; }
export interface JobRef<T> { readonly jobId: Id<"job">; readonly __result?: T; }
export type JobState<T> =
  | { readonly state: "queued" | "running"; readonly progress: number }
  | { readonly state: "succeeded"; readonly value: T }
  | { readonly state: "failed"; readonly error: Problem }
  | { readonly state: "cancelled" };
export interface JobsApi {
  inspect<T>(job: JobRef<T>): Rpc<JobState<T>>;
  wait<T>(job: JobRef<T>, options: { readonly timeoutMs: number }): Rpc<JobState<T>>;
  requestCancel<T>(job: JobRef<T>, meta: Meta): Rpc<{ readonly requested: boolean }>;
}
export type RequestRecovery =
  | { readonly state: "external"; readonly ticket: ExternalTicket }
  | { readonly state: "not-seen" | "processing" | "retention-expired" }
  | { readonly state: "job"; readonly method: string; readonly jobId: Id<"job">; readonly resultSchema: string }
  | { readonly state: "control"; readonly ticket: ControlTicket }
  | { readonly state: "completed"; readonly method: string; readonly result: Json };
export interface RequestsApi { inspect(commandId: Id<"command">): Rpc<RequestRecovery>; }
export interface SessionSnapshot {
  readonly token: Id<"programmer-snapshot">;
  readonly source: SessionRef;
}
export interface EffectDraft {
  readonly kind: "steps"; // 类型标签从草案阶段保留；后续效果类型单独增加 schema 与能力。
  readonly effectId: Id<"effect">;
  readonly name: string;
  readonly groupId: Id<"group">;
  readonly attribute: string;
  readonly steps: readonly { readonly value: Value; readonly duration: MediaTime }[];
  readonly cycles: Counter | "infinite";
}
export interface RecipeDraft {
  readonly recipeId: Id<"recipe">;
  readonly groupId: Id<"group">;
  readonly presetId: Id<"preset">;
  readonly effectId?: Id<"effect">;
  readonly selection: "follow-group" | "freeze-at-commit";
}
/** 只用于编排参考，不创建本机正式音视频播放。 */
export interface ReferenceClipDraft {
  readonly clipId: Id<"clip">;
  readonly assetId: Id<"asset">;
  readonly streamId: string;
  readonly sourceIn: MediaTime;
  readonly sourceOut: MediaTime;
  readonly start: MediaTime;
  readonly purpose: "editing-reference";
}
export interface ExternalEventDraft {
  readonly eventId: Id<"external-event">;
  readonly deviceId: Id<"external-device">;
  readonly at: MediaTime;
  readonly action: ExternalAction;
  readonly onSeek: "skip" | "reconcile-state"; // 只有可对账的状态动作允许后者；不重放离散触发。
  readonly ifLate: "skip" | "fail-group";
}
export interface TimelineDraft {
  readonly timelineId: Id<"timeline">;
  readonly syncGroupId: Id<"sync-group">;
  readonly references: readonly ReferenceClipDraft[];
  readonly externalEvents: readonly ExternalEventDraft[];
}
/** 首批有类型的编辑操作；未列高级语义必须显式返回能力不足，不接受任意 JSON patch。 */
export type EditOperation =
  | { readonly kind: "add-fixture"; readonly fixtureId: Id<"fixture">; readonly name: string; readonly profile: ProfileRef }
  | { readonly kind: "create-group"; readonly groupId: Id<"group">; readonly name: string; readonly fixtures: readonly Id<"fixture">[] }
  | { readonly kind: "create-preset"; readonly presetId: Id<"preset">; readonly name: string; readonly values: readonly Assignment[] }
  | { readonly kind: "create-sequence"; readonly sequenceId: Id<"sequence">; readonly name: string }
  | { readonly kind: "record-cue"; readonly sequenceId: Id<"sequence">; readonly cueId: Id<"cue">; readonly number: string;
      readonly name: string; readonly programmer: SessionSnapshot; readonly values: "retain-references" | "bake-literals" }
  | { readonly kind: "update-cue"; readonly sequenceId: Id<"sequence">; readonly cueId: Id<"cue">;
      readonly mode: "tracking" | "cue-only"; readonly values: readonly Assignment[] }
  | { readonly kind: "upsert-effect"; readonly effect: EffectDraft }
  | { readonly kind: "upsert-recipe"; readonly recipe: RecipeDraft }
  | { readonly kind: "upsert-timeline"; readonly timeline: TimelineDraft }
  | { readonly kind: "create-external-device"; readonly deviceId: Id<"external-device">; readonly name: string }
  | { readonly kind: "create-video-surface"; readonly surfaceId: Id<"video-surface">; readonly name: string;
      readonly width: number; readonly height: number } // 监看虚拟画布的像素尺寸；物理几何由场景服务描述，不创建正式视频输出。
  | { readonly kind: "configure-playback"; readonly playbackId: Id<"playback">; readonly sequenceId: Id<"sequence"> }
  | { readonly kind: "rename-object"; readonly objectId: string; readonly name: string }
  | { readonly kind: "delete-object"; readonly objectId: string; readonly policy: "reject-dependents" };
export interface EditRequest extends Meta { readonly base: ProjectRef; readonly operations: readonly EditOperation[]; }
export interface EditReceipt {
  readonly project: ProjectRef;
  readonly undoToken: Id<"undo">;
  readonly durability: "pending" | "durable";
  readonly changedObjects: readonly string[];
}
export interface ChangePreview {
  readonly token: Id<"change-preview">;
  readonly base: ProjectRef;
  readonly affectedObjects: readonly string[];
  readonly problems: readonly Problem[];
}
export interface ProjectApi {
  create(request: Meta & { readonly name: string }): Rpc<ProjectRef>;
  open(request: Meta & { readonly resource: Id<"input-resource"> }): Rpc<ProjectRef>;
  inspect(projectId: Id<"project">): Rpc<{ readonly current: ProjectRef; readonly saved: ProjectRef | null }>;
  edit(request: EditRequest): Rpc<EditReceipt>;
  previewEdit(request: EditRequest): Rpc<ChangePreview>;
  commitPreview(request: Meta & { readonly preview: Id<"change-preview">; readonly expected: ProjectRef }): Rpc<EditReceipt>;
  undo(request: Meta & { readonly base: ProjectRef; readonly undoToken: Id<"undo"> }): Rpc<EditReceipt>;
  save(request: Meta & { readonly project: ProjectRef }): Rpc<JobRef<{ readonly saved: ProjectRef }>>;
  allowedOperations(request: { readonly project: ProjectRef; readonly objectIds: readonly string[] }): Rpc<readonly string[]>;
}
export type SessionOperation =
  | { readonly kind: "select-group"; readonly groupId: Id<"group"> }
  | { readonly kind: "set-attribute"; readonly attribute: string; readonly value: Value }
  | { readonly kind: "apply-preset"; readonly presetId: Id<"preset"> }
  | { readonly kind: "clear-selection" | "deactivate" | "release-programmer" };
export interface SessionEditReceipt {
  readonly session: SessionRef;
  readonly liveApplication:
    { readonly kind: "blind" } |
    { readonly kind: "accepted"; readonly ticket: ControlTicket } |
    { readonly kind: "not-applied"; readonly error: Problem };
}
export interface SessionApi {
  openBlind(request: Meta & { readonly project: ProjectRef }): Rpc<SessionRef>;
  inspect(sessionId: Id<"session">): Rpc<SessionRef>;
  edit(request: Meta & { readonly base: SessionRef; readonly operations: readonly SessionOperation[] }): Rpc<SessionEditReceipt>;
  snapshot(session: SessionRef): Rpc<SessionSnapshot>;
  rebase(request: Meta & { readonly base: SessionRef; readonly project: ProjectRef }): Rpc<SessionRef>;
  close(request: Meta & { readonly sessionId: Id<"session"> }): Rpc<{ readonly closed: boolean }>;
}
export interface FixtureLibraryApi {
  importProfile(request: Meta & { readonly resource: Id<"input-resource"> }): Rpc<JobRef<ProfileRef>>;
  describe(profile: ProfileRef): Rpc<{
    readonly profile: ProfileRef;
    readonly elements: readonly { readonly id: Id<"element">; readonly attributes: readonly string[] }[];
    readonly connections: readonly { readonly id: Id<"fixture-connection">; readonly protocol: "dmx"; readonly footprint: number }[];
  }>;
}
export interface AssetInfo {
  readonly assetId: Id<"asset">;
  readonly contentHash: string;
  readonly kind: "audio" | "video" | "image" | "geometry" | "captured-appearance" | "document" | "bundle";
  readonly metadataSchema: string; // 非媒体资源可没有 streams；具体元数据按已注册 schema 查询／验证。
  readonly streams: readonly { readonly id: string; readonly kind: "audio" | "video"; readonly duration: MediaTime }[];
}
export interface AssetsApi {
  ingest(request: Meta & { readonly resource: Id<"input-resource"> }): Rpc<JobRef<AssetInfo>>;
  inspect(assetId: Id<"asset">): Rpc<AssetInfo>;
  attachReference(request: Meta & { readonly assetId: Id<"asset">; readonly resource: Id<"input-resource">;
    readonly purpose: "waveform" | "thumbnail" | "preview-video" }): Rpc<JobRef<Id<"derived-asset">>>;
}
export type Route =
  | { readonly kind: "dmx"; readonly fixtureId: Id<"fixture">; readonly connectionId: Id<"fixture-connection">; readonly endpointId: Id<"endpoint">; readonly universe: number; readonly startAddress: number }
  | { readonly kind: "external-control"; readonly deviceId: Id<"external-device">; readonly target: ExternalTargetRef };
export interface BindingsApi {
  /** 成功返回前独立绑定版本已持久提交；不会直接改变现场路由。 */
  create(request: Meta & { readonly projectId: Id<"project">; readonly targetId: Id<"target"> }): Rpc<BindingRef>;
  edit(request: Meta & { readonly base: BindingRef; readonly project: ProjectRef; readonly routes: readonly Route[] }): Rpc<BindingRef>;
}
export interface TargetSnapshot {
  readonly targetId: Id<"target">;
  readonly domainId: Id<"execution-domain">;
  readonly executionCapabilityRevision: Id<"execution-capability-revision">;
  readonly run: RunCursor;
  readonly endpoints: readonly { readonly id: Id<"endpoint">; readonly kind: "dmx" | "external-control" }[];
  readonly evidence: "reported" | "probed" | "tested-combination";
}
export interface TargetsApi {
  domains(targetId: Id<"target">): Rpc<readonly RunCursor[]>;
  inspect(request: { readonly targetId: Id<"target">; readonly domainId: Id<"execution-domain"> }): Rpc<TargetSnapshot>;
}
export interface BuildArtifact {
  readonly buildId: Id<"build">;
  readonly source: ProjectRef;
  readonly binding: BindingRef;
  readonly targetId: Id<"target">;
  readonly domainId: Id<"execution-domain">;
  readonly executionCapabilityRevision: Id<"execution-capability-revision">;
  readonly semanticVersion: string;
}
export interface BuildsApi {
  compile(request: Meta & { readonly project: ProjectRef; readonly binding: BindingRef;
    readonly target: TargetSnapshot; readonly domainId: Id<"execution-domain"> }): Rpc<JobRef<BuildArtifact>>;
}
export interface PreparedPlan {
  readonly preparedId: Id<"prepared-plan">;
  readonly artifact: BuildArtifact;
  readonly expectedRun: RunCursor;
  readonly expiresAt: ClockInstant;
}
/** 单执行域切换的结果；跨节点编排逐域收集回执，不伪装分布式原子激活。 */
export interface ActivationOutcome { readonly state: "active"; readonly run: ActiveRun; }
export interface DeploymentApi {
  prepare(request: Meta & { readonly artifact: BuildArtifact; readonly expectedRun: RunCursor }): Rpc<JobRef<PreparedPlan>>;
  activate(request: Meta & { readonly preparedId: Id<"prepared-plan">; readonly expectedRun: RunCursor;
    readonly when: { readonly kind: "next-boundary" } | { readonly kind: "at"; readonly instant: ClockInstant } }): Rpc<JobRef<ActivationOutcome>>;
  discard(request: Meta & { readonly preparedId: Id<"prepared-plan"> }): Rpc<{ readonly discarded: boolean }>;
}
export type ControlScope = { readonly kind: "playback"; readonly playbackId: Id<"playback"> } |
  { readonly kind: "programmer"; readonly sessionId: Id<"session"> } |
  { readonly kind: "sync-group"; readonly syncGroupId: Id<"sync-group"> };
export interface ControlLease {
  readonly token: Id<"control-lease">;
  readonly run: ActiveRun;
  readonly scope: ControlScope;
  readonly expiresAt: ClockInstant;
}
export type ControlAction =
  | { readonly kind: "go"; readonly playbackId: Id<"playback"> }
  | { readonly kind: "set-level"; readonly playbackId: Id<"playback">; readonly level: Decimal }
  | { readonly kind: "release-playback"; readonly playbackId: Id<"playback">; readonly fade: MediaTime }
  | { readonly kind: "seek"; readonly transport: TransportCursor; readonly position: MediaTime }
  | { readonly kind: "bind-programmer"; readonly session: SessionRef; readonly onDisconnect: "release-contribution" | "hold-until-expiry" };
export interface ControlTicket { readonly commandId: Id<"command">; readonly runtimeId: Id<"runtime-instance">; }
export type ControlOutcome =
  | { readonly state: "accepted" | "scheduled" }
  | { readonly state: "applied"; readonly at: ClockInstant }
  | { readonly state: "rejected"; readonly error: Problem }
  | { readonly state: "unknown" };
export interface ControlApi {
  acquire(request: Meta & { readonly run: ActiveRun; readonly scope: ControlScope; readonly takeover: "deny-if-owned" | "request-explicit-takeover" }): Rpc<ControlLease>;
  renew(request: Meta & { readonly lease: ControlLease }): Rpc<ControlLease>;
  relinquish(request: Meta & { readonly lease: ControlLease }): Rpc<{ readonly relinquished: boolean }>;
  submit(request: Meta & { readonly lease: ControlLease; readonly sequence: Counter; readonly action: ControlAction }): Rpc<ControlTicket>;
  outcome(ticket: ControlTicket): Rpc<ControlOutcome>;
}
export interface FrameRef { readonly run: ActiveRun; readonly frameSequence: Counter; }
export interface ActiveRuntimeSnapshot {
  readonly state: "active";
  readonly cursor: Id<"subscription-cursor">;
  readonly run: ActiveRun;
  readonly transports: readonly TransportCursor[];
  readonly latestFrame: FrameRef | null;
  readonly playbacks: readonly PlaybackView[];
}
export interface IdleRuntimeSnapshot {
  readonly state: "idle";
  readonly cursor: Id<"subscription-cursor">;
  readonly run: RunCursor & { readonly planGeneration: null };
  readonly transports: readonly [];
  readonly latestFrame: null;
  readonly playbacks: readonly [];
}
export type RuntimeSnapshot = IdleRuntimeSnapshot | ActiveRuntimeSnapshot;
export interface PlaybackView {
  readonly playbackId: Id<"playback">;
  readonly state: "released" | "running" | "paused" | "transitioning";
  readonly selectedCueId: Id<"cue"> | null;
  readonly level: Decimal;
}
export type RuntimeChange = { readonly kind: "snapshot"; readonly snapshot: RuntimeSnapshot } |
  { readonly kind: "delta"; readonly base: Id<"subscription-cursor">; readonly next: Id<"subscription-cursor">; readonly playbacks: readonly PlaybackView[] } |
  { readonly kind: "resync-required" };
export interface Subscription<T> { readonly events: AsyncIterable<T>; close(): Promise<void>; }
export interface ObserveApi {
  snapshot(run: RunCursor): Rpc<RuntimeSnapshot>;
  subscribe(request: { readonly run: RunCursor; readonly after: Id<"subscription-cursor"> }): Rpc<Subscription<RuntimeChange>>;
  explain(request: { readonly frame: FrameRef; readonly address: AttributeAddress }): Rpc<{
    readonly value: Value;
    readonly causes: readonly { readonly objectId: string; readonly role: "winner" | "suppressed" | "default" | "limited" }[];
  }>;
}
export interface PreviewRef { readonly previewId: Id<"preview">; readonly project: ProjectRef; }
export interface PreviewApi {
  open(request: Meta & { readonly project: ProjectRef }): Rpc<PreviewRef>;
  seek(request: Meta & { readonly preview: PreviewRef; readonly syncGroupId: Id<"sync-group">; readonly position: MediaTime }): Rpc<PreviewRef>;
  close(request: Meta & { readonly preview: PreviewRef }): Rpc<{ readonly closed: boolean }>;
}
export interface PackageManifestBase {
  readonly packageId: Id<"package">;
  readonly source: ProjectRef;
  readonly manifestHash: string;
  readonly blobs: readonly { readonly hash: string; readonly bytes: Counter }[];
}
export interface ProjectPackageManifest extends PackageManifestBase {
  readonly kind: "project";
  readonly projectFormatVersion: string;
}
export interface DeploymentPackageManifest extends PackageManifestBase {
  readonly kind: "deployment";
  readonly build: BuildArtifact;
}
export type PackageManifest = ProjectPackageManifest | DeploymentPackageManifest;
export interface LocalPackage<M extends PackageManifest = PackageManifest> {
  readonly manifest: M; readonly resource: Id<"read-resource">;
}
export interface PublishedDeployment { readonly kind: "deployment"; readonly packageId: Id<"package">; }
export type PublishedPackage = PublishedDeployment | { readonly kind: "project"; readonly packageId: Id<"package"> };
export interface UploadSession {
  readonly owner: Id<"cloud-service">;
  readonly uploadId: Id<"upload">;
  readonly grant: Id<"transfer-grant">;
}
export interface VerifiedUpload { readonly uploadedId: Id<"uploaded-package">; readonly manifestHash: string; }
export interface ExportsApi {
  project(request: Meta & { readonly savedProject: ProjectRef }): Rpc<JobRef<LocalPackage<ProjectPackageManifest>>>;
  deployment(request: Meta & { readonly savedProject: ProjectRef; readonly build: BuildArtifact }): Rpc<JobRef<LocalPackage<DeploymentPackageManifest>>>;
}
export interface TransfersApi {
  upload(request: Meta & { readonly source: Id<"read-resource">; readonly upload: UploadSession }): Rpc<JobRef<{ readonly transferredBytes: Counter }>>;
}
/** Facade 只聚合客户端代理；服务状态不在此对象共享。 */
export interface StageClient {
  readonly connection: {
    readonly serverId: Id<"service-instance">;
    readonly protocol: "draft-0.3";
    readonly operatorSession: OperatorSessionRef;
    readonly services: readonly ServiceContractRef[]; // 实际可用服务；聚合代理不要求宿主装齐所有模块。
    readonly supportedOperations: readonly string[];
    readonly maxWaitMs: number;
    readonly maxMessageBytes: number;
  };
  readonly projects: ProjectApi;
  readonly sessions: SessionApi;
  readonly fixtures: FixtureLibraryApi;
  readonly assets: AssetsApi;
  readonly bindings: BindingsApi;
  readonly targets: TargetsApi;
  readonly builds: BuildsApi;
  readonly deployment: DeploymentApi;
  readonly control: ControlApi;
  readonly observe: ObserveApi;
  readonly preview: PreviewApi;
  readonly external: ExternalApi;
  readonly monitors: MonitorApi;
  readonly jobs: JobsApi;
  readonly requests: RequestsApi;
  readonly exports: ExportsApi;
  readonly transfers: TransfersApi;
  disconnect(): Promise<void>; // 只断开客户端；运行行为遵守租约和会话策略。
}
export interface WireRequest {
  readonly protocol: "draft-0.3";
  readonly requestId: Id<"request">;
  readonly method: string; // 实现时从权威接口生成的封闭方法表中选择。
  readonly payload: Json;
}
export interface RpcTransport {
  request(message: WireRequest, options: { readonly timeoutMs: number }): Promise<Result<Json>>;
  close(): Promise<void>;
}
/** 以下只有构造签名；本地与远程共享客户端调用面，不共享服务端类实例。 */
export declare class LocalIpcTransport implements RpcTransport {
  constructor(options: { readonly endpoint: string });
  request(message: WireRequest, options: { readonly timeoutMs: number }): Promise<Result<Json>>;
  close(): Promise<void>;
}
export declare class RemoteTransport implements RpcTransport {
  constructor(options: { readonly url: string; readonly credentialProvider: () => Promise<string> });
  request(message: WireRequest, options: { readonly timeoutMs: number }): Promise<Result<Json>>;
  close(): Promise<void>;
}
export declare function connectStage(transport: RpcTransport): Rpc<StageClient>;

/** 云业务是独立客户端；不暴露任意现场 tick 控制。 */
export interface CloudClient {
  openUpload(request: Meta & { readonly manifest: PackageManifest }): Rpc<UploadSession>;
  verifyUpload(request: Meta & { readonly uploadId: Id<"upload"> }): Rpc<JobRef<VerifiedUpload>>;
  publish(request: Meta & { readonly verified: VerifiedUpload }): Rpc<JobRef<PublishedPackage>>;
  assign(request: Meta & { readonly deployment: PublishedDeployment; readonly deviceId: Id<"device"> }): Rpc<{
    readonly assignmentId: Id<"assignment">; readonly state: "desired";
  }>;
  inspectDevice(deviceId: Id<"device">): Rpc<{
    readonly desired: Id<"package"> | null;
    readonly downloaded: Id<"package"> | null;
    readonly prepared: Id<"package"> | null;
    readonly active: Id<"package"> | null;
    readonly bootId: Id<"device-boot"> | null;
    readonly lastReportedAt: string | null; // 从未上报不能伪造时间或运行状态。
  }>;
  readonly jobs: JobsApi;
  readonly requests: RequestsApi;
}
