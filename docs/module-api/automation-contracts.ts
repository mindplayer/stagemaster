/** 辅助编辑扩展 draft-1：仅类型化伪 API，无服务实现。依据 PRODUCT-ADR-011。
 * 实施时由 Rust 权威编辑契约生成 DTO／Schema；模型适配不持有 StageClient。
 */
import type { Assignment, AttributeAddress, EditOperation, EditReceipt, EffectDraft,
  PreviewRef, ProfileRef, RecipeDraft, Value } from "./contracts";
import type { ClockInstant, Id, Meta, Problem, ProjectRef, Result, Rpc } from "./shared-contracts";

/** 复用既有语义；增加新命令须先扩展领域契约，不接受任意 JSON patch。 */
export type AutomationOperation = Extract<EditOperation, { readonly kind:
  "create-group" | "create-preset" | "update-cue" | "upsert-effect" | "upsert-recipe" }>;
export type AutomationObjectKind = "fixture" | "group" | "preset" | "sequence" | "cue" | "effect" | "recipe";
export type CreatableAutomationKind = "group" | "preset" | "effect" | "recipe";
export interface AutomationSessionRef {
  readonly sessionId: Id<"automation-session">;
  readonly projectId: Id<"project">;
}
export interface AutomationLimits {
  readonly maxContextBytes: number;
  readonly maxReadObjects: number;
  readonly maxOperationsPerProposal: number;
  readonly maxAffectedObjectsPerProposal: number;
  readonly maxCreatedObjectsPerSession: number;
  readonly maxAppliedProposalsPerSession: number;
}
/** 仅可信宿主可申请；服务仍按已验证身份和用户授予的范围校验。 */
export interface AutomationPolicy {
  readonly mode: "review-required" | "scoped-auto";
  readonly readableObjectIds: readonly string[];
  readonly editableObjectIds: readonly string[];
  readonly targetFixtureIds: readonly Id<"fixture">[];
  readonly creatableKinds: readonly CreatableAutomationKind[];
  readonly attributes: readonly string[];
  readonly operations: readonly AutomationOperation["kind"][];
  readonly limits: AutomationLimits;
  readonly lifetimeMs: number;
}
export interface AutomationCapabilities {
  readonly contract: "assisted-editing-draft-1";
  readonly policy: AutomationPolicy; // 服务授予的实际子集，不回显未经核验的请求策略。
  readonly expiresAt: ClockInstant;
  readonly operations: readonly {
    readonly kind: AutomationOperation["kind"];
    readonly schemaId: string; // 由适配层解析到受信注册表中具体输入 Schema。
    readonly schemaHash: string;
  }[];
  readonly candidatePreview: "available" | "unavailable";
}
export interface AttributeCapability {
  readonly address: AttributeAddress;
  readonly acceptedKinds: readonly Value["kind"][];
  readonly constraintSchemaId: string; // 单位／范围／离散集合等，来自同一版本的注册表。
  readonly constraintSchemaHash: string;
}
export type AutomationObjectView =
  | { readonly kind: "fixture"; readonly fixtureId: Id<"fixture">; readonly name: string;
      readonly profile: ProfileRef; readonly attributes: readonly AttributeCapability[] }
  | { readonly kind: "group"; readonly groupId: Id<"group">; readonly name: string;
      readonly fixtures: readonly Id<"fixture">[] }
  | { readonly kind: "preset"; readonly presetId: Id<"preset">; readonly name: string;
      readonly values: readonly Assignment[] }
  | { readonly kind: "sequence"; readonly sequenceId: Id<"sequence">; readonly name: string }
  | { readonly kind: "cue"; readonly cueId: Id<"cue">; readonly sequenceId: Id<"sequence">;
      readonly name: string; readonly values: readonly Assignment[] }
  | { readonly kind: "effect"; readonly effect: EffectDraft }
  | { readonly kind: "recipe"; readonly recipe: RecipeDraft };
