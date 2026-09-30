import type { AudioTimeline } from "../../audio-types";

export interface LightingSegment {
  markerId: string | null;
  sceneId: string | null;
  start: number;
  end: number;
}

/** A display projection of existing hard-cut markers, never a playback evaluator. */
export function lightingSegments(track: AudioTimeline): LightingSegment[] {
  const duration = track.outMs - track.inMs;
  const markers = track.markers
    .filter((m) => m.sceneId)
    .slice()
    .sort((a, b) => a.timeMs - b.timeMs);
  const result: LightingSegment[] = [];
  const first = markers[0]?.timeMs ?? duration;
  if (first > 0)
    result.push({ markerId: null, sceneId: null, start: 0, end: first });
  markers.forEach((marker, i) =>
    result.push({
      markerId: marker.id,
      sceneId: marker.sceneId,
      start: marker.timeMs,
      end: markers[i + 1]?.timeMs ?? duration,
    }),
  );
  return result;
}

/** Keep lighting order and unique millisecond markers when adjusting a shared boundary. */
export function constrainBoundaryTime(
  track: AudioTimeline,
  id: string,
  raw: number,
): number {
  const marker = track.markers.find((m) => m.id === id);
  if (!marker) throw new Error("灯光段落已删除，请重新选择");
  if (!Number.isFinite(raw)) return marker.timeMs;
  const bound = track.markers
    .filter((m) => m.sceneId)
    .slice()
    .sort((a, b) => a.timeMs - b.timeMs);
  const index = bound.findIndex((m) => m.id === id);
  if (index < 0) throw new Error("此卡点未绑定灯光场景");
  const min = index > 0 ? bound[index - 1].timeMs + 1 : 0;
  const max = (bound[index + 1]?.timeMs ?? track.outMs - track.inMs) - 1;
  const time = Math.max(min, Math.min(max, Math.round(raw)));
  const occupied = new Set(
    track.markers.filter((m) => m.id !== id).map((m) => m.timeMs),
  );
  const direction = time >= marker.timeMs ? 1 : -1;
  for (let distance = 0; distance <= occupied.size + 1; distance++) {
    for (const candidate of [
      time + direction * distance,
      time - direction * distance,
    ]) {
      if (candidate >= min && candidate <= max && !occupied.has(candidate))
        return candidate;
    }
  }
  return marker.timeMs;
}
