import type {
  ExecutionBatchAction,
  ExecutionBatchRequest,
  ExecutionSource,
  ExecutionView,
} from "./execution-types";
import { sourceStatus } from "./execution-board.ts";

export const batchNames = {
  pause: "暂停所选",
  resume: "继续所选",
  stop: "停止所选",
} as const;
export const ordinaryProgram = (s: ExecutionSource) =>
  s.selection.kind === "scene" || s.selection.kind === "sequence";
export function selectPrograms(
  selected: readonly string[],
  sources: readonly ExecutionSource[],
  ids: readonly string[],
  mode: "toggle" | "add",
) {
  const available = new Set(sources.filter(ordinaryProgram).map((s) => s.id));
  const next = selected.filter((id) => available.has(id));
  for (const id of ids) {
    if (!available.has(id)) continue;
    const index = next.indexOf(id);
    if (index < 0) next.push(id);
    else if (mode === "toggle") next.splice(index, 1);
  }
  return next;
}
export interface BatchReview {
  request: ExecutionBatchRequest;
  layout: string;
  project: string;
  session: string;
  selectionKey: string;
  members: { id: string; name: string; status: string; number: number }[];
}
export function batchReady(runtime: ExecutionView) {
  const state = runtime.observation.snapshot?.state;
  return (
    runtime.controlling &&
    !!runtime.sessionId &&
    !runtime.pending &&
    runtime.observation.phase === "running" &&
    !runtime.observation.fault &&
    !!state &&
    !state.fault &&
    state.owner?.sessionId === runtime.sessionId &&
    runtime.catalog.capabilities?.includes("sourceBatch") === true
  );
}
export function batchEffect(
  runtime: ExecutionView,
  ids: readonly string[],
  kind: ExecutionBatchAction["kind"],
) {
  return runtime.catalog.sources
    .filter((s) => ids.includes(s.id) && ordinaryProgram(s))
    .filter((s) => {
      const status = sourceStatus(s, runtime);
      return kind === "pause"
        ? status === "Running"
        : kind === "resume"
          ? status === "Paused"
          : status === "Running" ||
            status === "Paused" ||
            status === "Finished";
    }).length;
}
export function reviewBatch(
  runtime: ExecutionView,
  selected: readonly string[],
  kind: ExecutionBatchAction["kind"],
): BatchReview {
  if (!batchReady(runtime))
    throw Error("后台未提供批量控制或状态不可操作，请刷新并核对控制权");
  if (
    !selected.length ||
    selected.length > 64 ||
    new Set(selected).size !== selected.length
  )
    throw Error("请选择 1—64 个不重复的普通节目");
  const members = selected.map((id) => {
    const source = runtime.catalog.sources.find((s) => s.id === id);
    if (!source || !ordinaryProgram(source))
      throw Error("所选节目已变更；音乐和手动层须单独操作");
    const status = sourceStatus(source, runtime);
    if (!status || status === "Unknown")
      throw Error(`“${source.name}”状态未知，请先刷新`);
    return {
      id,
      name: source.name,
      status,
      number: runtime.catalog.sources.indexOf(source) + 1,
    };
  });
  if (!batchEffect(runtime, selected, kind))
    throw Error("所选节目没有需要执行此操作的状态");
  return {
    request: {
      kind: "batch",
      hostId: runtime.hostId,
      revision: runtime.observation.snapshot!.state.revision,
      sources: [...selected],
      action: { kind },
    },
    layout: runtime.catalog.layout,
    project: runtime.catalog.projectId,
    session: runtime.sessionId!,
    selectionKey: JSON.stringify(selected),
    members,
  };
}
export function validBatchReview(
  review: BatchReview,
  runtime: ExecutionView,
  selected: readonly string[],
) {
  return (
    batchReady(runtime) &&
    review.request.hostId === runtime.hostId &&
    review.layout === runtime.catalog.layout &&
    review.project === runtime.catalog.projectId &&
    review.session === runtime.sessionId &&
    review.request.revision === runtime.observation.snapshot?.state.revision &&
    review.selectionKey === JSON.stringify(selected) &&
    review.members.every((m) =>
      runtime.catalog.sources.some(
        (s) => s.id === m.id && s.name === m.name && ordinaryProgram(s),
      ),
    )
  );
}
