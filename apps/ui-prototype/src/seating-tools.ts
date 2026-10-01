import type { SeatingShape } from "./stage-types.ts";
type Point = [number, number];
/** Bounded draft projection. Rust validates committed geometry and generates the UE mesh. */
export function seatingLayout(s: SeatingShape) {
  if (
    !Number.isInteger(s.rows) ||
    !Number.isInteger(s.columns) ||
    s.rows < 1 ||
    s.columns < 1 ||
    s.rows > 64 ||
    s.columns > 64 ||
    s.rows * s.columns > 512
  )
    return null;
  const number = (v: string, min: number, max: number) => {
    const n = Number(v);
    return v.trim() && Number.isFinite(n) && n >= min && n <= max ? n : NaN;
  };
  const w = number(s.seatWidthMeters, 0.3, 1.2),
    d = number(s.seatDepthMeters, 0.3, 1.2);
  const dx = number(s.columnSpacingMeters, w, 5),
    dy = number(s.rowSpacingMeters, d, 10);
  const x = number(s.positionMeters.x, -100000, 100000),
    y = number(s.positionMeters.y, -100000, 100000),
    z = number(s.positionMeters.z, -100000, 100000);
  const yaw = number(s.yawDegrees, -3600, 3600),
    angle = (yaw * Math.PI) / 180;
  if (![w, d, dx, dy, x, y, z, yaw].every(Number.isFinite)) return null;
  let extra = 0;
  if (s.aisle) {
    if (
      !Number.isInteger(s.aisle.afterColumn) ||
      s.aisle.afterColumn < 1 ||
      s.aisle.afterColumn >= s.columns
    )
      return null;
    const width = number(s.aisle.widthMeters, 0.3, 10);
    if (!Number.isFinite(width) || width + 1e-9 < dx - w) return null;
    extra = Math.max(0, width - (dx - w));
  }
  const width = (s.columns - 1) * dx + w + extra,
    depth = (s.rows - 1) * dy + d;
  const world = ([a, b]: Point): Point => [
    x + a * Math.cos(angle) - b * Math.sin(angle),
    y + a * Math.sin(angle) + b * Math.cos(angle),
  ];
  const outline: Point[] = [
    [-width / 2, -depth / 2],
    [width / 2, -depth / 2],
    [width / 2, depth / 2],
    [-width / 2, depth / 2],
  ];
  const worldOutline = outline.map(world);
  if (
    worldOutline.flat().some((v) => Math.abs(v) > 100000) ||
    z + 0.85 > 100000
  )
    return null;
  const centers: Point[] = [];
  for (let row = 0; row < s.rows; row++)
    for (let col = 0; col < s.columns; col++) {
      centers.push([
        -width / 2 +
          w / 2 +
          col * dx +
          (s.aisle && col >= s.aisle.afterColumn ? extra : 0),
        depth / 2 - d / 2 - row * dy,
      ]);
    }
  return {
    width,
    depth,
    w,
    d,
    yaw,
    centers,
    worldCenters: centers.map(world),
    outline: worldOutline,
  };
}
export const seatingOutline = (s: SeatingShape) =>
  seatingLayout(s)?.outline ?? [];
