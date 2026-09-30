import type { FixturePlacement, StageView } from "../../stage-types.ts";
import { selectInBox, togglePlacement } from "../../placement-tools.ts";
export type Point = [number, number];
export function nearbyFixtures(
  placements: FixturePlacement[],
  point: Point,
  radius: number,
) {
  return placements
    .filter(
      (p) =>
        Math.hypot(
          Number(p.positionMeters.x) - point[0],
          Number(p.positionMeters.y) - point[1],
        ) <= radius,
    )
    .map((p) => p.fixtureId);
}
export function selectionAfterClick(
  ids: string[],
  id: string,
  additive: boolean,
) {
  return togglePlacement(ids, id, additive);
}
export function selectionAfterBox(
  stage: StageView,
  selected: string[],
  from: Point,
  to: Point,
  additive: boolean,
) {
  const hit = selectInBox(stage.placements, from, to);
  return additive ? [...new Set([...selected, ...hit])] : hit;
}
