import type {
  AudioEdit,
  AudioLightingClip,
  AudioTimeline,
} from "../../audio-types";
import type { SceneView } from "../../application-host";
import { audioMilliseconds } from "../../audio-tools.ts";
export type ClipStateFilter = "all" | "enabled" | "disabled";
export type ClipGroupOperation =
  "move" | "copy" | "remove" | "enable" | "disable";
export function filteredClips(
  clips: AudioLightingClip[],
  scenes: SceneView[],
  query: string,
  state: ClipStateFilter = "all",
) {
  const term = query.trim().toLocaleLowerCase();
  return clips.filter(
    (c) =>
      (state === "all" ||
        (state === "disabled" ? c.enabled === false : c.enabled !== false)) &&
      `${c.name} ${scenes.find((s) => s.id === c.sceneId)?.name ?? ""}`
        .toLocaleLowerCase()
        .includes(term),
  );
}
export function clipGroupSelection(
  clips: AudioLightingClip[],
  ids: string[],
  visible: AudioLightingClip[],
) {
  const selected = new Set(ids),
    shown = new Set(visible.map((c) => c.id));
  const items = clips.filter((c) => selected.has(c.id));
  return {
    items,
    hidden: items.filter((c) => !shown.has(c.id)).length,
    locked: items.filter((c) => c.locked).length,
    inactive: items.filter((c) => c.enabled === false).length,
    first: items[0]?.startMs ?? 0,
    last: items.at(-1)?.endMs ?? 0,
  };
}
export { toggleOrderedRange as toggleClipRange } from "../selection/ordered-selection.ts";
export function clipGroupCommand(
  track: AudioTimeline,
  ids: string[],
  kind: ClipGroupOperation,
  destination: string,
): AudioEdit {
  const clips = track.lightingClips;
  if (
    !clips ||
    !ids.length ||
    ids.length > 512 ||
    new Set(ids).size !== ids.length ||
    ids.some((id) => !clips.some((c) => c.id === id))
  )
    throw new Error("请重新选择需要整理的灯光片段");
  if (kind === "enable" || kind === "disable")
    return {
      kind: "editLightingClips",
      ids,
      action: { kind: "enabled", enabled: kind === "enable" },
    };
  // Only form/selection validation here; domain collisions and limits belong to Rust.
  if (kind === "remove")
    return { kind: "editLightingClips", ids, action: { kind } };
  return {
    kind: "editLightingClips",
    ids,
    action: { kind, destinationMs: audioMilliseconds(destination, "目标起点") },
  };
}
