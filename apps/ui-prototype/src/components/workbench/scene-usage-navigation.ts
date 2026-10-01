import type { ProjectView } from "../../application-host";
import { hasSceneUsage, type SceneUsageTarget } from "./scene-usage.ts";
export async function locateSceneUsage({
  projectId,
  sceneId,
  target,
  read,
  run,
  reveal,
  open,
}: {
  projectId: string;
  sceneId: string;
  target: SceneUsageTarget;
  read(): ProjectView | null;
  run(action: () => Promise<void>): Promise<boolean>;
  reveal(target: SceneUsageTarget): boolean;
  open(kind: "sequences" | "audio"): void;
}) {
  return run(async () => {
    const project = read();
    if (
      !project ||
      project.id !== projectId ||
      !hasSceneUsage(project, sceneId, target)
    )
      throw new Error("该使用位置已变化，请重新选择");
    if (!reveal(target)) throw new Error("无法定位使用位置，请重新选择");
    open(target.kind === "step" ? "sequences" : "audio");
  });
}
