import type {
  AudioLoopRange,
  AudioPosition,
  AudioTimeline,
} from "./audio-types.ts";
import { audioMilliseconds } from "./audio-tools.ts";
export class AudioLoopError extends Error {
  field: "start" | "end";
  constructor(field: "start" | "end", message: string) {
    super(message);
    this.field = field;
  }
}
export function audioLoopRange(
  start: string,
  end: string,
  duration: number,
): AudioLoopRange {
  const parse = (value: string, field: "start" | "end") => {
    try {
      return audioMilliseconds(
        value,
        field === "start" ? "循环起点" : "循环终点",
      );
    } catch (error) {
      throw new AudioLoopError(field, String((error as Error).message));
    }
  };
  const startMs = parse(start, "start"),
    endMs = parse(end, "end");
  if (endMs > duration) throw new AudioLoopError("end", "循环终点超出音乐范围");
  if (endMs - startMs < 100 || endMs - startMs > 60000)
    throw new AudioLoopError("end", "循环长度需为 0.100–60 秒");
  return { startMs, endMs };
}
export function markerLoopRange(
  track: AudioTimeline,
  id: string,
): AudioLoopRange | null {
  if (track.lightingClips) {
    const clip = track.lightingClips.find((c) => c.id === id);
    return clip ? { startMs: clip.startMs, endMs: clip.endMs } : null;
  }
  const marker = track.markers.find((m) => m.id === id && m.sceneId);
  if (!marker) return null;
  const next = track.markers
    .filter((m) => m.sceneId && m.timeMs > marker.timeMs)
    .sort((a, b) => a.timeMs - b.timeMs)[0];
  return {
    startMs: marker.timeMs,
    endMs: next?.timeMs ?? track.outMs - track.inMs,
  };
}
/** Visual interpolation only. Never feeds the native transport or lighting. */
export function audioDisplayTime(
  position: AudioPosition,
  delta: number,
): number {
  const time =
    position.positionMs +
    (position.playing ? Math.min(120, Math.max(0, delta)) : 0);
  if (position.performance) {
    if (position.performance.snapshotPending) return position.positionMs;
    return Math.min(position.durationMs, position.performance.boundaryMs, time);
  }
  const range = position.loopRange;
  return range && position.playing
    ? range.startMs + ((time - range.startMs) % (range.endMs - range.startMs))
    : Math.min(position.durationMs, time);
}
