import { useState } from "react";
import type { StageSelection, StageView } from "../../stage-types";
import { selectTarget, validSelection } from "./stage-selection";
import { placementTargets } from "../../stage-locks";
export function useStageSelection(stage: StageView) {
  const [stored, setStored] = useState<StageSelection[]>([]);
  const targets = validSelection(stage, stored);
  const selection = targets.at(-1) ?? null;
  const ids = targets.filter((t) => t.kind === "placement").map((t) => t.id);
  return {
    targets,
    selection,
    ids,
    fixturesOnly: targets.length > 0 && ids.length === targets.length,
    replace: setStored,
    placements(ids: string[]) {
      setStored(placementTargets(ids));
    },
    choose(target: StageSelection, additive = false, preserve = false) {
      setStored((old) =>
        selectTarget(validSelection(stage, old), target, additive, preserve),
      );
    },
    merge(next: StageSelection[], additive = false) {
      setStored((old) =>
        validSelection(stage, additive ? [...old, ...next] : next),
      );
    },
  };
}
