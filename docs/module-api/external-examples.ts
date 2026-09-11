/** 类型检查样例，绝不自动连接设备。回调／呈现对象仅在客户端本地使用。 */
import { decimal, type ClockInstant, type Counter, type Id, type IdFactory,
  type Meta, type Result, type StageClient } from "./contracts";
import type { ExternalAction, ExternalControlContext, ExternalControlLease, ExternalOutcome, ExternalTargetRef,
  MonitorOutput, MonitorPresenter, MonitorState, MonitorView } from "./external-contracts";

function must<T>(result: Result<T>): T {
  if (!result.ok) throw result.error;
  return result.value;
}
function meta(ids: IdFactory): Meta {
  return { protocol: "draft-0.3", commandId: ids.next<"command">() };
}

// 1. 目录、能力与只读预检。返回租约由调用方保管、按有效期续租，结束时 relinquish。
export async function takeExternalTarget(client: StageClient, ids: IdFactory, context: ExternalControlContext,
  target: ExternalTargetRef, action: ExternalAction): Promise<ExternalControlLease> {
  const system = must(await client.external.inspect(target.systemId));
  const device = system.targets.find(item => item.target.targetId === target.targetId);
  if (!system.connected || !device?.actions.includes(action.kind)) throw new Error("目标离线或不支持动作");
  const report = must(await client.external.preflight({ expected: system.ref, target, action }));
  // 本例采用严格策略；产品可按明确节目政策处理 unverifiable，不能伪装已准备。
  if (report.readiness !== "reported-ready" || report.problems.length > 0) throw new Error("外部就绪尚未确认");
  return must(await client.external.acquire({ ...meta(ids), context, expected: system.ref, target, takeover: "deny-if-owned" }));
}

// 2. 示例提交一次；sequence 由该租约的控制会话统一递增，不能在每次调用时归一。
// deadline 必须是网关声明时钟域内的时间，不能使用 Date.now() 冒充。
export async function triggerExternalContent(client: StageClient, ids: IdFactory,
  lease: ExternalControlLease, sequence: Counter, contentId: Id<"external-content">, deadline: ClockInstant): Promise<{
    readonly ticket: import("./external-contracts").ExternalTicket; readonly outcome: ExternalOutcome;
  }> {
  const request = { ...meta(ids), lease, sequence,
    action: { kind: "trigger-content" as const, contentId },
    when: { kind: "immediate" as const }, deadline, ifLate: "reject" as const };
  const submitted = await client.external.submit(request);
  if (!submitted.ok && submitted.error.code === "OUTCOME_UNKNOWN") {
    const recovery = must(await client.requests.inspect(request.commandId));
    if (recovery.state === "external") {
      return { ticket: recovery.ticket, outcome: must(await client.external.outcome(recovery.ticket)) };
    }
    throw new Error("结果未确认，保留原 commandId 对账，不自动重发外部触发");
  }
  const ticket = must(submitted);
  // 可能只是 accepted/sent。UI 保留 ticket 继续查；不得立刻显示已出声／已出画。
  return { ticket, outcome: must(await client.external.outcome(ticket)) };
}

// 3. 纯监看：不需要 ExternalControlLease，退出只清理本地呈现与监看会话。
// during 回调结束即关闭；长会话由调用方安排 monitors.renew，不能靠活跃网络无限续租。
export async function withExternalMonitor(client: StageClient, presenter: MonitorPresenter, ids: IdFactory,
  sourceId: Id<"monitor-source">,
  during: (view: MonitorView, readState: () => Promise<MonitorState>) => Promise<void>,
  selectedOutput?: MonitorOutput): Promise<void> {
  const source = must(await client.monitors.sources({})).find(item => item.ref.sourceId === sourceId);
  if (!source?.available) throw new Error("监看源不可用；不能以缩略图冒充实时画面");
  const session = must(await client.monitors.open({ ...meta(ids), source: source.ref,
    viewer: presenter.viewer, quality: "realtime-low", initialAudio: "muted" }));
  let view: MonitorView | undefined;
  try {
    view = await presenter.attach(session);
    if (selectedOutput) {
      if (selectedOutput.viewerId !== presenter.viewer.viewerId) throw new Error("监听设备属于另一客户端");
      // 本地监听增益，不发外部 set-parameter，不修改调音台主扩。
      await view.setListening({ output: selectedOutput, muted: false, gainDb: decimal("-18") });
    }
    await during(view, async () => must(await client.monitors.state(session.sessionId)));
  } finally {
    try { await view?.close(); } catch { console.warn("本地呈现关闭未确认"); }
    try {
      const closed = await client.monitors.close({ ...meta(ids), sessionId: session.sessionId });
      if (!closed.ok) console.warn("监看回收未确认", closed.error.code);
    } catch { console.warn("监看回收连接中断，需对账或等待到期回收"); }
  }
}
