import type {
  AudioEdit,
  AudioLightingClip,
  AudioTimeline,
} from "../../audio-types.ts";
import { audioMilliseconds } from "../../audio-tools.ts";
import { AudioDraftError } from "./audio-draft-error.ts";
export interface ClipDraft {
  kind: "clip";
  id: string | null;
  copy: boolean;
  name: string;
  sceneId: string;
  start: string;
  end: string;
  fade: string;
}
export function clipDraft(c: AudioLightingClip): ClipDraft {
  return {
    kind: "clip",
    id: c.id,
    copy: false,
    name: c.name,
    sceneId: c.sceneId,
    start: (c.startMs / 1000).toFixed(3),
    end: (c.endMs / 1000).toFixed(3),
    fade: (c.fadeMs / 1000).toFixed(3),
  };
}
export function validateClip(
  c: AudioLightingClip,
  track: AudioTimeline,
  except = c.id,
) {
  if (
    ![c.startMs, c.endMs, c.fadeMs].every(
      (v) => Number.isSafeInteger(v) && v >= 0,
    )
  )
    throw new AudioDraftError("片段时间必须是非负整数毫秒", "clipStart");
  if (c.endMs <= c.startMs || c.endMs > track.outMs - track.inMs)
    throw new AudioDraftError("结束须晚于开始，且位于音乐范围内", "clipEnd");
  if (c.fadeMs > c.endMs - c.startMs)
    throw new AudioDraftError("进入渐变不能超过片段长度", "clipFade");
  if ((c.effectOffsetMs ?? 0) + c.endMs - c.startMs > 3_600_000)
    throw new AudioDraftError(
      "效果起点与片段长度之和不能超过 3600 秒",
      "clipEnd",
    );
  const conflict = track.lightingClips?.find(
    (other) =>
      other.id !== except && c.startMs < other.endMs && other.startMs < c.endMs,
  );
  if (conflict)
    throw new AudioDraftError(
      `与片段“${conflict.name}”重叠，请调整开始或结束`,
      conflict.startMs >= c.startMs ? "clipEnd" : "clipStart",
    );
}
export function collectClipDraft(
  d: ClipDraft,
  track: AudioTimeline,
): AudioEdit {
  let field = "clipName";
  try {
    if (!track.lightingClips) throw new Error("请先转换独立片段");
    const source = track.lightingClips.find((c) => c.id === d.id);
    if (d.id && !source) throw new Error("片段已删除，请重新选择");
    if (source?.locked && !d.copy) throw new Error("片段已锁定，请先解锁");
    if (!d.name.trim()) throw new Error("片段名称不能为空");
    field = "clipScene";
    if (!d.sceneId) throw new Error("请选择灯光场景");
    field = "clipStart";
    const startMs = audioMilliseconds(d.start, "片段开始");
    field = "clipEnd";
    const endMs =
      d.copy && source
        ? startMs + source.endMs - source.startMs
        : audioMilliseconds(d.end, "片段结束");
    field = "clipFade";
    const fadeMs =
      d.copy && source ? source.fadeMs : audioMilliseconds(d.fade, "进入渐变");
    const clip = {
      id: d.id ?? "",
      name: d.name.trim(),
      sceneId: d.sceneId,
      startMs,
      endMs,
      fadeMs,
      locked: source?.locked ?? false,
      ...(source?.effectOffsetMs
        ? { effectOffsetMs: source.effectOffsetMs }
        : {}),
      ...(source?.enabled === false ? { enabled: false } : {}),
    };
    validateClip(clip, track, d.copy ? "" : clip.id);
    if (d.copy && source)
      return { kind: "copyLightingClip", id: source.id, startMs };
    if (d.id) return { kind: "putLightingClip", clip };
    if (track.lightingClips.length >= 512)
      throw new Error("灯光片段最多 512 个");
    return {
      kind: "addLightingClip",
      name: clip.name,
      sceneId: clip.sceneId,
      startMs,
      endMs,
      fadeMs,
    };
  } catch (error) {
    if (error instanceof AudioDraftError) {
      if (d.copy && error.field === "clipEnd")
        throw new AudioDraftError(error.message, "clipStart");
      throw error;
    }
    throw new AudioDraftError(
      error instanceof Error ? error.message : String(error),
      field,
    );
  }
}
/** Find an available interval without moving existing work or the audio cursor. */
export function nextClipGap(track: AudioTimeline, from: number) {
  let startMs = Math.max(0, Math.round(from));
  for (const c of track.lightingClips ?? []) {
    if (c.endMs <= startMs) continue;
    if (c.startMs > startMs)
      return { startMs, endMs: Math.min(startMs + 5000, c.startMs) };
    startMs = c.endMs;
  }
  return startMs < track.outMs - track.inMs
    ? { startMs, endMs: Math.min(startMs + 5000, track.outMs - track.inMs) }
    : null;
}
