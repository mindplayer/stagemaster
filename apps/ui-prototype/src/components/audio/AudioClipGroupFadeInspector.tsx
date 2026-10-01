import { useEffect, useRef, type RefObject } from "react";
import type { AudioTimeline } from "../../audio-types";
import type { ClipGroupFadeDraft } from "./clip-group-fade";

export function AudioClipGroupFadeInspector({
  draft,
  track,
  form,
  busy,
  problem,
  onChange,
  onApply,
  onCancel,
}: {
  draft: ClipGroupFadeDraft;
  track: AudioTimeline;
  form: RefObject<HTMLFormElement | null>;
  busy: boolean;
  problem: string;
  onChange(value: ClipGroupFadeDraft): void;
  onApply(): void;
  onCancel(): void;
}) {
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    input.current?.focus();
  }, []);
  const items =
    track.lightingClips?.filter((c) => draft.ids.includes(c.id)) ?? [];
  const shortest = items.reduce(
    (a, b) => (a.endMs - a.startMs <= b.endMs - b.startMs ? a : b),
    items[0],
  );
  return (
    <form
      ref={form}
      className="audio-inspector"
      aria-label="统一进入渐变"
      onSubmit={(e) => {
        e.preventDefault();
        if (!busy) onApply();
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape" && !busy) {
          e.preventDefault();
          e.stopPropagation();
          onCancel();
        }
      }}
    >
      <h3>统一进入渐变</h3>
      <p role="status">修改已选 {draft.ids.length} 个片段，包含筛选外选择。</p>
      <label>
        渐变（秒）
        <input
          ref={input}
          name="clipGroupFade"
          aria-label="统一进入渐变（秒）"
          inputMode="decimal"
          required
          disabled={busy}
          value={draft.fade}
          placeholder="原值不同，请输入统一值"
          onChange={(e) => onChange({ ...draft, fade: e.target.value })}
        />
      </label>
      {shortest && (
        <small>
          最多 {((shortest.endMs - shortest.startMs) / 1000).toFixed(3)} 秒，受“
          {shortest.name}”限制。
        </small>
      )}
      <small>0 秒为直接切换；保留各段位置、长度和效果起点。</small>
      {problem && (
        <p className="audio-error" role="alert">
          {problem}
        </p>
      )}
      <div className="wb-actions">
        <button type="submit" className="primary" disabled={busy}>
          应用统一渐变
        </button>
        <button type="button" disabled={busy} onClick={onCancel}>
          取消统一渐变
        </button>
      </div>
    </form>
  );
}
