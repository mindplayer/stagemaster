import type { StepView } from "./sequence-types";

/** Selection is view state; mutations and final validation remain in Rust. */
export function orderedStepIds(steps: StepView[], ids: string[]) {
  const selected = new Set(ids);
  return steps.filter((s) => selected.has(s.id)).map((s) => s.id);
}
export function toggleStepRange(
  ids: string[],
  visible: StepView[],
  id: string,
  anchor: string | null,
  range: boolean,
) {
  const from = visible.findIndex((s) => s.id === anchor);
  const to = visible.findIndex((s) => s.id === id);
  if (to < 0) return ids;
  if (range && from >= 0) {
    return [
      ...new Set([
        ...ids,
        ...visible
          .slice(Math.min(from, to), Math.max(from, to) + 1)
          .map((s) => s.id),
      ]),
    ];
  }
  return ids.includes(id) ? ids.filter((value) => value !== id) : [...ids, id];
}
export function addedStepIds(before: StepView[], after: StepView[]) {
  const existing = new Set(before.map((s) => s.id));
  return after.filter((s) => !existing.has(s.id)).map((s) => s.id);
}
