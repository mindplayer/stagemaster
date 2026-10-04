import type { AudioEdit, AudioTimeline } from "../../audio-types";
import type { AudioLoopRegion } from "../../audio-performance-types";
import type { AudioLightingClip, AudioMarker } from "../../audio-types";
import type { OrderedSelection } from "../selection/ordered-selection";
import type { usePerformanceLoopActions } from "./usePerformanceLoopActions";
import type { usePerformanceLoopBatch } from "./usePerformanceLoopBatch";
import type { MarkerLaneSelection } from "./marker-selection";
import type { ClipLaneSelection } from "./clip-selection";
import type { LoopLaneSelection } from "./loop-lane-selection";
import { clipsInRange } from "./clip-selection";

/** Assembly of existing ordered-selection controllers for the shared time view. */
export function useAudioLaneSelections(p: {
  track: AudioTimeline | null;
  identity: string;
  selected: string;
  markerBatch: boolean;
  clipBatch: boolean;
  loopBatch: boolean;
  markerPending: boolean;
  clipPending: boolean;
  markers: OrderedSelection<AudioMarker>;
  clips: OrderedSelection<AudioLightingClip>;
  loops: OrderedSelection<AudioLoopRegion>;
  loopActions: ReturnType<typeof usePerformanceLoopActions>;
  loopGroup: ReturnType<typeof usePerformanceLoopBatch>;
  setBatchKey(value: string): void;
  setMarkerBatch(value: boolean): void;
  edit(command: AudioEdit): Promise<unknown>;
}) {
  const markerSelection: MarkerLaneSelection = {
    active: p.markerBatch,
    ids: p.markers.ids,
    blocked: p.markerPending || p.clipPending || p.loopGroup.pending,
    onMode: () => p.setMarkerBatch(!p.markerBatch),
    onPick: (id, range) => p.markers.toggle(id, p.track?.markers ?? [], range),
    onClear: () => p.markers.replace([]),
  };
  const clipSelection: ClipLaneSelection = {
    active: p.clipBatch,
    ids: p.clips.ids,
    movementBlocked:
      p.clipPending || p.loopGroup.pending
        ? "请先应用或取消右侧目标输入／删除确认"
        : "",
    onMode: () => {
      if (p.loopGroup.pending) return;
      if (!p.clipBatch && !p.clips.ids.length) p.clips.replace([p.selected]);
      p.setBatchKey(p.clipBatch ? "" : `${p.identity}:clips`);
    },
    onPick: (id, range) =>
      p.clips.toggle(id, p.track?.lightingClips ?? [], range),
    onRange: (start, end, append) =>
      p.clips.replace([
        ...(append ? p.clips.ids : []),
        ...clipsInRange(p.track?.lightingClips ?? [], start, end),
      ]),
    onClear: () => p.clips.replace([]),
    onMove: (ids, destinationMs) =>
      void p.edit({
        kind: "editLightingClips",
        ids,
        action: { kind: "move", destinationMs },
      }),
  };
  const loopSelection: LoopLaneSelection = {
    active: p.loopBatch,
    ids: p.loops.ids,
    blocked: p.markerPending || p.clipPending || p.loopGroup.pending,
    onPick: (id, range) =>
      p.loops.toggle(id, p.track?.loopRegions ?? [], range),
    onClear: () => p.loops.replace([]),
    onMove: (_ids, destination) =>
      void p.loopGroup.run("move", (destination / 1000).toFixed(3)),
    onEdit: (original, next, mode) =>
      void p.loopActions.motion(original, next, mode),
  };
  return { markerSelection, clipSelection, loopSelection };
}
