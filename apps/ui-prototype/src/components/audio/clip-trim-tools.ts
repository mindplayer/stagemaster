import type { AudioEdit, AudioLightingClip } from "../../audio-types.ts";
import { AudioDraftError } from "./audio-draft-error.ts";
/** Authoring feedback only; Rust owns the committed source-clock calculation. */
export function trimmedEffectOffset(
  previous: AudioLightingClip,
  startMs: number,
): number {
  const offset = (previous.effectOffsetMs ?? 0) + startMs - previous.startMs;
  if (!Number.isSafeInteger(offset) || offset < 0)
    throw new AudioDraftError(
      "裁切开始不能早于效果源零点，请使用重新安排或移动片段",
      "clipStart",
    );
  if (offset > 3_600_000)
    throw new AudioDraftError("裁切后的效果起点超出 3600 秒", "clipStart");
  return offset;
}
export function clipMotionCommand(
  clip: AudioLightingClip,
  mode: "move" | "start" | "end",
): AudioEdit {
  // Proposals intentionally retain the old offset; only Rust may update it.
  return {
    kind: mode === "move" ? "putLightingClip" : "trimLightingClip",
    clip,
  };
}
