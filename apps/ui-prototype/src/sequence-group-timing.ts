import type { SequenceEdit, StepView, StepTimingPatch } from "./sequence-types";
import { seconds, secondsToMs } from "./sequence-tools.ts";
export interface GroupTimingDraft {
  ids: string[];
  delayEnabled: boolean;
  delay: string;
  fadeEnabled: boolean;
  fade: string;
  advanceEnabled: boolean;
  advance: "manual" | "after" | "mixed";
  wait: string;
}
export function groupTimingDraft(steps: StepView[]): GroupTimingDraft {
  const shared = (key: "delayMs" | "fadeMs" | "waitMs") =>
    steps.length && steps.every((s) => s[key] === steps[0][key])
      ? steps[0][key]
      : undefined;
  return {
    ids: steps.map((s) => s.id),
    delayEnabled: false,
    fadeEnabled: false,
    advanceEnabled: false,
    delay: shared("delayMs") === undefined ? "" : seconds(shared("delayMs")!),
    fade: shared("fadeMs") === undefined ? "" : seconds(shared("fadeMs")!),
    advance: steps.every((s) => s.waitMs === null)
      ? "manual"
      : steps.every((s) => s.waitMs !== null)
        ? "after"
        : "mixed",
    wait:
      typeof shared("waitMs") === "number" ? seconds(shared("waitMs")!) : "",
  };
}
export class GroupTimingError extends Error {
  field: "delay" | "fade" | "wait" | "advance";
  constructor(field: GroupTimingError["field"], message: string) {
    super(message);
    this.field = field;
  }
}
export function groupTimingCommand(
  sequenceId: string,
  draft: GroupTimingDraft,
): SequenceEdit | null {
  const patch: StepTimingPatch = {};
  const time = (field: "delay" | "fade" | "wait", label: string) => {
    try {
      return secondsToMs(draft[field], label);
    } catch (reason) {
      throw new GroupTimingError(
        field,
        reason instanceof Error ? reason.message : String(reason),
      );
    }
  };
  if (draft.delayEnabled) patch.delayMs = time("delay", "延时");
  if (draft.fadeEnabled) patch.fadeMs = time("fade", "渐变");
  if (draft.advanceEnabled && draft.advance === "mixed")
    throw new GroupTimingError("advance", "请选择统一的推进方式");
  if (draft.advanceEnabled)
    patch.advance =
      draft.advance === "manual"
        ? { kind: "manual" }
        : { kind: "after", waitMs: time("wait", "自动等待") };
  if (!Object.keys(patch).length) return null;
  if (!draft.ids.length || new Set(draft.ids).size !== draft.ids.length)
    throw new Error("请重新选择需要修改时间的步骤");
  return {
    kind: "editSteps",
    id: sequenceId,
    stepIds: draft.ids,
    operation: { kind: "timing", patch },
  };
}
