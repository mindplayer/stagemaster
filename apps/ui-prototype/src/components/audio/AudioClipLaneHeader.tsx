import type { ClipLaneSelection } from "./clip-selection";
export function AudioClipLaneHeader({
  count,
  selection,
  disabled,
  moving,
  onTool,
}: {
  count: number;
  selection?: ClipLaneSelection;
  disabled: boolean;
  moving: boolean;
  onTool(value: boolean): void;
}) {
  return (
    <header>
      <strong>灯光片段</strong>
      <span>{count} 段</span>
      {selection && (
        <button
          type="button"
          aria-label="时间线片段多选"
          aria-pressed={selection.active}
          disabled={disabled}
          onClick={selection.onMode}
        >
          多选{selection.active ? ` · ${selection.ids.length}` : ""}
        </button>
      )}
      {selection?.active && (
        <div className="audio-clip-tools" role="group" aria-label="片段组工具">
          <button
            type="button"
            aria-pressed={!moving}
            disabled={disabled}
            onClick={() => onTool(false)}
          >
            框选片段
          </button>
          <button
            type="button"
            aria-pressed={moving}
            disabled={disabled || !!selection.movementBlocked}
            title={selection.movementBlocked}
            onClick={() => onTool(true)}
          >
            移动所选
          </button>
        </div>
      )}
      <span>
        {selection?.active
          ? selection.movementBlocked ||
            (moving
              ? "拖动所选 · ← → 微调 · Shift 1 秒 · Esc 取消"
              : "单击增减 · Shift 连选／追加框选 · Esc 取消")
          : "拖动移动 · 两端调整长度 · 空隙为默认值"}
      </span>
    </header>
  );
}
