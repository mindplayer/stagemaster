import type { ProjectView } from "../../application-host";
import type { StageSelection } from "../../stage-types";
import { selectedStage } from "../../stage-tools";
import { placementTargets } from "../../stage-locks";
import { useCommittedStageAction } from "./useCommittedStageAction";
import type { useStageSelection } from "./useStageSelection";
import { visibleStage, type PlanVisibility } from "./stage-display";
type Intent =
  | {
      kind: "choose";
      target: StageSelection;
      additive: boolean;
      preserve: boolean;
    }
  | { kind: "placements"; ids: string[]; additive: boolean }
  | { kind: "visibility"; value: PlanVisibility };
/** Selection changes follow the same committed-source/lifetime gate as arrangement and rigging. */
export function useStageSelectionActions({
  project,
  visible,
  busy,
  beforeChange,
  selected,
  reveal,
  setVisibility,
  cancel,
  onError,
}: {
  project: ProjectView;
  visible: boolean;
  busy: boolean;
  beforeChange(): Promise<boolean>;
  selected: ReturnType<typeof useStageSelection>;
  reveal(target: StageSelection): void;
  setVisibility(value: PlanVisibility): void;
  cancel(): void;
  onError(message: string): void;
}) {
  return useCommittedStageAction<Intent>({
    scopeId: project.id,
    active: visible,
    busy,
    beforeChange,
    onError,
    execute(intent) {
      if (intent.kind === "choose") {
        if (!selectedStage(project.stage, intent.target))
          throw new Error("所选对象已不存在，请重新选择");
        reveal(intent.target);
        selected.choose(intent.target, intent.additive, intent.preserve);
      } else if (intent.kind === "placements") {
        const targets = placementTargets(intent.ids);
        if (targets.some((t) => !selectedStage(project.stage, t)))
          throw new Error("所选灯位已不存在，请重新选择");
        targets.forEach((target) => reveal(target));
        selected.merge(targets, intent.additive);
      } else {
        const shown = visibleStage(project.stage, intent.value);
        selected.replace(
          selected.targets.filter((t) => selectedStage(shown, t)),
        );
        setVisibility(intent.value);
      }
      cancel();
    },
  });
}
