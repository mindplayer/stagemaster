import type { EditOperation, ProjectView } from "../../application-host";
import { sceneUsages } from "./scene-usage.ts";
export const MAX_SCENE_REMOVALS = 128;
export function sceneRemovalReview(
  project: ProjectView,
  ids: readonly string[],
) {
  if (
    !ids.length ||
    ids.length > MAX_SCENE_REMOVALS ||
    new Set(ids).size !== ids.length
  )
    throw new Error(`请选择 1–${MAX_SCENE_REMOVALS} 个不同场景`);
  const requested = new Set(ids);
  const scenes = project.scenes.filter((scene) => requested.has(scene.id));
  if (scenes.length !== ids.length)
    throw new Error("部分场景已不存在，请重新选择");
  return scenes.map((scene) => ({
    id: scene.id,
    name: scene.name,
    usages: sceneUsages(project, scene.id),
  }));
}
export function sceneRemovalCommands(
  project: ProjectView,
  ids: readonly string[],
): EditOperation[] {
  const review = sceneRemovalReview(project, ids);
  const used = review.filter((scene) => scene.usages.length);
  if (used.length)
    throw new Error(`${used.length} 个场景仍被使用，请先处理使用位置`);
  return review.map((scene) => ({ op: "removeScene", id: scene.id }));
}
