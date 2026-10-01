import type { AudioTimeline } from "../../audio-types";
import { lightingSegments } from "./lighting-segments.ts";
export interface SelectionViewRange {
  startMs: number;
  endMs: number;
  label: string;
}
export function selectionViewRange(
  track: AudioTimeline,
  selected: string,
  group?: { active: boolean; ids: string[] },
  markers?: { active: boolean; ids: string[] },
): SelectionViewRange | null {
  if (markers?.active) {
    const chosen = new Set(markers.ids);
    const items = track.markers.filter((m) => chosen.has(m.id));
    if (!items.length) return null;
    return {
      startMs: Math.min(...items.map((m) => m.timeMs)),
      endMs: Math.max(...items.map((m) => m.timeMs)),
      label: `${items.length} 个卡点`,
    };
  }
  if (group?.active) {
    const ids = new Set(group.ids);
    const clips = (track.lightingClips ?? []).filter((clip) =>
      ids.has(clip.id),
    );
    if (!clips.length) return null;
    return {
      startMs: Math.min(...clips.map((c) => c.startMs)),
      endMs: Math.max(...clips.map((c) => c.endMs)),
      label: `${clips.length} 个灯光片段`,
    };
  }
  const clip = track.lightingClips?.find((c) => c.id === selected);
  if (clip)
    return { startMs: clip.startMs, endMs: clip.endMs, label: clip.name };
  const marker = track.markers.find((m) => m.id === selected);
  if (!marker) return null;
  if (!track.lightingClips && marker.sceneId) {
    const segment = lightingSegments(track).find(
      (s) => s.markerId === marker.id,
    );
    if (segment)
      return { startMs: segment.start, endMs: segment.end, label: marker.name };
  }
  return { startMs: marker.timeMs, endMs: marker.timeMs, label: marker.name };
}
/** A rendering-only fit with context margins and the existing 400 px/s budget. */
export function fitSelectionView(
  duration: number,
  width: number,
  range: Pick<SelectionViewRange, "startMs" | "endMs">,
) {
  const { startMs, endMs } = range;
  if (
    ![duration, width, startMs, endMs].every(Number.isFinite) ||
    duration <= 0 ||
    width <= 0 ||
    startMs < 0 ||
    endMs < startMs ||
    endMs > duration
  )
    return null;
  const length = endMs - startMs;
  const span = Math.min(
    duration,
    Math.max(length * 1.2, length === 0 ? 3000 : 1000, (width / 400) * 1000),
  );
  const start = Math.max(
    0,
    Math.min(duration - span, (startMs + endMs - span) / 2),
  );
  return {
    start,
    end: start + span,
    ratio: duration / span,
    pixelsPerSecond: (width / span) * 1000,
    scrollPixels: (start / span) * width,
  };
}
