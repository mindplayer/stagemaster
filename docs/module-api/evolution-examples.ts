/** 架构变更用例：只做类型检查，不运行设备、云端或媒体服务。 */
import type { ClockInstant, Id, IdFactory, Meta, ProjectRef, StageClient } from "./contracts";
import { awaitJob } from "./examples";
import { bindSurface, type SurfaceCoordinator, type SurfaceDriver } from "./surface-contracts";
import { takeExternalTarget, triggerExternalContent } from "./external-examples";
import { counter } from "./shared-contracts";
import type { ExternalTargetRef, MonitorSourceKey } from "./external-contracts";

function meta(ids: IdFactory): Meta {
  return { protocol: "draft-0.3", commandId: ids.next<"command">() };
}

// 开机未装载节目即可连接面板；暂无运行目标时不启用播放映射。
export async function attachPanelBeforeShow(client: StageClient, coordinator: SurfaceCoordinator, driver: SurfaceDriver) {
  return bindSurface(coordinator, driver, client.connection.operatorSession, []);
}

// 纯编排可导出工程，不需要目标设备、配适、BuildArtifact 或 ActiveRun。
export async function exportForAnotherEditor(client: StageClient, ids: IdFactory, savedProject: ProjectRef) {
  const result = await client.exports.project({ ...meta(ids), savedProject });
  if (!result.ok) throw result.error;
  return awaitJob(client.jobs, result.value);
}

// 稳定监看配置在使用时重新解析；打开与呈现仍使用 withExternalMonitor 的生命周期。
export async function resolveAfterSourceRestart(client: StageClient, source: MonitorSourceKey) {
  const result = await client.monitors.resolve(source);
  if (!result.ok) throw result.error;
  if (!result.value.available) throw new Error("监看源不可用，保留配置并显示断流");
  return result.value.ref;
}

// 演出前显式手动触发外部测试内容也要取得控制租约；不需要虚构一个灯光节目。
export async function manuallyTriggerExternal(client: StageClient, ids: IdFactory, target: ExternalTargetRef,
  contentId: Id<"external-content">, deadline: ClockInstant) {
  const lease = await takeExternalTarget(client, ids,
    { kind: "manual", operator: client.connection.operatorSession }, target, { kind: "trigger-content", contentId });
  try {
    return await triggerExternalContent(client, ids, lease, counter("1"), contentId, deadline);
  } finally {
    try {
      const result = await client.external.relinquish({ ...meta(ids), lease });
      if (!result.ok) console.warn("外部控制权回收未确认", result.error.code);
    } catch { console.warn("控制权回收连接中断，保留原动作记录并等待租约到期"); }
  }
}
