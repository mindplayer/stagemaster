/** draft-1：调用边界设计，无运行实现；核心 DTO 实施时由 Rust 生成。 */
import type { Decimal, Id, Meta, ProjectRef, Rpc } from "./shared-contracts";

export interface EffectTemplateRef {
  readonly templateId: Id<"effect-template">;
  readonly revisionId: Id<"effect-template-revision">;
  readonly sha256: string; // 必须由资源验证器核实，不能只检查字符串形状。
}
export interface ElementRef {
  readonly fixtureId: Id<"fixture">;
  readonly elementId: Id<"element">;
}
export interface EffectTiming {
  readonly periodMs: number;
  readonly phaseDegrees: number;
  readonly spreadDegrees: number;
  readonly reverseOrder: boolean;
}
export interface RelativeAxisRule {
  readonly axis: "pan" | "tilt";
  readonly amplitudeDegrees: Decimal;
  readonly offsetDegrees: Decimal;
  readonly phaseDegrees: number;
}
/** 参数是语义量；没有灯具地址或裸 DMX。精确范围须与既有 Rust 能力对齐。 */
export type TemplateRecipe =
  | {
      readonly kind: "intensity-wave";
      readonly waveform: "smooth" | "triangle" | "pulse";
      readonly low: Decimal; // 0..1
      readonly high: Decimal;
      readonly dutyPercent: number;
    }
  | {
      readonly kind: "color-steps";
      readonly steps: readonly {
        readonly role: string;
        readonly position: number; // 周期万分比
        readonly transition: "hold" | "linear";
      }[];
    }
  | { readonly kind: "relative-axes"; readonly axes: readonly RelativeAxisRule[] };

export interface EffectTemplateDraft {
  readonly name: string;
  readonly recipe: TemplateRecipe;
  readonly timing: EffectTiming;
  // 不携带基础亮度／颜色／位置，运行时只占用规则声明的属性。
}
export type ColorRoleValue =
  | { readonly kind: "rgb"; readonly red: Decimal; readonly green: Decimal; readonly blue: Decimal }
  | { readonly kind: "wheel-slot"; readonly attributeId: string; readonly functionKey: string };
export interface ColorRoleBinding {
  readonly element: ElementRef;
  readonly role: string;
  readonly value: ColorRoleValue;
}
/** 通用规则不接受不相关的角色参数；所有目标顺序须显式冻结。 */
export type EffectInstanceInput =
  | {
      readonly kind: "intensity-wave" | "relative-axes";
      readonly colorBindings?: never;
    }
  | { readonly kind: "color-steps"; readonly colorBindings: readonly ColorRoleBinding[] };
export interface EffectBindingRequest {
  readonly project: ProjectRef;
  readonly sceneId: Id<"cue">;
  readonly template: EffectTemplateRef;
  readonly orderedElements: readonly ElementRef[];
  readonly timing: EffectTiming;
  readonly input: EffectInstanceInput;
  readonly target: {
    readonly targetId: Id<"target">;
    readonly capabilityRevision: Id<"target-capability-revision">;
  };
}
export interface EffectBindingIssue {
  readonly element?: ElementRef;
  readonly field: string;
  readonly code: "MISSING_BINDING" | "UNSUPPORTED_CAPABILITY" | "INVALID_VALUE" |
    "MISSING_DEPENDENCY" | "PHYSICAL_LIMIT" | "ATTRIBUTE_CONFLICT" | "RESOURCE_LIMIT";
  readonly message: string;
}
export interface EffectReviewTicket {
  readonly reviewId: Id<"effect-binding-review">;
  readonly project: ProjectRef;
  readonly template: EffectTemplateRef;
  // 服务保存其不可变请求、解析映射与依赖修订；不得仅信任客户端传回的票据字段。
}
export type EffectBindingReview =
  | { readonly status: "blocked"; readonly issues: readonly EffectBindingIssue[] }
  | { readonly status: "needs-binding"; readonly issues: readonly EffectBindingIssue[] }
  | {
      readonly status: "ready";
      readonly ticket: EffectReviewTicket;
      readonly resolvedElements: readonly ElementRef[];
      readonly notes: readonly string[];
    };
export interface EffectBindingApi {
  review(request: EffectBindingRequest): Rpc<EffectBindingReview>;
  apply(request: Meta & { readonly ticket: EffectReviewTicket }): Rpc<{
    readonly project: ProjectRef;
    readonly createdEffectId: Id<"effect">;
  }>;
}
export interface EffectCatalogEntry {
  readonly ref: EffectTemplateRef;
  readonly name: string;
  readonly kind: TemplateRecipe["kind"];
  readonly tags: readonly string[];
}
/** Fastify 目录服务；身份／权限通过既有连接上下文，不借此提供现场输出。 */
export interface EffectCatalogApi {
  search(request: { readonly query: string; readonly cursor: string | null }): Rpc<{
    readonly entries: readonly EffectCatalogEntry[];
    readonly nextCursor: string | null;
  }>;
  inspect(ref: EffectTemplateRef): Rpc<{
    readonly entry: EffectCatalogEntry;
    readonly recipe: EffectTemplateDraft;
    readonly requiredCoreCapabilities: readonly string[];
  }>;
}
