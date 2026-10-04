import type { AudioLightingClip, AudioTimeline } from "../../audio-types.ts";
import { moveIntervalGroup } from "./interval-group-motion.ts";

export interface ClipGroupMotion {
  ids: string[];
  clips: AudioLightingClip[];
  delta: number;
  destination: number;
  problem: string;
}
/** Geometry preview only. Rust validates the complete atomic edit on release. */
export function moveClipGroup(
  track: AudioTimeline,
  ids: string[],
  requested: number,
  tolerance = 0,
): ClipGroupMotion {
  const chosen = new Set(ids);
  const others = (track.lightingClips ?? []).filter((c) => !chosen.has(c.id));
  const targets = [
    0,
    track.outMs - track.inMs,
    ...track.markers.map((m) => m.timeMs),
    ...others.flatMap((c) => [c.startMs, c.endMs]),
  ];
  const { items, ...motion } = moveIntervalGroup(
    track.lightingClips ?? [],
    track.outMs - track.inMs,
    ids,
    requested,
    targets,
    tolerance,
    "片段",
  );
  return { ...motion, clips: items };
}
