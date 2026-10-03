import { FixtureLabelControl } from "./FixtureLabelControl";
import type { PlanLabelMode } from "../../fixture-plan-display";
import {
  ArrowsOutIcon,
  MagnifyingGlassMinusIcon,
  MagnifyingGlassPlusIcon,
} from "@phosphor-icons/react";
type Tool = "select" | "move" | "pan" | "measure";
export function StagePlanToolbar({
  tool,
  onTool,
  snap,
  onSnap,
  labels,
  onLabels,
  disabled,
  visibleCount,
  selectedCount,
  onSelectAll,
  onArrange,
  measured,
  onClearMeasure,
  canFocus,
  onFocus,
  onFit,
  onZoom,
}: {
  tool: Tool;
  onTool(tool: Tool): void;
  snap: boolean;
  onSnap(value: boolean): void;
  labels: PlanLabelMode;
  onLabels(value: PlanLabelMode): void;
  disabled: boolean;
  visibleCount: number;
  selectedCount: number;
  onSelectAll(): void;
  onArrange(): void;
  measured: boolean;
  onClearMeasure(): void;
  canFocus: boolean;
  onFocus(): void;
  onFit(): void;
  onZoom(factor: number): void;
}) {
  return (
    <div className="stage-canvas-toolbar">
      <div className="stage-tool-switch" aria-label="布置工具">
        {(
          [
            ["select", "选择"],
            ["move", "移动"],
            ["pan", "平移视图"],
            ["measure", "测距"],
          ] as const
        ).map(([key, label]) => (
          <button
            key={key}
            aria-pressed={tool === key}
            disabled={disabled && (key === "select" || key === "move")}
            onClick={() => onTool(key)}
          >
            {label}
          </button>
        ))}
      </div>
      <label className="stage-check">
        <input
          type="checkbox"
          checked={snap}
          onChange={(e) => onSnap(e.target.checked)}
        />
        吸附 0.1 米
      </label>
      <FixtureLabelControl value={labels} onChange={onLabels} />
      <button disabled={disabled || !visibleCount} onClick={onSelectAll}>
        全选可见灯位
      </button>
      <button disabled={disabled || !selectedCount} onClick={onArrange}>
        排列所选 · {selectedCount}
      </button>
      {measured && <button onClick={onClearMeasure}>清除测距</button>}
      <span />
      <button aria-label="缩小场地" onClick={() => onZoom(1.25)}>
        <MagnifyingGlassMinusIcon />
      </button>
      <button aria-label="放大场地" onClick={() => onZoom(0.8)}>
        <MagnifyingGlassPlusIcon />
      </button>
      <button
        aria-label="聚焦所选"
        title="聚焦所选（F）"
        disabled={!canFocus}
        onClick={onFocus}
      >
        聚焦所选
      </button>
      <button aria-label="查看全场" title="查看可见对象" onClick={onFit}>
        <ArrowsOutIcon />
      </button>
    </div>
  );
}
