import { useEffect } from "react";
import type { ProjectView } from "../../application-host";
import type { StageObject, StageView } from "../../stage-types";
import {
  fixtureSymbol,
  fixtureAddress,
  type PlanLabelMode,
} from "../../fixture-plan-display";
import { layoutPlanLabels, type LabelBox } from "../../plan-label-layout";
import { FixturePlanSymbol } from "./FixturePlanSymbol";
import { useFixtureLabelMetrics } from "./useFixtureLabelMetrics";
import { displayMeters } from "./stage-display";
import "./fixture-label-layout.css";
export function StageFixtureObjects({
  project,
  stage,
  drawn,
  selectedIds,
  selected,
  unit,
  labels,
  viewport,
  onHiddenLabels,
}: {
  project: ProjectView;
  stage: StageView;
  drawn(object: StageObject): StageObject;
  selectedIds: string[];
  selected(id: string): boolean;
  unit: number;
  labels: PlanLabelMode;
  viewport?: LabelBox;
  onHiddenLabels?(count: number): void;
}) {
  const metrics = useFixtureLabelMetrics(project.fixtures, labels);
  const fixtures = new Map(project.fixtures.map((f) => [f.id, f]));
  const items = stage.placements.flatMap((p) => {
    const moved = drawn({ kind: "placement", value: p });
    if (moved.kind !== "placement") return [];
    const f = fixtures.get(p.fixtureId),
      active = selected(p.fixtureId),
      order = selectedIds.indexOf(p.fixtureId) + 1;
    return [
      {
        id: p.fixtureId,
        x: Number(moved.value.positionMeters.x),
        y: -Number(moved.value.positionMeters.y),
        z: moved.value.positionMeters.z,
        fixture: f,
        symbol: fixtureSymbol(f),
        active,
        order,
        label: metrics.get(p.fixtureId),
      },
    ];
  });
  const normalized = viewport
    ? {
        x: viewport.x / unit,
        y: viewport.y / unit,
        width: viewport.width / unit,
        height: viewport.height / unit,
      }
    : undefined;
  const layout = layoutPlanLabels(
    items
      .filter((i) => i.label)
      .map((i) => ({
        id: i.id,
        x: i.x / unit,
        y: i.y / unit,
        width: i.label!.width,
        selected: i.active,
        priority: i.active ? i.order : 100000,
      })),
    normalized,
  );
  useEffect(() => {
    onHiddenLabels?.(layout.hidden.length);
  }, [layout.hidden.length, onHiddenLabels]);
  const byId = new Map(items.map((i) => [i.id, i]));
  return (
    <>
      {items.map((i) => (
        <g
          key={i.id}
          data-kind="placement"
          data-id={i.id}
          transform={`translate(${i.x},${i.y})`}
          className={`stage-light ${i.active ? "is-selected" : ""}`}
        >
          <FixturePlanSymbol
            symbol={i.symbol}
            unit={unit}
            selected={i.active}
          />
          {i.active && (
            <text
              className="stage-selection-number"
              y={-unit * 1.55}
              fontSize={unit * 0.9}
            >
              {i.order}
            </text>
          )}
          <title>{`${i.fixture?.name ?? "未知灯具"} · ${i.symbol.label} · ${fixtureAddress(i.fixture)} · 高度 ${displayMeters(i.z)} 米`}</title>
        </g>
      ))}
      <g
        className="fixture-label-layer"
        pointerEvents="none"
        aria-label="灯位标注"
      >
        {layout.placed.map((b) => {
          const i = byId.get(b.id)!;
          const cx = (b.x + b.width / 2) * unit,
            cy = (b.y + b.height / 2) * unit;
          const dx = cx - i.x,
            dy = cy - i.y,
            len = Math.hypot(dx, dy),
            gap = (unit * 1.3) / len;
          return (
            <g
              key={b.id}
              data-fixture-label={b.id}
              className={i.active ? "selected" : ""}
            >
              <line
                x1={i.x + dx * gap}
                y1={i.y + dy * gap}
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
                {i.label!.text}
              </text>
            </g>
          );
        })}
      </g>
    </>
  );
}
