import { clamp, normalizeClip } from "./demo-session.ts";
import type { Clip } from "./demo-session.ts";

export type DragMode = "move" | "left" | "right";

/** Screen-space attraction, not a time-step quantizer. */
export function dragClip(
  clip: Clip,
  mode: DragMode,
  deltaPixels: number,
  pixelsPerSecond: number,
  length: number,
  targets: number[],
  snap: boolean,
): { clip: Clip; snapTarget: number | null } {
  const delta = deltaPixels / pixelsPerSecond;
  const end = clip.start + clip.duration;
  let start = clip.start;
  let duration = clip.duration;
  if (mode === "move") start = clamp(start + delta, 0, length - duration);
  if (mode === "left") {
    start = clamp(start + delta, 0, end - 0.2);
    duration = end - start;
  }
  if (mode === "right") duration = clamp(duration + delta, 0.2, length - start);

  let correction = 0;
  let snapTarget: number | null = null;
  let bestPixels = 6;
  if (snap) {
    const edges =
      mode === "move"
        ? [start, start + duration]
        : [mode === "left" ? start : start + duration];
    for (const target of targets) {
      for (const edge of edges) {
        const distance = target - edge;
        const pixels = Math.abs(distance * pixelsPerSecond);
        const valid =
          mode === "move"
            ? start + distance >= 0 && start + distance + duration <= length
            : mode === "left"
              ? target >= 0 && target <= end - 0.2
              : target >= start + 0.2 && target <= length;
        if (valid && pixels <= bestPixels) {
          bestPixels = pixels;
          correction = distance;
          snapTarget = target;
        }
      }
    }
  }
  if (mode !== "right") start += correction;
  if (mode === "left") duration -= correction;
  if (mode === "right") duration += correction;
  return {
    clip: normalizeClip({ ...clip, start, duration }, length),
    snapTarget,
  };
}

export function pointerTime(
  x: number,
  left: number,
  width: number,
  length: number,
) {
  return clamp(((x - left) / Math.max(1, width)) * length, 0, length);
}
