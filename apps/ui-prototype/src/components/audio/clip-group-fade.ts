import type { AudioEdit, AudioTimeline } from "../../audio-types";
import { audioMilliseconds } from "../../audio-tools.ts";
import { AudioDraftError } from "./audio-draft-error.ts";

export interface ClipGroupFadeDraft {
  kind: "clipGroupFade";
  ids: string[];
  fade: string;
}
export function groupFadeItems(track: AudioTimeline, ids: string[]) {
  const selected = new Set(ids);
  if (!ids.length || ids.length > 512 || selected.size !== ids.length)
    throw new Error("请选择 1–512 个不重复的灯光片段");
  const items = track.lightingClips?.filter((c) => selected.has(c.id)) ?? [];
  if (items.length !== ids.length)
    throw new Error("选中的灯光片段已不存在，请重新选择");
  const locked = items.find((c) => c.locked);
  if (locked) throw new Error(`片段“${locked.name}”已锁定，请先解锁或移出选择`);
  return items;
}
export function groupFadeDraft(
  track: AudioTimeline,
  ids: string[],
): ClipGroupFadeDraft {
  const items = groupFadeItems(track, ids);
  return {
    kind: "clipGroupFade",
    ids: [...ids],
    fade: items.every((c) => c.fadeMs === items[0].fadeMs)
      ? (items[0].fadeMs / 1000).toFixed(3)
      : "",
  };
}
export function collectGroupFade(
  value: ClipGroupFadeDraft,
  track: AudioTimeline,
): AudioEdit {
  try {
    const items = groupFadeItems(track, value.ids);
    const fadeMs = audioMilliseconds(value.fade, "统一进入渐变");
    const short = items.find((c) => fadeMs > c.endMs - c.startMs);
    if (short)
      throw new Error(`进入渐变超出片段“${short.name}”的长度，请缩短渐变`);
    return {
      kind: "editLightingClips",
      ids: [...value.ids],
      action: { kind: "fade", fadeMs },
    };
  } catch (error) {
    throw new AudioDraftError(
      error instanceof Error ? error.message : String(error),
      "clipGroupFade",
    );
  }
}
