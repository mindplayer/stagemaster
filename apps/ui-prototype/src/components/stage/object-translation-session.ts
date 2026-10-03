import type { EditOperation, ProjectView } from "../../application-host.ts";
import type { StageSelection, SpatialVector3 } from "../../stage-types.ts";
import { translationCommand } from "./object-translation.ts";
export interface ObjectTranslationDraft {
  source: string;
  targets: StageSelection[];
  delta: SpatialVector3;
}
export const translationSource = (project: ProjectView) =>
  JSON.stringify([project.id, project.stage]);
export function objectTranslationOperations(
  project: ProjectView,
  draft: ObjectTranslationDraft,
): EditOperation[] {
  if (draft.source !== translationSource(project))
    throw new Error("场地已变化，请取消本次移动后重新输入");
  const command = translationCommand(project.stage, draft.targets, draft.delta);
  return Object.values(draft.delta).every((v) => Number(v) === 0)
    ? []
    : [{ op: "stage", command }];
}
