import { ClipFadeModeSelect } from "./ClipFadeModeSelect";
import { AudioClipTimeMode } from "./AudioClipTimeMode";
import { AudioClipSplit } from "./AudioClipSplit";
import type { ClipSplitActions } from "./clip-split-tools";
import type { RefObject } from "react";
import type { SceneView } from "../../application-host";
import type { AudioLightingClip } from "../../audio-types";
import type { ClipDraft } from "./audio-clip-draft";
export function AudioClipInspector({
  data,
  clip,
  scenes,
  busy,
  dirty,
  ready,
  form,
  onChange,
  onApply,
  onCancel,
  onCopy,
  onLock,
  onEnabled,
  splitActions,
  onRemove,
  onPreview,
  onEditScene,
}: {
  data: ClipDraft;
  clip?: AudioLightingClip;
  scenes: SceneView[];
  busy: boolean;
  dirty: boolean;
  ready: boolean;
  form: RefObject<HTMLFormElement | null>;
  onChange(d: ClipDraft): void;
  onApply(): void;
  onCancel(): void;
  onCopy(): void;
  onLock(): void;
  onEnabled(): void;
  splitActions: ClipSplitActions;
  onRemove(): void;
  onPreview(): void;
  onEditScene(): void;
}) {
  const locked = !!clip?.locked && !data.copy;
  const blocked = busy || locked;
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
          e.preventDefault();
          e.stopPropagation();
          onCancel();
        }
      }}
    >
      <h3>
        {data.copy
          ? "复制灯光片段"
          : !data.id
            ? "新建灯光片段"
            : "灯光片段属性"}
      </h3>
      {clip && !data.copy && (
        <button type="button" disabled={busy || dirty} onClick={onLock}>
          {clip.locked ? "解锁片段" : "锁定片段"}
        </button>
      )}
      {clip && !data.copy && (
        <div className="wb-actions">
          <button type="button" disabled={blocked || dirty} onClick={onEnabled}>
            {clip.enabled === false ? "恢复片段" : "停用片段"}
          </button>
          <span role="status">
            {clip.enabled === false ? "已停用 · 使用灯具默认值" : "已启用"}
          </span>
        </div>
      )}
      <label>
        名称
        <input
          name="clipName"
          aria-label="片段名称"
          required
          maxLength={128}
          value={data.name}
          disabled={blocked || data.copy}
          onChange={(e) => onChange({ ...data, name: e.target.value })}
        />
      </label>
      <label>
        灯光场景
        <select
          name="clipScene"
          aria-label="片段灯光场景"
          required
          value={data.sceneId}
          disabled={blocked || data.copy || !!data.preserveEntry}
          onChange={(e) => onChange({ ...data, sceneId: e.target.value })}
        >
          <option value="">选择场景</option>
          {scenes.map((s) => (
            <option key={s.id} value={s.id}>
              {s.name}
            </option>
          ))}
        </select>
      </label>
      <ClipFadeModeSelect value={data.fadeMode ?? "snapshot"} disabled={blocked || data.copy || !!data.preserveEntry}
        onChange={value => onChange({...data, fadeMode: value || "snapshot"})} />
      {clip && !data.copy && (
        <AudioClipTimeMode
          data={data}
          clip={clip}
          disabled={blocked}
          onChange={onChange}
        />
      )}
      {(
        [
          ["start", "clipStart", "开始（秒）"],
          ["end", "clipEnd", "结束（秒）"],
          ["fade", "clipFade", "进入渐变（秒）"],
        ] as const
      ).map(([key, name, label]) => (
        <label key={key}>
          {data.copy && key === "start" ? "复制到（秒）" : label}
          <input
            name={name}
            aria-label={`片段${label}`}
            inputMode="decimal"
            required
            value={
              data.copy && key === "end" && clip
                ? (
                    Number(data.start) +
                    (clip.endMs - clip.startMs) / 1000
                  ).toFixed(3)
                : data[key]
            }
            disabled={
              blocked ||
              (data.copy && key !== "start") ||
              (!!data.preserveEntry && key === "fade")
            }
            onChange={(e) => onChange({ ...data, [key]: e.target.value })}
          />
        </label>
      ))}
      {(clip?.entryFade || clip?.entryCrossfade) &&
        !data.preserveEntry &&
        (Number(data.fade) !== clip.fadeMs / 1000 ||
          data.sceneId !== clip.sceneId || (data.fadeMode ?? "snapshot") !== (clip.fadeMode ?? "snapshot")) && (
          <small>修改场景、方式或渐变时长后，将重新计算进入渐变。</small>
        )}
      <p>
        {data.copy
          ? "复制保留原片段时长、渐变和启停状态，副本解除锁定。"
          : locked
            ? "已锁定位置与内容，解锁后可修改。"
            : data.preserveEntry || clip?.entryFade || clip?.entryCrossfade
              ? "单轨片段不能重叠；保留的渐变随截取范围调整。"
              : "单轨片段不能重叠，进入渐变不能超过新片段长度。"}
      </p>
      <div className="wb-actions">
        <button type="submit" className="primary" disabled={blocked || !dirty}>
          {data.copy ? "确认复制" : "应用"}
        </button>
        <button type="button" disabled={busy || !dirty} onClick={onCancel}>
          取消修改
        </button>
      </div>
      {clip && !data.copy && (
        <>
          <div className="audio-marker-actions">
            <button
              type="button"
              className="primary"
              disabled={busy || !ready}
              onClick={onPreview}
            >
              {clip.enabled === false ? "从此位置预演" : "从此片段预演"}
            </button>
            <button type="button" disabled={busy} onClick={onEditScene}>
              编辑关联场景
            </button>
          </div>
          <div className="wb-actions">
            <button type="button" disabled={busy || dirty} onClick={onCopy}>
              复制片段
            </button>
            <button
              type="button"
              disabled={blocked || dirty}
              onClick={onRemove}
            >
              删除片段
            </button>
          </div>
        </>
      )}
      {clip && !data.copy && (
        <AudioClipSplit
          key={`${clip.id}:${clip.startMs}:${clip.endMs}:${clip.effectOffsetMs ?? 0}`}
          clip={clip}
          disabled={blocked || dirty}
          ready={ready}
          actions={splitActions}
          sourceName={clip.entryCrossfade?.source.sceneId
            ? scenes.find(scene => scene.id === clip.entryCrossfade?.source.sceneId)?.name
            : "灯具默认值"}
        />
      )}
      <small>
        空隙使用灯具默认值。动态交叉期间前后效果持续运行；色盘等离散功能进入片段时直接切换。
      </small>
    </form>
  );
}
