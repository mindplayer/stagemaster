import type { AudioMarker, AudioTimeline } from "./audio-types.ts";

export function audioFadeLimit(
  track: AudioTimeline,
  marker: AudioMarker,
): number {
  const end = track.markers
    .filter(
      (next) =>
        next.sceneId && next.id !== marker.id && next.timeMs > marker.timeMs,
    )
    .reduce(
      (end, next) => Math.min(end, next.timeMs),
      track.outMs - track.inMs,
    );
  return Math.max(0, end - marker.timeMs);
}

/** Editor-side diagnostics only; the Rust domain owns accepted timing and playback. */
export function validateAudioTransitions(track: AudioTimeline): void {
  for (const marker of track.markers) {
    const fade = marker.fadeMs ?? 0;
    if (!Number.isSafeInteger(fade) || fade < 0)
      throw new Error(`“${marker.name}”的渐变时间需要非负毫秒数`);
    if (!fade) continue;
    const limit = audioFadeLimit(track, marker);
    if (!marker.sceneId || fade > limit)
      throw new Error(
        `“${marker.name}”的渐变须绑定场景，且不能超过下一灯光段落或音乐结束（最多 ${(limit / 1000).toFixed(3)} 秒）`,
      );
  }
}
