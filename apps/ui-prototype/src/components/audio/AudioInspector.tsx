import { AudioClipInspector } from "./AudioClipInspector";
import { clipDraft } from "./audio-clip-draft";
import type { AudioLightingClip } from "../../audio-types";
import type { RefObject } from "react";
import type { AudioMarker, AudioTimeline } from "../../audio-types";
import type { SceneView } from "../../application-host";
import { audioFadeLimit } from "../../audio-transition-tools";
import { markerDraft, type AudioDraft } from "./audio-inspector-draft";
export type { AudioDraft } from "./audio-inspector-draft";
export function AudioInspector({
  track,
  marker,
  clip,
  onCopy,
  onLock,
  onEnabled,
  draft,
  scenes,
  busy,
  form,
  onChange,
  onApply,
  onCancel,
  onRemove,
  ready,
  onPreview,
  onEditScene,
}: {
  ready: boolean;
  onPreview(): void;
  onEditScene(): void;
  track: AudioTimeline;
  marker?: AudioMarker;
  clip?: AudioLightingClip;
  onCopy(): void;
  onLock(): void;
  onEnabled(): void;
  draft: AudioDraft | null;
  scenes: SceneView[];
  busy: boolean;
  form: RefObject<HTMLFormElement | null>;
  onChange(value: AudioDraft): void;
  onApply(): void;
  onCancel(): void;
  onRemove(): void;
}) {
  const data =
    draft ??
    (clip
      ? clipDraft(clip)
      : marker
        ? markerDraft(marker)
        : {
            kind: "trim" as const,
            start: (track.inMs / 1000).toFixed(3),
            end: (track.outMs / 1000).toFixed(3),
          });
  if (data.kind === "clip")
    return (
      <AudioClipInspector
        data={data}
        clip={clip}
        scenes={scenes}
        busy={busy}
        dirty={!!draft}
        ready={ready}
        form={form}
        onChange={onChange}
        onApply={onApply}
        onCancel={onCancel}
        onCopy={onCopy}
        onLock={onLock}
        onEnabled={onEnabled}
        onRemove={onRemove}
        onPreview={onPreview}
        onEditScene={onEditScene}
      />
    );
  return (
    <form
      className="audio-inspector"
      ref={form}
      onSubmit={(e) => {
        e.preventDefault();
        onApply();
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.stopPropagation();
          onCancel();
        }
      }}
    >
      <h3>{data.kind === "marker" ? "卡点属性" : "音乐范围"}</h3>
      {data.kind === "marker" ? (
        <>
          <label>
            名称
            <input
              name="markerName"
              aria-label="卡点名称"
              maxLength={128}
              required
              value={data.name}
              disabled={busy}
              onChange={(e) => onChange({ ...data, name: e.target.value })}
            />
          </label>
          <label>
            时间（秒）
            <input
              name="markerTime"
              aria-label="卡点时间（秒）"
              inputMode="decimal"
              required
              value={data.time}
              disabled={busy}
              onChange={(e) => onChange({ ...data, time: e.target.value })}
            />
          </label>
          {!track.lightingClips && (
            <label>
              灯光场景
              <select
                aria-label="卡点灯光场景"
                value={data.sceneId}
                disabled={busy}
                onChange={(e) =>
                  onChange({
                    ...data,
                    sceneId: e.target.value,
                    fade: e.target.value ? data.fade : "0",
                  })
                }
              >
                <option value="">仅作节奏标记</option>
                {scenes.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.name}
                  </option>
                ))}
              </select>
            </label>
          )}
          {data.sceneId && (
            <>
              <label>
                进入方式
                <select
                  aria-label="灯光进入方式"
                  disabled={busy}
                  value={data.fade === "0" ? "cut" : "fade"}
                  onChange={(e) =>
                    onChange({
                      ...data,
                      fade:
                        e.target.value === "cut"
                          ? "0"
                          : (
                              Math.min(
                                1000,
                                marker ? audioFadeLimit(track, marker) : 1000,
                              ) / 1000
                            ).toFixed(3),
                    })
                  }
                >
                  <option value="cut">直接切换</option>
                  <option value="fade">渐变进入</option>
                </select>
              </label>
              {data.fade !== "0" && (
                <label>
                  渐变（秒）
                  <input
                    name="markerFade"
                    aria-label="灯光渐变（秒）"
                    inputMode="decimal"
                    required
                    disabled={busy}
                    value={data.fade}
                    onChange={(e) =>
                      onChange({ ...data, fade: e.target.value })
                    }
                  />
                </label>
              )}
              {marker && (
                <small>
                  本段最多 {(audioFadeLimit(track, marker) / 1000).toFixed(3)}{" "}
                  秒；功能档位仍在卡点直接切换。
                </small>
              )}
            </>
          )}
          <div className="audio-marker-actions">
            <button
              type="button"
              className="primary"
              disabled={busy || !ready}
              onClick={onPreview}
            >
              从此卡点预演
            </button>
            <button
              type="button"
              disabled={busy || !scenes.some((s) => s.id === data.sceneId)}
              onClick={onEditScene}
            >
              编辑关联场景
            </button>
          </div>
          <p>
            {track.lightingClips
              ? "节奏标记与灯光片段独立，移动标记不会改变片段。"
              : "到达卡点开始所选场景；渐变从前段边界状态进入，新效果从此处计时。修改关联场景也会影响其他引用位置。"}
          </p>
        </>
      ) : (
        <>
          <label>
            源文件开始（秒）
            <input
              name="trimStart"
              aria-label="音乐裁切开始（秒）"
              required
              value={data.start}
              disabled={busy}
              onChange={(e) => onChange({ ...data, start: e.target.value })}
            />
          </label>
          <label>
            源文件结束（秒）
            <input
              name="trimEnd"
              aria-label="音乐裁切结束（秒）"
              required
              value={data.end}
              disabled={busy}
              onChange={(e) => onChange({ ...data, end: e.target.value })}
            />
          </label>
          <p>卡点时间以裁切后的起点计算。缩短范围前需移走超出部分的卡点。</p>
        </>
      )}
      <div className="wb-actions">
        <button className="primary" disabled={busy || !draft} type="submit">
          应用
        </button>
        <button type="button" disabled={busy || !draft} onClick={onCancel}>
          取消修改
        </button>
      </div>
      {marker && (
        <button type="button" disabled={busy} onClick={onRemove}>
          删除此卡点
        </button>
      )}
    </form>
  );
}
