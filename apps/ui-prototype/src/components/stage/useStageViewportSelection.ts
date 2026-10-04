import { useEffect, type RefObject } from "react";
import type { ProjectView } from "../../application-host";
import type { StageSelection } from "../../stage-types";
import { readPrevisTargets, type PrevisTarget } from "../../previs-objects";
import { validSelection } from "./stage-selection";

export function useStageViewportSelection({
  targets,
  onTargets,
  project,
  beforeChange,
  replace,
  reveal,
  cancel,
}: {
  targets: StageSelection[];
  onTargets?(targets: StageSelection[]): void;
  project: RefObject<ProjectView>;
  beforeChange(): Promise<boolean>;
  replace(targets: StageSelection[]): void;
  reveal(target: StageSelection, project?: ProjectView): void;
  cancel(): void;
}) {
  const key = JSON.stringify(targets);
  useEffect(
    () => onTargets?.(JSON.parse(key) as StageSelection[]),
    [key, onTargets],
  );
  return async (next: PrevisTarget[], isActive: () => boolean) => {
    if (
      !isActive() ||
      !(await beforeChange()) ||
      !isActive() ||
      !readPrevisTargets(next)
    )
      return false;
    if (validSelection(project.current.stage, next).length !== next.length)
      return false;
    next.forEach((target) => reveal(target, project.current));
    replace(next);
    cancel();
    return true;
  };
}
