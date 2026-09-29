import type { RefObject } from "react";
import type { AudioMarker, AudioTimeline } from "../../audio-types";
import type { SceneView } from "../../application-host";
export type AudioDraft =
  | { kind: "marker"; id: string; name: string; time: string; sceneId: string }
  | { kind: "trim"; start: string; end: string };
export function markerDraft(marker: AudioMarker): AudioDraft {
  return {
    kind: "marker",
    id: marker.id,
    name: marker.name,
    time: (marker.timeMs / 1000).toFixed(3),
    sceneId: marker.sceneId ?? "",
  };
}
export function AudioInspector({
  track,
  marker,
  draft,
  scenes,
  busy,
  form,
  onChange,
  onApply,
  onCancel,
  onRemove,
}: {
  track: AudioTimeline;
  marker?: AudioMarker;
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
    (marker
      ? markerDraft(marker)
      : {
          kind: "trim" as const,
          start: (track.inMs / 1000).toFixed(3),
          end: (track.outMs / 1000).toFixed(3),
        });
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
          <label>
            灯光场景
            <select
              aria-label="卡点灯光场景"
              value={data.sceneId}
              disabled={busy}
              onChange={(e) => onChange({ ...data, sceneId: e.target.value })}
            >
              <option value="">仅作节奏标记</option>
              {scenes.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.name}
                </option>
              ))}
            </select>
          </label>
          <p>到达此处切换到所选场景，动态效果从该点开始。</p>
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
