import type { AudioLightingClip, AudioTimeline } from "../../audio-types.ts";
export type ClipMotion = "move" | "start" | "end";
/** A gesture stays within neighboring intervals; precise input can relocate across them. */
export function moveLightingClip(
  track: AudioTimeline,
  original: AudioLightingClip,
  mode: ClipMotion,
  delta: number,
  snap = false,
  tolerance = 0,
): AudioLightingClip {
  if (original.locked || !Number.isFinite(delta)) return original;
  const others = (track.lightingClips ?? []).filter(
    (c) => c.id !== original.id,
  );
  const previous =
    others.filter((c) => c.endMs <= original.startMs).at(-1)?.endMs ?? 0;
  const next =
    others.find((c) => c.startMs >= original.endMs)?.startMs ??
    track.outMs - track.inMs;
  const length = original.endMs - original.startMs;
  const origin = mode === "end" ? original.endMs : original.startMs;
  const min =
    mode === "end" ? original.startMs + Math.max(1, original.fadeMs) : previous;
  const max =
    mode === "end"
      ? next
      : mode === "move"
        ? next - length
        : original.endMs - Math.max(1, original.fadeMs);
  let value = Math.max(min, Math.min(max, origin + Math.round(delta)));
  if (snap) {
    const targets = [
      0,
      track.outMs - track.inMs,
      ...track.markers.map((m) => m.timeMs),
      ...others.flatMap((c) => [c.startMs, c.endMs]),
    ];
    const candidates = targets
      .flatMap((v) => (mode === "move" ? [v, v - length] : [v]))
      .filter((v) => v >= min && v <= max && Math.abs(v - value) <= tolerance);
    candidates.sort(
      (a, b) => Math.abs(a - value) - Math.abs(b - value) || a - b,
    );
    value = candidates[0] ?? value;
  }
  return {
    ...original,
    startMs: mode === "end" ? original.startMs : value,
    endMs:
      mode === "move"
        ? value + length
        : mode === "end"
          ? value
          : original.endMs,
  };
}
