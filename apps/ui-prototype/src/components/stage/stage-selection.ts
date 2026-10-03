import type { StageSelection, StageView } from "../../stage-types.ts";
import { selectedStage } from "../../stage-tools.ts";
export const targetKey = (t: StageSelection) => `${t.kind}:${t.id}`;
export function validSelection(stage: StageView, targets: StageSelection[]) {
  const seen = new Set<string>();
  return targets.filter((t) => {
    const key = targetKey(t);
    if (seen.has(key) || !selectedStage(stage, t)) return false;
    seen.add(key);
    return true;
  });
}
export function selectTarget(
  current: StageSelection[],
  target: StageSelection,
  additive = false,
  preserve = false,
) {
  const exists = current.some((t) => targetKey(t) === targetKey(target));
  if (preserve && exists) return current;
  if (!additive) return [target];
  return exists
    ? current.filter((t) => targetKey(t) !== targetKey(target))
    : [...current, target];
}
