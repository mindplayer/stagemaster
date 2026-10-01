import { positionDecimal } from "../../position-tools.ts";
import type { Camera } from "../stage/usePlanCamera";
export type AimPoint = { x: number; y: number };
export type PlaneRect = {
  left: number;
  top: number;
  width: number;
  height: number;
};
const LIMIT = 100000;
export function readAimPoint(x: string, y: string): AimPoint | null {
  try {
    return {
      x: Number(positionDecimal(x, "x", "目标 X", LIMIT)),
      y: Number(positionDecimal(y, "y", "目标 Y", LIMIT)),
    };
  } catch {
    return null;
  }
}
function rounded(value: number) {
  return Math.round(value * 1000) / 1000;
}
/** SVG meet projection, including letterboxing. Pointer capture stays at the visible edge. */
export function aimPointAt(
  camera: Camera,
  ratio: number,
  rect: PlaneRect,
  clientX: number,
  clientY: number,
): AimPoint | null {
  if (
    ![
      camera.x,
      camera.y,
      camera.width,
      ratio,
      rect.left,
      rect.top,
      rect.width,
      rect.height,
      clientX,
      clientY,
    ].every(Number.isFinite) ||
    camera.width <= 0 ||
    ratio <= 0 ||
    rect.width <= 0 ||
    rect.height <= 0
  )
    return null;
  const height = camera.width / ratio;
  const scale = Math.min(rect.width / camera.width, rect.height / height);
  const dx = Math.max(
    -camera.width / 2,
    Math.min(camera.width / 2, (clientX - rect.left - rect.width / 2) / scale),
  );
  const dy = Math.max(
    -height / 2,
    Math.min(height / 2, (rect.top + rect.height / 2 - clientY) / scale),
  );
  const point = { x: rounded(camera.x + dx), y: rounded(camera.y + dy) };
  return Math.abs(point.x) <= LIMIT && Math.abs(point.y) <= LIMIT
    ? point
    : null;
}
export function nudgeAimPoint(
  point: AimPoint,
  key: string,
  fine: boolean,
): AimPoint | null {
  const directions: Record<string, [number, number]> = {
    ArrowLeft: [-1, 0],
    ArrowRight: [1, 0],
    ArrowUp: [0, 1],
    ArrowDown: [0, -1],
  };
  const direction = directions[key];
  if (!direction || ![point.x, point.y].every(Number.isFinite)) return null;
  const step = fine ? 0.01 : 0.1;
  return {
    x: rounded(
      Math.max(-LIMIT, Math.min(LIMIT, point.x + direction[0] * step)),
    ),
    y: rounded(
      Math.max(-LIMIT, Math.min(LIMIT, point.y + direction[1] * step)),
    ),
  };
}
export function pointFields(point: AimPoint) {
  return { x: String(point.x), y: String(point.y) };
}
