import "./audio-clip-time.css";
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
  if (data.preserveProgress) {
    try {
      offset = trimmedEffectOffset(
        clip,
        audioMilliseconds(data.start, "裁切开始"),
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
          value={data.preserveProgress ? "trim" : "arrange"}
          onChange={(e) =>
            onChange({ ...data, preserveProgress: e.target.value === "trim" })
          }
        >
          <option value="arrange">重新安排 · 保持效果起点</option>
          <option value="trim">裁切 · 保留效果进度</option>
        </select>
      </label>
      {data.preserveProgress ? (
        <>
          <small>
            可向左恢复至{" "}
            {Math.max(0, clip.startMs - (clip.effectOffsetMs ?? 0)) / 1000}{" "}
            秒；进入渐变从新边界开始。
          </small>
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
