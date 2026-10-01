import type { ProjectView } from "../../application-host";
import type { StageObject, StageView } from "../../stage-types";
import { fixtureSymbol, fixtureAddress } from "../../fixture-plan-display";
import { FixturePlanSymbol } from "./FixturePlanSymbol";
import { displayMeters } from "./stage-display";
export function StageFixtureObjects({
  project,
  stage,
  drawn,
  selectedIds,
  selected,
  unit,
}: {
  project: ProjectView;
  stage: StageView;
  drawn(object: StageObject): StageObject;
  selectedIds: string[];
  selected(id: string): boolean;
  unit: number;
}) {
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
      },
    ];
  });
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
    </>
  );
}
