import type { AudioLightingClip, AudioTimeline } from "../../audio-types.ts";

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
  const source = (track.lightingClips ?? []).filter((c) => chosen.has(c.id));
  const others = (track.lightingClips ?? []).filter((c) => !chosen.has(c.id));
  const first = source[0]?.startMs ?? 0,
    last = source.at(-1)?.endMs ?? 0;
  const result = {
    ids: source.map((c) => c.id),
    clips: source,
    delta: 0,
    destination: first,
    problem: "",
  };
  if (
    !source.length ||
    source.length !== ids.length ||
    !Number.isFinite(requested)
  )
    return { ...result, problem: "请重新选择需要移动的片段" };
  const locked = source.find((c) => c.locked);
  if (locked)
    return { ...result, problem: `“${locked.name}”已锁定，整组未移动` };
  const min = -first,
    max = track.outMs - track.inMs - last;
  let delta = Math.max(min, Math.min(max, Math.round(requested)));
  // Sorted disjoint source intervals allow a linear collision pass, even at 512 clips.
  const collision = (offset: number) => {
    let i = 0;
    for (const c of source) {
      const start = c.startMs + offset,
        end = c.endMs + offset;
      while (others[i] && others[i].endMs <= start) i++;
      if (others[i] && others[i].startMs < end) return others[i];
    }
    return undefined;
  };
  if (tolerance > 0) {
    const targets = [
      0,
      track.outMs - track.inMs,
      ...track.markers.map((m) => m.timeMs),
      ...others.flatMap((c) => [c.startMs, c.endMs]),
    ];
    const candidates = [
      ...new Set(targets.flatMap((t) => [t - first, t - last])),
    ]
      .filter((d) => d >= min && d <= max && Math.abs(d - delta) <= tolerance)
      .sort((a, b) => Math.abs(a - delta) - Math.abs(b - delta) || a - b);
    delta = candidates.find((d) => !collision(d)) ?? delta;
  }
  const conflict = collision(delta);
  return {
    ids: result.ids,
    clips: source.map((c) => ({
      ...c,
      startMs: c.startMs + delta,
      endMs: c.endMs + delta,
    })),
    delta,
    destination: first + delta,
    problem: conflict ? `目标与“${conflict.name}”重叠，整组未移动` : "",
  };
}
