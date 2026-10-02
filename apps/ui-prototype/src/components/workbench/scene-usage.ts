import type { ProjectView } from "../../application-host";
import { audioTime } from "../../audio-tools.ts";
export type SceneUsageTarget =
  | { kind: "step"; sequenceId: string; id: string }
  | { kind: "clip" | "marker"; id: string };
export interface SceneUsage {
  key: string;
  target: SceneUsageTarget;
  title: string;
  detail: string;
}
export function sceneUsages(
  project: Pick<ProjectView, "sequences" | "audio">,
  sceneId: string,
): SceneUsage[] {
  const result: SceneUsage[] = [];
  for (const list of project.sequences)
    for (const step of list.steps)
      if (step.sceneId === sceneId)
        result.push({
          key: `step:${list.id}:${step.id}`,
          target: { kind: "step", sequenceId: list.id, id: step.id },
          title: `${step.number} · ${step.name}`,
          detail: `执行步骤 · ${list.name}`,
        });
  for (const clip of project.audio?.lightingClips ?? [])
    if (clip.sceneId === sceneId)
      result.push({
        key: `clip:${clip.id}`,
        target: { kind: "clip", id: clip.id },
        title: clip.name,
        detail: `灯光片段 · ${audioTime(clip.startMs)}—${audioTime(clip.endMs)}${clip.enabled === false ? " · 已停用" : ""}${clip.locked ? " · 已锁定" : ""}`,
      });
  for (const clip of project.audio?.lightingClips ?? [])
    if (clip.entryCrossfade?.source.sceneId === sceneId)
      result.push({
        key: `crossfade:${clip.id}`,
        target: { kind: "clip", id: clip.id },
        title: clip.name,
        detail: `交叉来源 · ${audioTime(clip.startMs)}—${audioTime(clip.endMs)}${clip.enabled === false ? " · 已停用" : ""}${clip.locked ? " · 已锁定" : ""}`,
      });
  for (const marker of project.audio?.markers ?? [])
    if (marker.sceneId === sceneId)
      result.push({
        key: `marker:${marker.id}`,
        target: { kind: "marker", id: marker.id },
        title: marker.name,
        detail: `卡点绑定 · ${audioTime(marker.timeMs)}`,
      });
  return result;
}
export function usageMatches(usage: SceneUsage, query: string) {
  return `${usage.title} ${usage.detail}`
    .toLocaleLowerCase()
    .includes(query.trim().toLocaleLowerCase());
}
export function hasSceneUsage(
  project: ProjectView,
  sceneId: string,
  target: SceneUsageTarget,
) {
  return sceneUsages(project, sceneId).some(
    (entry) =>
      entry.target.kind === target.kind &&
      entry.target.id === target.id &&
      (target.kind !== "step" ||
        (entry.target.kind === "step" &&
          entry.target.sequenceId === target.sequenceId)),
  );
}
