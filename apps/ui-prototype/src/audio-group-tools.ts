import type { AudioMarker, AudioTimeline, AudioEdit } from "./audio-types";
import type { SceneView } from "./application-host";
import { audioMilliseconds } from "./audio-tools.ts";
export function filteredAudioMarkers(
  track: AudioTimeline,
  scenes: SceneView[],
  query: string,
): AudioMarker[] {
  const term = query.trim().toLocaleLowerCase();
  return track.markers.filter((m) =>
    `${m.name} ${scenes.find((s) => s.id === m.sceneId)?.name ?? ""}`
      .toLocaleLowerCase()
      .includes(term),
  );
}
export function markerSelection(
  track: AudioTimeline,
  ids: string[],
  visible: AudioMarker[],
) {
  const selected = new Set(ids),
    shown = new Set(visible.map((m) => m.id));
  const markers = track.markers.filter((m) => selected.has(m.id));
  return {
    markers,
    hidden: markers.filter((m) => !shown.has(m.id)).length,
    first: markers[0]?.timeMs ?? 0,
    last: markers.at(-1)?.timeMs ?? 0,
  };
}
export function markerGroupCommand(
  track: AudioTimeline,
  ids: string[],
  kind: "move" | "copy" | "remove",
  destination: string,
): AudioEdit {
  if (
    !ids.length ||
    new Set(ids).size !== ids.length ||
    ids.some((id) => !track.markers.some((m) => m.id === id))
  )
    throw new Error("请重新选择需要整理的卡点");
  if (kind === "remove") return { kind: "editMarkers", ids, action: { kind } };
  const destinationMs = audioMilliseconds(destination, "目标起点");
  if (destinationMs >= track.outMs - track.inMs)
    throw new Error("目标起点须位于音乐范围内");
  return { kind: "editMarkers", ids, action: { kind, destinationMs } };
}
