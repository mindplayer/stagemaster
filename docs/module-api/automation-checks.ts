/** 编译期反例：只检查接口隔离，鉴权和业务限制必须由运行验收证明。 */
import type { AutomationOperation, AutomationToolsApi, ApplyProposalRequest,
  AutomationSessionRef, EditProposalRef } from "./automation-contracts";
import type { Id, Meta } from "./shared-contracts";
import type { TimelineDraft } from "./contracts";

export function noAuthorityEscalation(tools: AutomationToolsApi, request: ApplyProposalRequest): void {
  // @ts-expect-error 模型代理不能创建或扩大授权会话。
  void tools.openSession;
  // @ts-expect-error 人工确认入口不对模型工具暴露。
  void tools.applyReviewed(request);
  // @ts-expect-error 不能自报 confirmed 绕过服务策略。
  void tools.applyScoped({ ...request, confirmed: true });
  // @ts-expect-error 编辑代理不提供现场控制。
  void tools.control;
  // @ts-expect-error 编辑代理不提供包激活或物理帧发送。
  void tools.deployment;
  // @ts-expect-error 没有原始输出接口。
  void tools.sendDmx;
}
export function noUnstructuredOrExternalWrite(timeline: TimelineDraft): void {
  // @ts-expect-error 不能用自由 JSON patch 绕过领域命令。
  const patch: AutomationOperation = { kind: "json-patch", path: "/lighting", value: {} };
  // @ts-expect-error 初批编辑能力不包括外部媒体／机构事件。
  const external: AutomationOperation = { kind: "upsert-timeline", timeline };
  void patch;
  void external;
}
export function proposalNeedsVersion(tools: AutomationToolsApi, meta: Meta,
  session: AutomationSessionRef, proposal: EditProposalRef, context: Id<"automation-context">): void {
  // @ts-expect-error 应用须指定当前预期修订。
  void tools.applyScoped({ ...meta, session, proposal });
  // @ts-expect-error 自由自然语言不是可执行编辑操作。
  void tools.propose({ ...meta, session, context, title: "生成效果", operations: ["让灯漂亮一点"] });
  // @ts-expect-error 上下文引用不能当作提案引用。
  void tools.inspect({ session, proposalId: context });
}
