import type { PreviewSnapshot } from "../../sequence-types";

export interface ExecutionPosition {
  sequenceId: string | null;
  currentId: string | null;
  nextId: string | null;
  status: NonNullable<PreviewSnapshot["loaded"]>["status"] | null;
  stale: boolean;
}

/** Only the loaded player snapshot determines running and next steps. */
export function executionPosition(
  loaded: PreviewSnapshot["loaded"],
): ExecutionPosition {
  if (!loaded || loaded.sceneId)
    return {
      sequenceId: null,
      currentId: null,
      nextId: null,
      status: null,
      stale: false,
    };
  const index = loaded.steps.findIndex((s) => s.id === loaded.stepId);
  const next =
    loaded.canNext && (!loaded.stepId || index >= 0)
      ? (loaded.steps[index + 1] ?? loaded.steps[0])
      : undefined;
  return {
    sequenceId: loaded.sequenceId,
    currentId: index < 0 ? null : loaded.stepId,
    nextId: next?.id ?? null,
    status: loaded.status,
    stale: loaded.stale,
  };
}

export function executionPhase(loaded: PreviewSnapshot["loaded"]) {
  if (!loaded?.stepId)
    return { label: "等待开始", elapsed: 0, total: 0, progress: 0 };
  const elapsed = loaded.elapsedMs;
  const delay = loaded.delayMs;
  const fade = loaded.fadeMs;
  const part =
    elapsed < delay
      ? { label: "延时", elapsed, total: delay }
      : elapsed < delay + fade
        ? { label: "渐变", elapsed: elapsed - delay, total: fade }
        : loaded.waitMs === null
          ? {
              label: loaded.sceneId ? "场景持续播放" : "等待人工推进",
              elapsed: elapsed - delay - fade,
              total: 0,
            }
          : {
              label: "自动等待",
              elapsed: elapsed - delay - fade,
              total: loaded.waitMs,
            };
  return {
    ...part,
    progress: part.total ? Math.min(1, part.elapsed / part.total) : 1,
  };
}
