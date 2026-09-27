import type { RigShape, FixturePlacement } from "./stage-types";
import { decimal } from "./stage-tools.ts";
export function rigOutline(rig: RigShape): [number, number][] {
  const x = Number(rig.positionMeters.x),
    y = Number(rig.positionMeters.y),
    l = Number(rig.lengthMeters) / 2,
    w = Number(rig.widthMeters) / 2,
    a = (Number(rig.yawDegrees) * Math.PI) / 180;
  if (![x, y, l, w, a].every(Number.isFinite)) return [];
  return [
    [-l, -w],
    [l, -w],
    [l, w],
    [-l, w],
  ].map(([u, v]) => [
    x + u! * Math.cos(a) - v! * Math.sin(a),
    y + u! * Math.sin(a) + v! * Math.cos(a),
  ]);
}
// Visual draft only. Committed attachment transforms are evaluated in Rust.
export function previewRigPlacement(
  p: FixturePlacement,
  old: RigShape,
  next: RigShape,
): FixturePlacement {
  const x = Number(p.positionMeters.x) - Number(old.positionMeters.x),
    y = Number(p.positionMeters.y) - Number(old.positionMeters.y),
    a = ((Number(next.yawDegrees) - Number(old.yawDegrees)) * Math.PI) / 180;
  const nx = Number(next.positionMeters.x) + x * Math.cos(a) - y * Math.sin(a),
    ny = Number(next.positionMeters.y) + x * Math.sin(a) + y * Math.cos(a),
    nz =
      Number(p.positionMeters.z) +
      Number(next.positionMeters.z) -
      Number(old.positionMeters.z);
  return [nx, ny, nz].every(Number.isFinite)
    ? {
        ...p,
        positionMeters: { x: decimal(nx), y: decimal(ny), z: decimal(nz) },
      }
    : p;
}
export function planeDistance(a: [number, number], b: [number, number]) {
  return {
    distance: Math.hypot(b[0] - a[0], b[1] - a[1]),
    dx: b[0] - a[0],
    dy: b[1] - a[1],
  };
}
