import { bounds, decimal, objectOutline, rectangle } from './stage-tools.ts';
import type { StageObject } from './stage-types.ts';
export type Footprint = [string, string][];
export type Extents = { minX: number; minY: number; maxX: number; maxY: number };
export function footprintBounds(points: Footprint): Extents {
  return bounds(points.map(p => [Number(p[0]), Number(p[1])]));
}
export function footprintArea(points: Footprint): number {
  return Math.abs(points.reduce((sum, p, i) => {
    const q = points[(i + 1) % points.length]!;
    return sum + Number(p[0]) * Number(q[1]) - Number(q[0]) * Number(p[1]);
  }, 0)) / 2;
}
export function lShape(x: number, y: number, width: number, depth: number, notchWidth: number, notchDepth: number): Footprint {
  if (!(width > 0 && depth > 0 && notchWidth > 0 && notchWidth < width && notchDepth > 0 && notchDepth < depth))
    throw new Error('缺口尺寸必须大于 0，并小于外部宽度和深度');
  return [[x, y], [x + width, y], [x + width, y + depth - notchDepth],
    [x + width - notchWidth, y + depth - notchDepth], [x + width - notchWidth, y + depth], [x, y + depth]]
    .map(([a, b]) => [decimal(a!), decimal(b!)]);
}
/** Editing geometry only; Rust still owns authoritative polygon validity and history. */
export function resized(object: StageObject, next: Extents): StageObject {
  const copy = structuredClone(object), points = objectOutline(copy);
  if (!points) return copy;
  const old = footprintBounds(points);
  if (![...Object.values(old), ...Object.values(next)].every(Number.isFinite) ||
      old.maxX <= old.minX || old.maxY <= old.minY || next.maxX - next.minX < 0.01 || next.maxY - next.minY < 0.01)
    throw new Error('宽度和深度不能小于 0.01 米');
  for (const p of points) {
    p[0] = decimal(next.minX + (Number(p[0]) - old.minX) / (old.maxX - old.minX) * (next.maxX - next.minX));
    p[1] = decimal(next.minY + (Number(p[1]) - old.minY) / (old.maxY - old.minY) * (next.maxY - next.minY));
  }
  return copy;
}
export function resizedByHandle(object: StageObject, handle: string, dx: number, dy: number): StageObject {
  const points = objectOutline(object);
  if (!points) return object;
  const b = footprintBounds(points), next = { ...b };
  if (handle.includes('w')) next.minX = Math.min(b.maxX - 0.1, b.minX + dx);
  if (handle.includes('e')) next.maxX = Math.max(b.minX + 0.1, b.maxX + dx);
  if (handle.includes('s')) next.minY = Math.min(b.maxY - 0.1, b.minY + dy);
  if (handle.includes('n')) next.maxY = Math.max(b.minY + 0.1, b.maxY + dy);
  return resized(object, next);
}
export { rectangle };
