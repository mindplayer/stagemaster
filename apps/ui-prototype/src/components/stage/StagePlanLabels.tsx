import { useEffect } from "react";
import type { ProjectView } from "../../application-host";
import type { StageObject, StageSelection, StageView } from "../../stage-types";
import type { PlanLabelMode } from "../../fixture-plan-display";
import { layoutPlanLabels, type LabelBox } from "../../plan-label-layout";
import { stageLabelItems } from "../../stage-label-items";
import { usePlanLabelMetrics } from "./usePlanLabelMetrics";
import "./fixture-label-layout.css";

export function StagePlanLabels({
  project,
  stage,
  drawn,
  isSelected,
  selectedIds,
  unit,
  labels,
  viewport,
  onHiddenLabels,
}: {
  project: ProjectView;
  stage: StageView;
  drawn(object: StageObject): StageObject;
  isSelected(kind: StageSelection["kind"], id: string): boolean;
  selectedIds: string[];
  unit: number;
  labels: PlanLabelMode;
  viewport?: LabelBox;
  onHiddenLabels?(count: number): void;
}) {
  const items = stageLabelItems(
    project.fixtures,
    stage,
    labels,
    unit,
    drawn,
    isSelected,
    selectedIds,
  );
  const metrics = usePlanLabelMetrics(items);
  const normalized = viewport
    ? {
        x: viewport.x / unit,
        y: viewport.y / unit,
        width: viewport.width / unit,
        height: viewport.height / unit,
      }
    : undefined;
  const layout = layoutPlanLabels(
    items.map((i) => ({ ...i, width: metrics.get(i.id)!.width })),
    normalized,
  );
  useEffect(() => {
    onHiddenLabels?.(layout.hidden.length);
  }, [layout.hidden.length, onHiddenLabels]);
  const byId = new Map(items.map((i) => [i.id, i]));
  return (
    <g
      className="fixture-label-layer"
      pointerEvents="none"
      aria-label="场地标注"
    >
      {layout.placed.map((b) => {
        const i = byId.get(b.id)!;
        const cx = (b.x + b.width / 2) * unit,
          cy = (b.y + b.height / 2) * unit;
        const x = i.x * unit,
          y = i.y * unit;
        const dx = cx - x,
          dy = cy - y,
          len = Math.hypot(dx, dy);
        const gap = i.symbol && len > 0 ? Math.min(1, (unit * 1.3) / len) : 0;
        return (
          <g
            key={b.id}
            data-plan-label={b.id}
            data-fixture-label={i.symbol ? i.objectId : undefined}
            className={i.selected ? "selected" : ""}
          >
            <line
              x1={x + dx * gap}
              y1={y + dy * gap}
              x2={cx}
              y2={cy}
              strokeWidth={unit * 0.07}
            />
            <rect
              x={b.x * unit}
              y={b.y * unit}
              width={b.width * unit}
              height={b.height * unit}
              rx={unit * 0.2}
            />
            <text
              x={cx}
              y={cy + unit * 0.3}
              fontSize={unit * 0.9}
              fontFamily="sans-serif"
            >
              {metrics.get(b.id)!.text}
            </text>
          </g>
        );
      })}
    </g>
  );
}