export interface AutomationContext {
  readonly token: Id<"automation-context">;
  readonly base: ProjectRef;
  readonly objects: readonly AutomationObjectView[];
  // 服务端同时固定档案、Schema、授权代次和查询范围；过期必须重新读取。
}
export interface EditProposalRef {
  readonly proposalId: Id<"edit-proposal">;
  readonly base: ProjectRef;
  readonly digest: string; // 服务计算的不可变内容摘要；本身不授予任何权限。
}
export interface ReadyEditProposal {
  readonly proposal: EditProposalRef;
  readonly title: string;
  readonly operations: readonly AutomationOperation[];
  readonly changes: readonly {
    readonly objectId: string;
    readonly before: AutomationObjectView | null;
    readonly after: AutomationObjectView | null;
  }[];
  readonly affectedObjectIds: readonly string[];
  readonly warnings: readonly Problem[];
}
export type ProposalCheck = { readonly status: "ready"; readonly value: ReadyEditProposal } |
  { readonly status: "rejected"; readonly problems: readonly Problem[] };
export type ProposalStatus =
  | { readonly state: "ready"; readonly value: ReadyEditProposal }
  | { readonly state: "applied"; readonly proposal: EditProposalRef; readonly receipt: EditReceipt }
  | { readonly state: "discarded" | "expired"; readonly proposal: EditProposalRef };
export interface ApplyProposalRequest extends Meta {
  readonly session: AutomationSessionRef;
  readonly proposal: EditProposalRef;
  readonly expected: ProjectRef;
}
export type ApplicationOutcome =
  | { readonly state: "not-seen" | "processing" | "not-retained" }
  | { readonly state: "completed"; readonly result: Result<EditReceipt> };

/** 可交给模型工具适配层的窄代理。令牌还需经过服务端身份／作用域校验。 */
export interface AutomationToolsApi {
  capabilities(session: AutomationSessionRef): Rpc<AutomationCapabilities>;
  search(request: { readonly session: AutomationSessionRef; readonly base: ProjectRef;
    readonly kind: AutomationObjectKind; readonly text?: string; readonly cursor?: string;
    readonly limit: number }): Rpc<{
      readonly matches: readonly { readonly objectId: string; readonly kind: AutomationObjectKind; readonly name: string }[];
      readonly nextCursor: string | null;
    }>;
  readContext(request: { readonly session: AutomationSessionRef; readonly base: ProjectRef;
    readonly objectIds: readonly string[] }): Rpc<AutomationContext>;
  propose(request: Meta & { readonly session: AutomationSessionRef; readonly context: Id<"automation-context">;
    readonly title: string; readonly operations: readonly AutomationOperation[] }): Rpc<ProposalCheck>;
  inspect(request: { readonly session: AutomationSessionRef; readonly proposalId: Id<"edit-proposal"> }): Rpc<ProposalStatus>;
  /** 只有服务授予 scoped-auto 且仍在范围内才可成功；review-required 返回 UNAUTHORIZED。 */
  applyScoped(request: ApplyProposalRequest): Rpc<EditReceipt>;
  discard(request: Meta & { readonly session: AutomationSessionRef; readonly proposal: EditProposalRef }): Rpc<{
    readonly discarded: boolean; // 已应用时返回 DEPENDENCY_CONFLICT，不能伪装为已撤销。
  }>;
  applicationOutcome(request: { readonly session: AutomationSessionRef;
    readonly commandId: Id<"command"> }): Rpc<ApplicationOutcome>;
}

/** 可信 UI／宿主入口，禁止注册为模型工具。没有 actor 或 confirmed:true 自报授权字段。 */
export interface AutomationHostApi {
  openSession(request: Meta & { readonly base: ProjectRef; readonly policy: AutomationPolicy }): Rpc<AutomationSessionRef>;
  closeSession(request: Meta & { readonly session: AutomationSessionRef }): Rpc<{ readonly closed: boolean }>;
  applyReviewed(request: ApplyProposalRequest): Rpc<EditReceipt>;
  undoApplied(request: Meta & { readonly session: AutomationSessionRef; readonly proposal: EditProposalRef;
    readonly expected: ProjectRef; readonly undoToken: Id<"undo"> }): Rpc<EditReceipt>;
  /** 候选快照进入隔离 PreviewService；不修改工程头或激活运行计划。
   * 不支持时返回 UNSUPPORTED_CAPABILITY；释放／定位复用宿主的 PreviewApi.close/seek。
   */
  openCandidatePreview(request: Meta & { readonly session: AutomationSessionRef;
    readonly proposal: EditProposalRef }): Rpc<PreviewRef>;
}
