import type { AudioLightingClip } from "../../audio-types";
/** Selection only: half-open time intersection, independent of playback/lock state. */
export function clipsInRange(
  clips: AudioLightingClip[],
  start: number,
  end: number,
) {
  if (!Number.isFinite(start) || !Number.isFinite(end) || start === end)
    return [];
  const left = Math.min(start, end),
    right = Math.max(start, end);
  return clips
    .filter((c) => c.startMs < right && c.endMs > left)
    .map((c) => c.id);
}
export { currentOrderedIds as currentClipIds } from "../selection/ordered-selection.ts";
export interface ClipSelection {
  ids: string[];
  replace(ids: string[]): void;
  toggle(id: string, visible: AudioLightingClip[], range: boolean): void;
}
export interface ClipLaneSelection {
  movementBlocked?: string;
  active: boolean;
  ids: string[];
  onMode(): void;
  onPick(id: string, range: boolean): void;
  onRange(start: number, end: number, append: boolean): void;
  onClear(): void;
  onMove(ids: string[], destinationMs: number): void;
}
