import { useRef, useState } from "react";
import type { ProjectView } from "../../application-host";
import type { StageSelection } from "../../stage-types";
import {
  ALL_VISIBLE,
  revealStageTarget,
  type PlanVisibility,
} from "./stage-display";

/** Display preferences and revealing edited targets share one source of truth. */
export function useStageVisibility(project: ProjectView) {
  const source = useRef(project);
  source.current = project;
  const [planVisibility, setPlanVisibility] =
    useState<PlanVisibility>(ALL_VISIBLE);
  function revealInPlan(target: StageSelection, project = source.current) {
    setPlanVisibility((old) => revealStageTarget(project.stage, old, target));
  }
  function revealPlacements(ids: string[], project = source.current) {
    setPlanVisibility((old) =>
      ids.reduce(
        (value, id) =>
          revealStageTarget(project.stage, value, { kind: "placement", id }),
        old,
      ),
    );
  }
  return { planVisibility, setPlanVisibility, revealInPlan, revealPlacements };
}
