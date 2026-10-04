import type { AudioEdit, AudioTimeline } from "../../audio-types.ts";
import type { AudioLoopRegion } from "../../audio-performance-types.ts";
import { moveIntervalGroup } from "./interval-group-motion.ts";
import { sameLoopRegion } from "./performance-loop-draft.ts";
export type LoopMotion = "move" | "start" | "end";
function targets(track: AudioTimeline, ids: string[]) {
  const chosen = new Set(ids);
  return [
    0,
    track.outMs - track.inMs,
    ...track.markers.map((m) => m.timeMs),
    ...(track.lightingClips ?? []).flatMap((c) => [c.startMs, c.endMs]),
    ...(track.loopRegions ?? [])
      .filter((r) => !chosen.has(r.id))
      .flatMap((r) => [r.startMs, r.endMs]),
  ];
}
export function moveLoopGroup(
  track: AudioTimeline,
  ids: string[],
  delta: number,
  tolerance = 0,
) {
  return moveIntervalGroup(
    track.loopRegions ?? [],
    track.outMs - track.inMs,
    ids,
    delta,
    targets(track, ids),
    tolerance,
    "循环区段",
  );
}
export function moveLoopRegion(
  track: AudioTimeline,
  region: AudioLoopRegion,
  mode: LoopMotion,
  requested: number,
  tolerance = 0,
): AudioLoopRegion {
  if (region.locked || !Number.isFinite(requested)) return region;
  const regions = track.loopRegions ?? [],
    index = regions.findIndex((r) => r.id === region.id);
  if (index < 0) return region;
  const previous = regions[index - 1]?.endMs ?? 0;
  const next = regions[index + 1]?.startMs ?? track.outMs - track.inMs;
  const length = region.endMs - region.startMs;
  const origin = mode === "end" ? region.endMs : region.startMs;
  const min = mode === "end" ? region.startMs + 1 : previous;
  const max =
    mode === "start"
      ? region.endMs - 1
      : mode === "move"
        ? next - length
        : next;
  let value = Math.max(min, Math.min(max, Math.round(origin + requested)));
  if (tolerance > 0) {
    const candidates = targets(track, [region.id])
      .flatMap((t) => (mode === "move" ? [t, t - length] : [t]))
      .filter((t) => t >= min && t <= max && Math.abs(t - value) <= tolerance)
      .sort((a, b) => Math.abs(a - value) - Math.abs(b - value) || a - b);
    value = candidates[0] ?? value;
  }
  return {
    ...region,
    startMs: mode === "end" ? region.startMs : value,
    endMs:
      mode === "start"
        ? region.endMs
        : mode === "move"
          ? value + length
          : value,
  };
}
export function loopMotionCommand(
  track: AudioTimeline,
  original: AudioLoopRegion,
  next: AudioLoopRegion,
  mode: LoopMotion,
): AudioEdit {
  const current = track.loopRegions?.find((r) => r.id === original.id);
  if (!current || !sameLoopRegion(current, original))
    throw new Error("区段已变化，请重新选择后编辑");
  if (current.locked) throw new Error("区段已锁定，请先解锁");
  return {
    kind: "loopRegions",
    command:
      mode === "move"
        ? {
            kind: "edit",
            ids: [current.id],
            action: { kind: "move", destinationMs: next.startMs },
          }
        : {
            kind: "put",
            region: { ...current, startMs: next.startMs, endMs: next.endMs },
          },
  };
}
