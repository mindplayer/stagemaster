/** 草案 0.3 的共享值与上下文；不导入任何业务 API 或驱动模块。 */
export type Id<K extends string> = string & { readonly __id: K };
export type Counter = string & { readonly __unsignedDecimal: true };
export type Decimal = string & { readonly __decimal: true };
export type Json = null | boolean | number | string | readonly Json[] |
  { readonly [key: string]: Json };
export type Result<T> = { readonly ok: true; readonly value: T } |
  { readonly ok: false; readonly error: Problem };
export type Rpc<T> = Promise<Result<T>>;
export interface Problem {
  readonly code: "UNAUTHORIZED" | "REVISION_CONFLICT" | "STALE_CONTEXT" |
    "INVALID_INPUT" | "UNSUPPORTED_CAPABILITY" | "DEPENDENCY_CONFLICT" |
    "QUEUE_FULL" | "RESOURCE_MISSING" | "LEASE_LOST" | "NOT_RETAINED" |
    "IO_FAILURE" | "OUTCOME_UNKNOWN" | "CANCELLED";
  readonly messageKey: string;
  readonly details: Json;
  readonly retry: "never" | "same-command" | "refresh-context" | "reconcile";
}
export interface Meta {
  readonly protocol: "draft-0.3";
  readonly commandId: Id<"command">;
}
export interface IdFactory { next<K extends string>(): Id<K>; }
/** 仅做表示格式检查；范围、单位适用性仍由 Rust 权威服务校验。 */
export declare function decimal(text: string): Decimal;
export declare function counter(text: string): Counter;
export interface ProjectRef {
  readonly projectId: Id<"project">;
  readonly revisionId: Id<"project-revision">;
}
export interface BindingRef {
  readonly bindingId: Id<"binding">;
  readonly revisionId: Id<"binding-revision">;
}
export interface MediaTime {
  readonly ticks: string; // 有符号整数的规范字符串。
  readonly ticksPerSecond: Counter; // 正整数；跨时基转换的舍入规则由语义版本规定。
}
export interface ClockInstant {
  readonly clockId: Id<"clock">;
  readonly clockEpoch: Counter;
  readonly ticks: Counter; // 本接口统一为该时钟域的单调纳秒；禁止与别的 clockId 直接比较。
}
export interface RunCursor {
  readonly runtimeId: Id<"runtime-instance">;
  readonly domainId: Id<"execution-domain">;
  readonly planGeneration: Counter | null;
}
export interface ActiveRun extends RunCursor {
  readonly planGeneration: Counter;
  readonly source: ProjectRef;
  readonly binding: BindingRef;
}
/** 宿主签发的操作会话，开机即可建立；身份不等于编辑、控制或输出授权。 */
export interface OperatorSessionRef {
  readonly kind: "operator-session";
  readonly sessionId: Id<"operator-session">;
  readonly generation: Counter;
}
export interface TransportCursor {
  readonly run: ActiveRun;
  readonly syncGroupId: Id<"sync-group">;
  readonly transportGeneration: Counter;
}
/** 协商后的服务契约；版本与 schema 必须来自双方认可的注册表。 */
export interface ServiceContractRef {
  readonly serviceId: string;
  readonly major: number;
  readonly minor: number;
  readonly schemaHash: string;
}
