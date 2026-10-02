import "./audio-clip-time.css";
import { clipRestoreStart, entryFadePreview } from "./clip-fade-tools";
import type { AudioLightingClip } from "../../audio-types";
import { audioMilliseconds } from "../../audio-tools";
import type { ClipDraft } from "./audio-clip-draft";
import { trimmedEffectOffset } from "./clip-trim-tools";
export function AudioClipTimeMode({
  data,
  clip,
  disabled,
  onChange,
}: {
  data: ClipDraft;
  clip: AudioLightingClip;
  disabled: boolean;
  onChange(draft: ClipDraft): void;
}) {
  let offset: number | null = null;
  let fade: ReturnType<typeof entryFadePreview> = null;
  if (data.preserveProgress) {
    try {
      offset = trimmedEffectOffset(
        clip,
        audioMilliseconds(data.start, "裁切开始"),
      );
      fade = entryFadePreview(
        clip,
        {
          ...clip,
          sceneId: data.sceneId,
          fadeMode: data.fadeMode,
          fadeMs: audioMilliseconds(data.fade, "渐变"),
          startMs: audioMilliseconds(data.start, "裁切开始"),
          endMs: audioMilliseconds(data.end, "裁切结束"),
        },
        true,
        !!data.preserveEntry,
      );
    } catch {
      /* Detailed field validation happens on apply. */
    }
  }
  return (
    <section className="audio-clip-time-mode" aria-label="片段时间调整方式">
      <label>
        时间调整方式
        <select
          aria-label="片段时间调整方式"
          disabled={disabled}
          value={
            data.preserveProgress
              ? data.preserveEntry
                ? "slice"
                : "trim"
              : "arrange"
          }
          onChange={(e) =>
            onChange({
              ...data,
              preserveProgress: e.target.value !== "arrange",
              preserveEntry: e.target.value === "slice",
              ...(e.target.value === "slice"
                ? {
                    sceneId: clip.sceneId,
                    fadeMode: clip.fadeMode,
                    fade: (clip.fadeMs / 1000).toFixed(3),
                  }
                : {}),
            })
          }
        >
          <option value="arrange">重新安排 · 保持效果起点</option>
          <option value="trim">裁切 · 保留效果进度</option>
          <option value="slice">内部截取 · 同时保留原渐变</option>
        </select>
      </label>
      {data.preserveProgress ? (
        <>
          <small>
            可向左恢复至 {clipRestoreStart(clip, !!data.preserveEntry) / 1000}{" "}
            秒；
            {clip.entryFade || clip.entryCrossfade || data.preserveEntry
              ? "保留原渐变进度。"
              : "进入渐变从新边界开始。"}
          </small>
          {fade && (
            <output>
              待用渐变：{(fade.visibleMs / 1000).toFixed(3)} 秒 · 原渐变已进行{" "}
              {(fade.offsetMs / 1000).toFixed(3)} 秒
            </output>
          )}
          {offset !== null && (
            <output>待用效果起点：{(offset / 1000).toFixed(3)} 秒</output>
          )}
        </>
      ) : (
        <small>
          当前效果起点：{((clip.effectOffsetMs ?? 0) / 1000).toFixed(3)}{" "}
          秒。重新安排保留此起点，端点拖动使用裁切。
        </small>
      )}
    </section>
  );
}
