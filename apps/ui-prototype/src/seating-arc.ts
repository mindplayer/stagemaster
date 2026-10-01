import type { SeatingShape } from "./stage-types.ts";
type Point = [number, number];
/** Draft projection only; Rust validates the same geometry before any write. */
export function curvedSeating(
  s: SeatingShape,
  dimensions: [number, number, number, number],
  radius: number,
) {
  const [w, d, dx, dy] = dimensions;
  const step = dx / radius;
  let gap = 0;
  if (s.aisle) {
    const clear = Number(s.aisle.widthMeters),
      radial = 2 * radius - d;
    const diagonal = Math.hypot(radial, w);
    if (clear >= diagonal) return null;
    gap = Math.max(
      0,
      2 * (Math.atan2(w, radial) + Math.asin(clear / diagonal)) - step,
    );
  }
  const spread = (s.columns - 1) * step + gap;
  if (spread + 2 * Math.atan(w / (2 * radius - d)) > Math.PI + 1e-9)
    return null;
  const centers: Point[] = [],
    angles: number[] = [];
  const min = [Infinity, Infinity],
    max = [-Infinity, -Infinity];
  for (let row = 0; row < s.rows; row++) {
    const r = radius + row * dy;
    for (let col = 0; col < s.columns; col++) {
      const angle =
        -spread / 2 +
        col * step +
        (s.aisle && col >= s.aisle.afterColumn ? gap : 0);
      const sin = Math.sin(angle),
        cos = Math.cos(angle);
      const center: Point = [r * sin, radius - r * cos];
      for (const x of [-w / 2, w / 2])
        for (const y of [-d / 2, d / 2]) {
          const corner = [
            center[0] + x * cos - y * sin,
            center[1] + x * sin + y * cos,
          ];
          for (let axis = 0; axis < 2; axis++) {
            min[axis] = Math.min(min[axis], corner[axis]);
            max[axis] = Math.max(max[axis], corner[axis]);
          }
        }
      centers.push(center);
      angles.push(angle);
    }
  }
  for (let i = 0; i < centers.length; i++)
    for (let j = 0; j < i; j++) {
      const x = centers[i][0] - centers[j][0],
        y = centers[i][1] - centers[j][1];
      if (x * x + y * y >= w * w + d * d) continue;
      const separated = [angles[i], angles[j]].some((a) =>
        [a, a + Math.PI / 2].some((axis) => {
          const distance = Math.abs(x * Math.cos(axis) + y * Math.sin(axis));
          const extent = (a: number) =>
            (w * Math.abs(Math.cos(a - axis)) +
              d * Math.abs(Math.sin(a - axis))) /
            2;
          return distance >= extent(angles[i]) + extent(angles[j]) - 1e-9;
        }),
      );
      if (!separated) return null;
    }
  const middle = [(min[0] + max[0]) / 2, (min[1] + max[1]) / 2];
  return {
    centers: centers.map(([x, y]): Point => [x - middle[0], y - middle[1]]),
    angles: angles.map((a) => (a * 180) / Math.PI),
    focus: [-middle[0], radius - middle[1]] as Point,
    width: max[0] - min[0],
    depth: max[1] - min[1],
  };
}
