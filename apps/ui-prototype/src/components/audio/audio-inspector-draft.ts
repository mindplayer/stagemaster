import { AudioDraftError } from "./audio-draft-error.ts";
export { AudioDraftError } from "./audio-draft-error.ts";
import { collectClipDraft, type ClipDraft } from "./audio-clip-draft.ts";
import type { AudioEdit, AudioMarker, AudioTimeline } from "../../audio-types";
import { audioMilliseconds, validateMarker } from "../../audio-tools.ts";
import { validateAudioTransitions } from "../../audio-transition-tools.ts";
import { collectPerformanceLoop, type PerformanceLoopDraft } from "./performance-loop-draft.ts";

import {
  collectGroupFade,
  type ClipGroupFadeDraft,
} from "./clip-group-fade.ts";

export type AudioDraft =
  | PerformanceLoopDraft
  | ClipGroupFadeDraft
  | ClipDraft
  | {
      kind: "marker";
      id: string;
      name: string;
      time: string;
      sceneId: string;
      fade: string;
    }
  | { kind: "trim"; start: string; end: string };
export function markerDraft(marker: AudioMarker): AudioDraft {
  return {
    kind: "marker",
    id: marker.id,
    name: marker.name,
    time: (marker.timeMs / 1000).toFixed(3),
    sceneId: marker.sceneId ?? "",
    fade: marker.fadeMs ? (marker.fadeMs / 1000).toFixed(3) : "0",
  };
}
export function collectAudioDraft(
  value: AudioDraft,
  track: AudioTimeline,
): AudioEdit {
  if (value.kind === "performanceLoop") return collectPerformanceLoop(value, track);
  if (value.kind === "clipGroupFade") return collectGroupFade(value, track);
  if (value.kind === "clip") return collectClipDraft(value, track);
  let field = value.kind === "marker" ? "markerName" : "trimStart";
  try {
    if (value.kind === "marker") {
      if (!value.name.trim()) throw new Error("卡点名称不能为空");
      field = "markerTime";
      const timeMs = audioMilliseconds(value.time, "卡点时间");
      const moved = {
        ...track.markers.find((m) => m.id === value.id),
        id: value.id,
        name: value.name.trim(),
        timeMs,
        sceneId: value.sceneId || null,
      };
      // Time changes must also respect an earlier segment's fade.
      validateMarker({ ...moved, fadeMs: 0 }, track);
      field = "markerFade";
      const fadeMs = value.sceneId
        ? audioMilliseconds(value.fade, "灯光渐变")
        : 0;
      const marker = validateMarker({ ...moved, fadeMs }, track);
      if (!fadeMs) delete marker.fadeMs;
      return { kind: "putMarker", marker };
    }
    const inMs = audioMilliseconds(value.start, "裁切开始");
    field = "trimEnd";
    const outMs = audioMilliseconds(value.end, "裁切结束");
    if (inMs >= outMs || outMs > track.asset.durationMs)
      throw new Error("裁切范围必须在源文件内，结束晚于开始");
    if (track.markers.some((m) => m.timeMs >= outMs - inMs))
      throw new Error("裁切后部分卡点超出音乐，请先移动或删除这些卡点");
    if (track.lightingClips?.some((c) => c.endMs > outMs - inMs))
      throw new Error("裁切后部分灯光片段超出音乐，请先移动或缩短这些片段");
    if (track.loopRegions?.some((r) => r.endMs > outMs - inMs))
      throw new Error("裁切后部分循环区段超出音乐，请先移动或缩短这些区段");
    validateAudioTransitions({ ...track, inMs, outMs });
    return { kind: "trim", inMs, outMs };
  } catch (error) {
    throw new AudioDraftError(
      error instanceof Error ? error.message : String(error),
      field,
    );
  }
}
