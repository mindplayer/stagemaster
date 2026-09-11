/** 宿主内部硬件控制面草案；没有 HID／MIDI／固件实现，不暴露为普通 UI RPC。 */
import type { ActiveRun, ClockInstant, Counter, Decimal, Id, OperatorSessionRef, Result } from "./shared-contracts";
import type { ExternalTargetRef } from "./external-contracts";

export interface SurfaceRef {
  readonly surfaceId: Id<"surface">;
  readonly sessionId: Id<"surface-session">;
  readonly connectionGeneration: Counter;
}
export type SurfaceControl =
  | { readonly kind: "fader" | "potentiometer"; readonly id: Id<"surface-control">;
      readonly minCounts: number; readonly maxCounts: number; readonly touch: boolean; readonly motor: boolean }
  | { readonly kind: "encoder"; readonly id: Id<"surface-control">; readonly press: boolean; readonly touch: boolean }
  | { readonly kind: "button"; readonly id: Id<"surface-control"> }
  | { readonly kind: "axes"; readonly id: Id<"surface-control">; readonly axes: number; readonly mode: "absolute" | "relative" };
export interface SurfaceDescriptor {
  readonly surfaceId: Id<"surface">;
  readonly profileId: Id<"surface-profile">;
  readonly capabilityRevision: Id<"surface-capabilities">;
  readonly firmwareVersion: string;
  readonly controls: readonly SurfaceControl[];
  readonly feedback: readonly ("indicator" | "ring" | "label" | "motor")[];
}
export interface SurfaceContext { readonly surface: SurfaceRef; readonly mappingGeneration: Counter; }
export type SurfaceChange =
  | { readonly kind: "absolute"; readonly controlId: Id<"surface-control">; readonly counts: number;
      readonly origin: "user" | "motor-echo" | "unknown" }
  | { readonly kind: "relative"; readonly controlId: Id<"surface-control">; readonly delta: number }
  | { readonly kind: "button"; readonly controlId: Id<"surface-control">; readonly edge: "press" | "release";
      readonly gestureId: Id<"surface-gesture"> }
  | { readonly kind: "touch"; readonly controlId: Id<"surface-control">; readonly touched: boolean }
  | { readonly kind: "axes"; readonly controlId: Id<"surface-control">; readonly values: readonly number[] };
export interface SurfaceInput {
  readonly context: SurfaceContext;
  readonly sequence: Counter;
  readonly observedAt: ClockInstant;
  readonly changes: readonly SurfaceChange[]; // 有界批次；原始整数范围与有限数值由适配器校验。
}
export interface SurfaceSnapshot {
  readonly context: SurfaceContext;
  readonly sequence: Counter; // 此序号及以前的输入已被屏障消耗；后续 read 必须更新。
  readonly observedAt: ClockInstant;
  readonly positions: readonly { readonly controlId: Id<"surface-control">; readonly counts: number }[];
  readonly pressed: readonly Id<"surface-control">[]; // 重同步状态，不转成 press 事件。
  readonly touched: readonly Id<"surface-control">[];
}
export type SurfaceTarget =
  | { readonly kind: "playback-level" | "playback-go"; readonly run: ActiveRun; readonly playbackId: Id<"playback"> }
  | { readonly kind: "programmer-attribute"; readonly sessionId: Id<"session">; readonly attribute: string }
  | { readonly kind: "external-parameter"; readonly target: ExternalTargetRef; readonly parameterId: Id<"external-parameter"> }
  | { readonly kind: "registered-function"; readonly functionId: Id<"surface-function"> }; // 服务端已注册语义，不能任意执行字符串。
export interface SurfaceBinding {
  readonly controlId: Id<"surface-control">;
  readonly target: SurfaceTarget; // 已解析的运行时绑定；持久映射保存逻辑 ID，不保存 ActiveRun。
  readonly behavior: "absolute-pickup" | "absolute-jump" | "motor-follow" | "relative" | "momentary";
}
export type SurfaceFeedback =
  | { readonly kind: "indicator"; readonly controlId: Id<"surface-control">; readonly state: "off" | "on" | "blink"; readonly color: string }
  | { readonly kind: "ring"; readonly controlId: Id<"surface-control">; readonly value: Decimal }
  | { readonly kind: "label"; readonly controlId: Id<"surface-control">; readonly text: string }
  | { readonly kind: "motor"; readonly controlId: Id<"surface-control">; readonly position: Decimal; readonly mode: "follow" | "hold" };
export interface SurfaceFeedbackBatch {
  readonly context: SurfaceContext;
  readonly sequence: Counter;
  readonly stateCursor: Id<"surface-feedback-state">;
  readonly validUntil: ClockInstant;
  readonly updates: readonly SurfaceFeedback[];
}
export interface SurfaceDriver {
  describe(): Promise<Result<SurfaceDescriptor>>;
  rebaseInput(context: SurfaceContext): Promise<Result<SurfaceSnapshot>>; // 安装会话／映射上下文，屏障后取得基准。
  read(): Promise<Result<SurfaceInput>>; // 宿主工作通路，不由灯光 tick 等待。
  writeFeedback(batch: SurfaceFeedbackBatch): Promise<Result<{ readonly accepted: boolean }>>;
  close(): Promise<void>;
}
export interface SurfaceCoordinator {
  attach(driver: SurfaceDriver, profile: Id<"surface-profile">, operator: OperatorSessionRef): Promise<Result<SurfaceContext>>;
  applyMapping(expected: SurfaceContext, bindings: readonly SurfaceBinding[]): Promise<Result<SurfaceContext>>;
  ingest(input: SurfaceInput): Result<{ readonly state: "accepted" | "ignored-echo" | "resync-required" }>;
  feedback(batch: SurfaceFeedbackBatch): Result<{ readonly queued: boolean }>;
  detach(surface: SurfaceRef): Promise<Result<{ readonly detached: boolean }>>;
}

/** 示例只展示宿主调用，实例由组合入口注入；不创建真实驱动。 */
export async function bindSurface(coordinator: SurfaceCoordinator, driver: SurfaceDriver, operator: OperatorSessionRef,
  bindings: readonly SurfaceBinding[]): Promise<Result<SurfaceContext>> {
  const descriptor = await driver.describe();
  if (!descriptor.ok) return descriptor;
  const attached = await coordinator.attach(driver, descriptor.value.profileId, operator);
  if (!attached.ok) return attached;
  const mapped = await coordinator.applyMapping(attached.value, bindings);
  if (!mapped.ok) {
    // 独立记录清理失败，不覆盖映射错误；宿主还需故障回收与设备反馈超时。
    try {
      const detached = await coordinator.detach(attached.value.surface);
      if (!detached.ok) console.warn("控制面回收未确认", detached.error.code);
    } catch { console.warn("控制面回收连接中断，交由宿主故障回收"); }
  }
  return mapped;
}
