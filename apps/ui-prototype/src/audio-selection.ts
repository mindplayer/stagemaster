import type { AudioTimeline } from "./audio-types";
/** Navigation/display identity only. Playback is evaluated by Rust. */
export function audioSelection(
  track: AudioTimeline | null | undefined,
  id: string,
) {
  const clip = track?.lightingClips?.find((c) => c.id === id);
  return clip
    ? {
        id: clip.id,
        name: clip.name,
        timeMs: clip.startMs,
        sceneId: clip.sceneId,
      }
    : track?.markers.find((m) => m.id === id);
}
