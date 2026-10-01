import type { AudioLightingClip } from "../../audio-types.ts";
import { AudioDraftError } from "./audio-draft-error.ts";

/** Editing feedback only. The core captures values and applies both source clocks. */
export function entryFadePreview(
  source: AudioLightingClip,
  next: AudioLightingClip,
  trim: boolean,
  complete = false,
) {
  const changed =
    source.sceneId !== next.sceneId || source.fadeMs !== next.fadeMs;
  if (complete && changed)
    throw new AudioDraftError(
      "内部截取时请保留原场景和渐变，或使用重新安排",
      source.sceneId !== next.sceneId ? "clipScene" : "clipFade",
    );
  const fade = changed
    ? null
    : (source.entryFade ??
      (complete && source.fadeMs > 0
        ? { durationMs: source.fadeMs, offsetMs: 0 }
        : null));
  if (!fade) return null;
  const offsetMs = fade.offsetMs + (trim ? next.startMs - source.startMs : 0);
  if (!Number.isSafeInteger(offsetMs) || offsetMs < 0)
    throw new AudioDraftError("裁切开始不能早于原渐变零点", "clipStart");
  if (offsetMs + next.endMs - next.startMs > 3_600_000)
    throw new AudioDraftError("保留渐变源范围不能超过 3600 秒", "clipEnd");
  return {
    offsetMs,
    durationMs: fade.durationMs,
    visibleMs: Math.min(
      Math.max(0, fade.durationMs - offsetMs),
      Math.max(0, next.endMs - next.startMs),
    ),
  };
}

export function clipRestoreStart(clip: AudioLightingClip, complete = false) {
  const effect = clip.effectOffsetMs ?? 0;
  const fade =
    clip.entryFade?.offsetMs ?? (complete && clip.fadeMs > 0 ? 0 : effect);
  return Math.max(0, clip.startMs - Math.min(effect, fade));
}
