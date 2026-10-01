import type { FixtureView } from "./application-host";
import type { StageObject, StageSelection, StageView } from "./stage-types";
import {
  fixturePlanLabel,
  type PlanLabelMode,
} from "./fixture-plan-display.ts";
import { selectionPoints } from "./components/stage/plan-focus.ts";

export interface StageLabelItem {
  id: string;
  objectId: string;
  kind: StageSelection["kind"];
  x: number;
  y: number;
  text: string;
  selected: boolean;
  priority: number;
  symbol: boolean;
}
/** Read-only label anchors use the same transient geometry as the rendered objects. */
export function stageLabelItems(
  fixtures: FixtureView[],
  stage: StageView,
  mode: PlanLabelMode,
  unit: number,
  drawn: (object: StageObject) => StageObject,
  selected: (kind: StageSelection["kind"], id: string) => boolean,
  selectedIds: string[],
): StageLabelItem[] {
  if (mode === "none" || !Number.isFinite(unit) || unit <= 0) return [];
  const fixtureMap = new Map(fixtures.map((f) => [f.id, f]));
  const result: StageLabelItem[] = stage.placements.flatMap((p) => {
    const moved = drawn({ kind: "placement", value: p });
    if (moved.kind !== "placement") return [];
    const f = fixtureMap.get(p.fixtureId);
    const active = selected("placement", p.fixtureId);
    return [
      {
        id: `placement:${p.fixtureId}`,
        objectId: p.fixtureId,
        kind: "placement",
        x: Number(moved.value.positionMeters.x) / unit,
        y: -Number(moved.value.positionMeters.y) / unit,
        text: f ? fixturePlanLabel(f, mode) : "未知灯具",
        selected: active,
        priority: active ? selectedIds.indexOf(p.fixtureId) + 1 : 100000,
        symbol: true,
      },
    ];
  });
  if (mode !== "name") return result;
  const objects: Exclude<StageObject, { kind: "placement" }>[] = [
    ...stage.spaces.map(
      (value): Exclude<StageObject, { kind: "placement" }> => ({
        kind: "space",
        value,
      }),
    ),
    ...stage.constructions
      .filter((c) => c.shape.kind !== "enclosure")
      .map((value): Exclude<StageObject, { kind: "placement" }> => ({
        kind: "construction",
        value,
      })),
  ];
  for (const object of objects) {
    const moved = drawn(object);
    let points = selectionPoints(stage, moved, []);
    // Invalid seating drafts retain the saved chairs in SeatingPlanObject.
    if (
      !points?.length &&
      object.kind === "construction" &&
      object.value.shape.kind === "seating"
    )
      points = selectionPoints(stage, object, []);
    if (!points?.length) continue;
    const xs = points.map((p) => Number(p[0])),
      ys = points.map((p) => -Number(p[1]));
    const minX = Math.min(...xs),
      maxX = Math.max(...xs),
      minY = Math.min(...ys),
      maxY = Math.max(...ys);
    const active = selected(object.kind, object.value.id);
    const seating =
      object.kind === "construction" && object.value.shape.kind === "seating"
        ? object.value.shape
        : null;
    result.push({
      id: `${object.kind}:${object.value.id}`,
      objectId: object.value.id,
      kind: object.kind,
      x: (minX + maxX) / (2 * unit),
      y:
        (object.kind === "space" ? minY : seating ? maxY : (minY + maxY) / 2) /
        unit,
      text: `${object.value.name}${seating ? ` · ${seating.rows * seating.columns} 座` : ""}`,
      selected: active,
      priority: active ? 0 : object.kind === "space" ? 300000 : 200000,
      symbol: false,
    });
  }
  return result;
}
