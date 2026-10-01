import type { AudioEdit, AudioLightingClip } from "../../audio-types.ts";
import { audioMilliseconds } from "../../audio-tools.ts";
export interface ClipSplitActions {
  split(time: number): Promise<boolean>;
  resetOffset(): Promise<boolean>;
  readPosition(): Promise<number>;
}
export function clipSplitLimits(clip: AudioLightingClip) {
  return { min: clip.startMs + Math.max(1, clip.fadeMs), max: clip.endMs - 1 };
}
export function splitClipCommand(
  clip: AudioLightingClip,
  text: string,
): AudioEdit {
  if (clip.locked) throw new Error("此灯光片段已锁定，请先解锁");
  const timeMs = audioMilliseconds(text, "分割位置");
  if (timeMs <= clip.startMs || timeMs >= clip.endMs)
    throw new Error("分割位置必须在片段开始与结束之间");
  if (timeMs - clip.startMs < clip.fadeMs)
    throw new Error("进入渐变尚未结束，请选择渐变结束后的分割位置");
  return { kind: "splitLightingClip", id: clip.id, timeMs };
}
