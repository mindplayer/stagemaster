/** 草案 0.3：外部控制与监看；仅声明，没有协议、播放器或设备实现。 */
import type { ClockInstant, Counter, Decimal, Id, MediaTime, Meta, OperatorSessionRef,
  Problem, Rpc, TransportCursor } from "./shared-contracts";

export type ExternalKind = "audio-player" | "audio-console" | "media-server" |
  "projector" | "video-router" | "laser-controller" | "camera" | "stage-controller";
export interface ExternalSystemRef {
  readonly systemId: Id<"external-system">;
  readonly connectionGeneration: Counter;
  readonly capabilityRevision: Id<"external-capabilities">;
  readonly contextId: Id<"external-context">; // 服务签发，绑定外部工程／设备上下文；不是可靠远端事务锁。
}
export interface ExternalTargetRef {
  readonly systemId: Id<"external-system">;
  readonly targetId: Id<"external-target">;
}
export type ExternalValue =
  | { readonly kind: "boolean"; readonly value: boolean }
  | { readonly kind: "scalar"; readonly value: Decimal; readonly unit: "normalized" | "db" | "degree" | "second" }
  | { readonly kind: "choice"; readonly optionId: Id<"external-option"> };
export type ExternalAction =
  | { readonly kind: "trigger-content"; readonly contentId: Id<"external-content"> }
  | { readonly kind: "pause" | "resume" | "stop" }
  | { readonly kind: "seek"; readonly position: MediaTime }
  | { readonly kind: "recall-preset"; readonly presetId: Id<"external-preset"> }
  | { readonly kind: "select-input"; readonly inputId: Id<"external-input"> }
  | { readonly kind: "set-parameter"; readonly parameterId: Id<"external-parameter">; readonly value: ExternalValue };
export interface ExternalTargetDescriptor {
  readonly target: ExternalTargetRef;
  readonly kind: ExternalKind;
  readonly name: string;
  readonly actions: readonly ExternalAction["kind"][];
  readonly feedback: "none" | "poll" | "subscribe";
  readonly scheduling: "immediate-only" | "timestamped";
  readonly parameters: readonly { readonly id: Id<"external-parameter">; readonly name: string;
    readonly schemaId: Id<"external-parameter-schema"> }[]; // 类型、范围、枚举和适用目标由权威 schema 校验。
}
export interface ExternalState {
  readonly expected: ExternalSystemRef;
  readonly target: ExternalTargetRef;
  readonly observedAt: ClockInstant; // 网关观测时刻，不伪装对端呈现时刻。
  readonly freshness: "fresh" | "stale" | "unavailable";
  readonly transport: "playing" | "paused" | "stopped" | "unknown" | "not-applicable";
  readonly position: MediaTime | null;
  readonly parameters: readonly { readonly id: Id<"external-parameter">; readonly value: ExternalValue }[];
}
export type ExternalControlContext =
  | { readonly kind: "manual"; readonly operator: OperatorSessionRef }
  | { readonly kind: "show"; readonly transport: TransportCursor };
export interface ExternalControlLease {
  readonly token: Id<"external-control-lease">;
  readonly context: ExternalControlContext;
  readonly expected: ExternalSystemRef;
  readonly target: ExternalTargetRef;
  readonly expiresAt: ClockInstant;
}
export interface ExternalTicket {
  readonly kind: "external-ticket";
  readonly commandId: Id<"command">;
  readonly expected: ExternalSystemRef;
  readonly target: ExternalTargetRef;
}
export type ExternalOutcome =
  | { readonly state: "accepted" | "sent" | "peer-acknowledged" }
  | { readonly state: "state-reported"; readonly observation: ExternalState;
      readonly correlation: "peer-command-id" | "state-only" }
  | { readonly state: "rejected"; readonly error: Problem }
  | { readonly state: "unknown"; readonly reason: string };
export interface ExternalApi {
  systems(): Rpc<readonly ExternalSystemRef[]>; // 已配置且当前主体可访问的目录；配置凭据另由宿主管理。
  inspect(systemId: Id<"external-system">): Rpc<{
    readonly ref: ExternalSystemRef; readonly connected: boolean;
    readonly targets: readonly ExternalTargetDescriptor[];
  }>;
  contents(request: { readonly expected: ExternalSystemRef; readonly target: ExternalTargetRef;
    readonly after?: Id<"external-catalog-cursor"> }): Rpc<{
      readonly entries: readonly { readonly contentId: Id<"external-content">; readonly name: string }[];
      readonly next: Id<"external-catalog-cursor"> | null;
    }>;
  /** 只读预检，不向外部发送 Load 或改变播放；是否真的 ready 取决于对端反馈能力。 */
  preflight(request: { readonly expected: ExternalSystemRef; readonly target: ExternalTargetRef;
    readonly action: ExternalAction }): Rpc<{
      readonly observedAt: ClockInstant;
      readonly readiness: "reported-ready" | "not-ready" | "unverifiable";
      readonly problems: readonly Problem[];
    }>;
  acquire(request: Meta & { readonly context: ExternalControlContext; readonly expected: ExternalSystemRef;
    readonly target: ExternalTargetRef; readonly takeover: "deny-if-owned" }): Rpc<ExternalControlLease>;
  renew(request: Meta & { readonly lease: ExternalControlLease }): Rpc<ExternalControlLease>;
  relinquish(request: Meta & { readonly lease: ExternalControlLease }): Rpc<{ readonly relinquished: boolean }>;
  submit(request: Meta & { readonly lease: ExternalControlLease; readonly sequence: Counter;
    readonly action: ExternalAction;
    readonly when: { readonly kind: "immediate" } | { readonly kind: "at"; readonly instant: ClockInstant };
    readonly deadline: ClockInstant; readonly ifLate: "reject" }): Rpc<ExternalTicket>;
  outcome(ticket: ExternalTicket): Rpc<ExternalOutcome>;
  state(request: { readonly expected: ExternalSystemRef; readonly target: ExternalTargetRef }): Rpc<ExternalState>;
}

