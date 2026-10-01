import type { EditOperation, SceneView } from "../../application-host";
import { uniqueName } from "../../editor-tools.ts";
export const MAX_SCENE_COPIES = 128;
export function sceneCopyCommands(
  scenes: SceneView[],
  ids: readonly string[],
): EditOperation[] {
  const chosen = new Set(ids);
  if (
    !ids.length ||
    ids.length > MAX_SCENE_COPIES ||
    chosen.size !== ids.length
  )
    throw new Error(`请选择 1–${MAX_SCENE_COPIES} 个不重复的场景`);
  const sources = scenes.filter((scene) => chosen.has(scene.id));
  if (sources.length !== ids.length)
    throw new Error("部分场景已不存在，请重新选择");
  const names = scenes.map((scene) => scene.name);
  return sources.map((scene) => {
    const name = uniqueName(`${scene.name} 副本`, names);
    names.push(name);
    return { op: "duplicateScene", id: scene.id, name };
  });
}
