import type { ExecutionSource } from "./execution-types";

export interface SourceProgress {
  phase: "idle" | "delay" | "fade" | "wait" | "hold" | "finished";
  elapsedMs: string;
  phaseElapsedMs: string;
  phaseDurationMs: string | null;
  nextStep: string | null;
  nextWrap: boolean;
}
export interface ExecutionSourceState {
  id: string;
  level: number;
  status: string | null;
  step: string | null;
  progress?: SourceProgress | null;
  held?: import("./execution-manual").ManualTarget[];
}
function milliseconds(value: string): bigint | null {
  if (!/^(0|[1-9]\d{0,19})$/.test(value)) return null;
  const parsed = BigInt(value);
  return parsed <= 18446744073709551615n ? parsed : null;
}
export function progressSeconds(value: bigint): string {
  return `${value / 1000n}.${((value % 1000n) / 100n).toString()} 秒`;
}
// Format one authoritative snapshot only; no browser clock, command or interpolation.
export function sourceProgress(
  source: ExecutionSource,
  state?: ExecutionSourceState,
) {
  const p = state?.progress;
  if (!p || state.id !== source.id) return null;
  const elapsed = milliseconds(p.elapsedMs);
  const phaseElapsed = milliseconds(p.phaseElapsedMs);
  const duration =
    p.phaseDurationMs === null ? null : milliseconds(p.phaseDurationMs);
  const next = source.steps.find((s) => s.id === p.nextStep);
  const current = source.steps.findIndex((s) => s.id === state.step);
  const bounded = ["delay", "fade", "wait"].includes(p.phase);
  if (
    elapsed === null ||
    phaseElapsed === null ||
    phaseElapsed > elapsed ||
    (p.nextStep !== null && !next) ||
    (p.nextWrap &&
      (!next ||
        next.id !== source.steps[0]?.id ||
        current !== source.steps.length - 1)) ||
    (bounded &&
      (duration === null ||
        duration <= 0n ||
        duration > 86400000n ||
        phaseElapsed >= duration)) ||
    (!bounded && p.phaseDurationMs !== null) ||
    (p.phase === "idle"
      ? state.status !== "Idle" || state.step !== null || elapsed !== 0n
      : p.phase === "finished"
        ? state.status !== "Finished" || current < 0
        : !["Running", "Paused"].includes(state.status ?? "") || current < 0)
  )
    return null;
  const labels: Record<SourceProgress["phase"], string> = {
    idle: "尚未执行",
    delay: "延时",
    fade: "渐变",
    wait: "自动等待",
    hold:
      source.selection.kind === "scene"
        ? "保持输出"
        : next
          ? "等待执行下一步"
          : "末步保持",
    finished: "已结束",
  };
  if (!labels[p.phase]) return null;
  return {
    phase: p.phase,
    label: `${state.status === "Paused" ? "已暂停 · " : ""}${labels[p.phase]}`,
    elapsed: progressSeconds(elapsed),
    remaining:
      duration === null ? null : progressSeconds(duration - phaseElapsed),
    percent:
      duration === null ? null : Number((phaseElapsed * 1000n) / duration) / 10,
    next: next
      ? `${next.number} · ${next.name}${p.nextWrap ? "（回到首步）" : ""}`
      : null,
  };
}