export interface MonitorSourceRef {
  readonly kind: "monitor-source-session";
  readonly sourceId: Id<"monitor-source">;
  readonly generation: Counter; // 同名源重启／来源变化不能静默复用旧身份。
}
/** 持久配置只保存稳定目录键；每次使用时解析出当前会话引用。 */
export interface MonitorSourceKey {
  readonly kind: "monitor-source-key";
  readonly sourceId: Id<"monitor-source">;
}
export interface MonitorBindingSpec {
  readonly surfaceId: Id<"video-surface">;
  readonly source: MonitorSourceKey;
}
export type MonitorSignal = "audio" | "video" | "audio-video" | "laser-preview-2d" | "laser-geometry";
export type MonitorTap = "content-preview" | "program-pre-mapping" | "program-post-mapping" |
  "capture-input" | "venue-camera" | "audio-return" | "simulation-reference";
export interface MonitorSource {
  readonly key: MonitorSourceKey;
  readonly ref: MonitorSourceRef;
  readonly name: string;
  readonly ownerNodeId: Id<"node">;
  readonly systemId: Id<"external-system"> | null;
  readonly signal: MonitorSignal;
  readonly tap: MonitorTap;
  readonly available: boolean;
}
export interface MonitorViewer {
  readonly viewerId: Id<"monitor-viewer">;
  readonly delivery: readonly ("browser-stream" | "native-receiver")[];
  readonly signals: readonly MonitorSignal[];
}
export interface MonitorSession {
  readonly sessionId: Id<"monitor-session">;
  readonly source: MonitorSource; // 实际协商的来源与取点；服务端从权威目录返回。
  readonly viewerId: Id<"monitor-viewer">;
  readonly expiresAt: ClockInstant;
  readonly delivery: { readonly kind: "browser-stream" | "native-receiver";
    readonly grantId: Id<"monitor-delivery-grant"> }; // 凭据／本机句柄不入工程，由接收适配解析。
}
export interface MonitorState {
  readonly sessionId: Id<"monitor-session">;
  readonly state: "connecting" | "receiving" | "stale" | "disconnected" | "closed";
  readonly lastReceivedAt: ClockInstant | null;
  readonly sourceTime: ClockInstant | null;
  readonly sourcePosition: MediaTime | null;
  readonly latency: { readonly kind: "unknown" } |
    { readonly kind: "estimated" | "measured"; readonly milliseconds: Decimal; readonly basis: string };
  readonly video: { readonly width: number; readonly height: number; readonly framesPerSecond: Decimal } | null;
  readonly audio: { readonly channels: number; readonly sampleRate: number } | null;
}
export interface MonitorApi {
  resolve(source: MonitorSourceKey): Rpc<MonitorSource>;
  sources(request: { readonly systemId?: Id<"external-system"> }): Rpc<readonly MonitorSource[]>;
  open(request: Meta & { readonly source: MonitorSourceRef; readonly viewer: MonitorViewer;
    readonly quality: "thumbnail" | "realtime-low" | "realtime-full";
    readonly initialAudio: "muted" }): Rpc<MonitorSession>;
  renew(request: Meta & { readonly sessionId: Id<"monitor-session"> }): Rpc<MonitorSession>;
  state(sessionId: Id<"monitor-session">): Rpc<MonitorState>;
  close(request: Meta & { readonly sessionId: Id<"monitor-session"> }): Rpc<{ readonly closed: boolean }>;
}

/** 客户端本地呈现端口；不序列化到服务端，也不改变外部播放或主扩设置。 */
export interface MonitorOutput {
  readonly viewerId: Id<"monitor-viewer">;
  readonly outputId: Id<"local-monitor-output">;
  readonly name: string;
}
export interface MonitorView {
  setListening(request: { readonly output: MonitorOutput; readonly muted: boolean; readonly gainDb: Decimal }): Promise<void>;
  presentation(): Promise<{ readonly latestPresentedAt: ClockInstant | null; readonly audioAudible: boolean }>;
  close(): Promise<void>; // 只关闭本地呈现；服务端订阅另用 monitors.close。
}
export interface MonitorPresenter {
  readonly viewer: MonitorViewer; // 宿主会话注册；服务端必须核实 viewer 归属，不能相信客户端任意填写。
  outputs(): Promise<readonly MonitorOutput[]>;
  attach(session: MonitorSession): Promise<MonitorView>; // 拒绝不匹配的 viewer／格式，默认静音。
}
