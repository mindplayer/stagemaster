import type { Camera } from "./usePlanCamera";

/** Keep the world point beneath an anchor fixed when zooming. Anchor is -0.5…0.5. */
export function zoomPlan(
  camera: Camera,
  ratio: number,
  factor: number,
  anchor: [number, number] = [0, 0],
): Camera {
  const width = Math.max(1, Math.min(200000, camera.width * factor));
  return {
    width,
    x: camera.x + anchor[0] * (camera.width - width),
    y: camera.y + (anchor[1] * (camera.width - width)) / ratio,
  };
}

export function panPlan(
  camera: Camera,
  dx: number,
  dy: number,
  pixels: number,
): Camera {
  return {
    ...camera,
    x: camera.x - (dx / pixels) * camera.width,
    y: camera.y + (dy / pixels) * camera.width,
  };
}
