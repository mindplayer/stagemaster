import type { AudioEdit, AudioTimeline } from "../../audio-types.ts";
import type { AudioLoopPlays, AudioLoopRegion } from "../../audio-performance-types.ts";
import { audioMilliseconds } from "../../audio-tools.ts";
import { AudioDraftError } from "./audio-draft-error.ts";

export interface PerformanceLoopDraft {
  kind: "performanceLoop";
  original: AudioLoopRegion | null;
  name: string;
  start: string;
  end: string;
  mode: AudioLoopPlays["kind"];
  count: string;
}
export function performanceLoopDraft(region: AudioLoopRegion): PerformanceLoopDraft {
  return {
    kind: "performanceLoop",
    original: structuredClone(region),
    name: region.name,
    start: (region.startMs / 1000).toFixed(3),
    end: (region.endMs / 1000).toFixed(3),
    mode: region.plays.kind,
    count: region.plays.kind === "count" ? String(region.plays.count) : "2",
  };
}
export function newPerformanceLoop(track: AudioTimeline, position: number): PerformanceLoopDraft {
  const duration = track.outMs - track.inMs;
  let start = Math.max(0, Math.min(duration, Math.round(position)));
  let end = Math.min(start + 1000, duration);
  for (const region of track.loopRegions ?? []) {
    if (region.endMs <= start) continue;
    if (region.startMs > start) {
      end = Math.min(start + 1000, region.startMs);
      break;
    }
    start = region.endMs;
    end = Math.min(start + 1000, duration);
  }
  if (start >= end)
    throw new AudioDraftError("播放头之后没有可用空隙，请移动播放头或缩短现有区段", "loopStart");
  return {
    kind: "performanceLoop",
    original: null,
    name: `循环区段 ${(track.loopRegions?.length ?? 0) + 1}`,
    start: (start / 1000).toFixed(3),
    end: (end / 1000).toFixed(3),
    mode: "count",
    count: "2",
  };
}
export function loopPlaysLabel(plays: AudioLoopPlays) {
  return plays.kind === "untilExit" ? "持续循环" : `共播放 ${plays.count} 次`;
}
function sameRegion(a: AudioLoopRegion, b: AudioLoopRegion) {
  return (
    a.id === b.id && a.name === b.name &&
    a.startMs === b.startMs && a.endMs === b.endMs &&
    a.enabled === b.enabled && a.locked === b.locked &&
    a.plays.kind === b.plays.kind &&
    (a.plays.kind !== "count" ||
      (b.plays.kind === "count" && a.plays.count === b.plays.count))
  );
}
export function collectPerformanceLoop(draft: PerformanceLoopDraft, track: AudioTimeline): AudioEdit {
  let field = "loopName";
  try {
    const source = draft.original && track.loopRegions?.find(
      (r) => r.id === draft.original?.id,
    );
    if (draft.original && !source) throw new Error("循环区段已删除，请重新选择");
    if (source?.locked) throw new Error("循环区段已锁定，请先取消输入并解锁");
    if (source && draft.original && !sameRegion(source, draft.original))
      throw new Error("循环区段已变化，请取消输入后重新编辑");
    if (!draft.name.trim()) throw new Error("循环区段名称不能为空");
    if (!source && (track.loopRegions?.length ?? 0) >= 128)
      throw new Error("演出循环最多 128 个区段");
    field = "loopStart";
    const startMs = audioMilliseconds(draft.start, "区段开始");
    field = "loopEnd";
    const endMs = audioMilliseconds(draft.end, "区段结束");
    if (endMs <= startMs || endMs > track.outMs - track.inMs)
      throw new Error("结束须晚于开始，且位于音乐范围内");
    const conflict = track.loopRegions?.find(
      (r) => r.id !== source?.id && startMs < r.endMs && endMs > r.startMs,
    );
    if (conflict) {
      field = conflict.startMs >= startMs ? "loopEnd" : "loopStart";
      throw new Error(`与区段“${conflict.name}”重叠，请调整范围`);
    }
    field = "loopCount";
    if (draft.mode === "count" && (
      !/^\d+$/.test(draft.count.trim()) ||
      Number(draft.count) < 1 || Number(draft.count) > 4294967295
    ))
      throw new Error("总播放次数须为 1–4294967295 的整数，包含第一遍");
    const plays: AudioLoopPlays = draft.mode === "untilExit"
      ? { kind: "untilExit" }
      : { kind: "count", count: Number(draft.count) };
    const value = { name: draft.name.trim(), startMs, endMs, plays };
    return {
      kind: "loopRegions",
      command: source
        ? { kind: "put", region: { ...source, ...value } }
        : { kind: "add", ...value },
    };
  } catch (error) {
    throw new AudioDraftError(error instanceof Error ? error.message : String(error), field);
  }
}
