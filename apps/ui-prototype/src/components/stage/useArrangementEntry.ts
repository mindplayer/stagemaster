import type { ProjectView } from "../../application-host";
import type { StageSpace } from "../../stage-types";
import { movementBlocker, placementTargets } from "../../stage-locks";
import { useCommittedStageAction } from "./useCommittedStageAction";

export function useArrangementEntry({
  project,
  visible,
  busy,
  beforeChange,
  cancel,
  open,
  reveal,
  onOpen,
  onError,
}: {
  project: ProjectView;
  visible: boolean;
  busy: boolean;
  beforeChange(): Promise<boolean>;
  cancel(): void;
  open(ids: string[], space: StageSpace | undefined, selected: boolean): void;
  reveal(ids: string[]): void;
  onOpen?(): void;
  onError(message: string): void;
}) {
  return useCommittedStageAction<{ ids: string[] | null; spaceId?: string }>({
    scopeId: project.id,
    active: visible,
    busy,
    beforeChange,
    onError,
    execute(intent) {
      const ids =
        intent.ids ??
        project.fixtures
          .filter(
            (f) => !project.stage.placements.some((p) => p.fixtureId === f.id),
          )
          .map((f) => f.id);
      if (ids.some((id) => !project.fixtures.some((f) => f.id === id)))
        throw new Error("所选灯具已不存在，请重新选择");
      if (movementBlocker(project.stage, placementTargets(ids)))
        throw new Error("所选灯位包含锁定对象，请先解锁再移动或排列");
      const space = project.stage.spaces.find((s) => s.id === intent.spaceId);
      cancel();
      open(ids, space, intent.ids !== null);
      reveal(ids);
      onOpen?.();
    },
  });
}
