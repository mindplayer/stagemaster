import type { FixturePlacement } from "./stage-types";
export type SpatialOrderAxis = "x" | "y" | "z";
export type SpatialOrderDirection = "ascending" | "descending";

/** World coordinates, never camera-relative. Invalid or absent placements stay last. */
export function fixtureAxisPositions(
  placements: FixturePlacement[],
  axis: SpatialOrderAxis,
): Map<string, number> {
  const result = new Map<string, number>();
  for (const placement of placements) {
    const raw = placement.positionMeters[axis].trim();
    const value = Number(raw);
    if (
      /^-?(0|[1-9]\d*)(\.\d{1,6})?$/.test(raw) &&
      Number.isFinite(value) &&
      Math.abs(value) <= 100000
    )
      result.set(placement.fixtureId, value);
  }
  return result;
}
/** Only reorder existing members. Equal positions retain the author's prior order. */
export function arrangeSpatialFixtureIds(
  ids: string[],
  placements: FixturePlacement[],
  axis: SpatialOrderAxis,
  direction: SpatialOrderDirection,
): string[] {
  const positions = fixtureAxisPositions(placements, axis);
  const sign = direction === "ascending" ? 1 : -1;
  return [...ids].sort((a, b) => {
    const left = positions.get(a),
      right = positions.get(b);
    if (left === undefined || right === undefined)
      return Number(left === undefined) - Number(right === undefined);
    return (left - right) * sign;
  });
}
