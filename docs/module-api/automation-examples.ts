/** 调用样例，不运行模型／设备；只验证现有命令能组成辅助编辑流程。 */
import { decimal, type Assignment, type EditReceipt } from "./contracts";
import type { Id, IdFactory, Meta, ProjectRef, Result } from "./shared-contracts";
import type { AutomationHostApi, AutomationSessionRef, AutomationToolsApi, ReadyEditProposal } from "./automation-contracts";

function must<T>(result: Result<T>): T {
  if (!result.ok) throw new Error(result.error.code);
  return result.value;
}
function meta(ids: IdFactory): Meta { return { protocol: "draft-0.3", commandId: ids.next<"command">() }; }

/** 已授权上下文中的灯具：将现有场景亮度改为 30%，只改当前场景。
 * fixtureIds 和 cueId 来自用户明确选择／服务查询，不由名称猜测。
 */
export async function proposeSelectedIntensity(tools: AutomationToolsApi, ids: IdFactory, input: {
  readonly session: AutomationSessionRef;
  readonly base: ProjectRef;
  readonly fixtureIds: readonly Id<"fixture">[];
  readonly cueId: Id<"cue">;
  readonly intensityAttribute: string; // 从已注册能力选择的亮度键，不硬编码厂商通道名。
}): Promise<ReadyEditProposal> {
  const caps = must(await tools.capabilities(input.session));
  if (!caps.operations.some(op => op.kind === "update-cue")) throw new Error("UNSUPPORTED_CAPABILITY");
  const fixtures = new Set(input.fixtureIds);
  if (fixtures.size === 0) throw new Error("INVALID_INPUT");
  const context = must(await tools.readContext({
    session: input.session, base: input.base, objectIds: [...fixtures, input.cueId],
  }));
  const cue = context.objects.find(obj => obj.kind === "cue" && obj.cueId === input.cueId);
  if (!cue || cue.kind !== "cue") throw new Error("RESOURCE_MISSING");
  const values: Assignment[] = [];
  for (const fixtureId of fixtures) {
    const fixture = context.objects.find(obj => obj.kind === "fixture" && obj.fixtureId === fixtureId);
    if (!fixture || fixture.kind !== "fixture") throw new Error("RESOURCE_MISSING");
    const attributes = fixture.attributes.filter(attr => attr.address.attribute === input.intensityAttribute);
    // 多单元灯具应显式选定单元，不自动猜“主亮度”。
    if (attributes.length !== 1) throw new Error("INVALID_INPUT");
    const attribute = attributes[0];
    if (!attribute || !attribute.acceptedKinds.includes("normalized")) throw new Error("UNSUPPORTED_CAPABILITY");
    values.push({ address: attribute.address, source: { kind: "literal", value: { kind: "normalized", value: decimal("0.3") } } });
  }
  const checked = must(await tools.propose({
    ...meta(ids), session: input.session, context: context.token, title: "所选灯具亮度改为 30%",
    operations: [{ kind: "update-cue", sequenceId: cue.sequenceId, cueId: cue.cueId, mode: "cue-only", values }],
  }));
  if (checked.status === "rejected") throw new Error(checked.problems.map(p => p.code).join(","));
  return checked.value; // UI 展示服务生成的 changes；到这里工程与现场均未改变。
}

/** 只在用户已经查看此精确提案并点击应用的宿主事件中调用。模型拿不到 host。 */
export async function applyFromReview(host: AutomationHostApi, ids: IdFactory,
  session: AutomationSessionRef, ready: ReadyEditProposal): Promise<EditReceipt> {
  return must(await host.applyReviewed({ ...meta(ids), session, proposal: ready.proposal, expected: ready.proposal.base }));
}

/** 用户此前已授予 scoped-auto 时可自动应用；本函数无法自行提升会话权限。
 * commandId 由宿主记录；遇到超时先 applicationOutcome，对账后才按政策重试相同请求。
 */
export async function applyWithinExistingScope(tools: AutomationToolsApi,
  commandId: Id<"command">, session: AutomationSessionRef, ready: ReadyEditProposal): Promise<EditReceipt> {
  return must(await tools.applyScoped({ protocol: "draft-0.3", commandId, session,
    proposal: ready.proposal, expected: ready.proposal.base }));
}

/** 应用后仍使用正常编辑历史；expected 须为用户此刻看到的工程修订。 */
export async function undoFromEditor(host: AutomationHostApi, ids: IdFactory, session: AutomationSessionRef,
  ready: ReadyEditProposal, receipt: EditReceipt, expected: ProjectRef): Promise<EditReceipt> {
  return must(await host.undoApplied({ ...meta(ids), session, proposal: ready.proposal, expected, undoToken: receipt.undoToken }));
}
