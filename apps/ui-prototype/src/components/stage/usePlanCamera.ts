import { useEffect, useRef, useState, type RefObject } from "react";
import { bounds } from "../../stage-tools";
export type Camera = { x: number; y: number; width: number };
export function fittedCamera(
  points: [number, number][],
  ratio: number,
): Camera {
  if (!points.length) return { x: 4, y: 3, width: 20 };
  const b = bounds(points);
  return {
    x: (b.minX + b.maxX) / 2,
    y: (b.minY + b.maxY) / 2,
    width: Math.max(
      8,
      (b.maxX - b.minX) * 1.35,
      (b.maxY - b.minY) * ratio * 1.35,
    ),
  };
}

/** Fit once when a parked canvas first gets a real viewport; later resizing keeps navigation. */
export function usePlanCamera(
  svg: RefObject<SVGSVGElement | null>,
  points: [number, number][],
  projectId: string,
) {
  const [camera, setCamera] = useState<Camera>({ x: 4, y: 3, width: 20 });
  const [ratio, setRatio] = useState(1.4);
  const latest = useRef(points);
  latest.current = points;
  useEffect(() => {
    const el = svg.current;
    if (!el) return;
    let initialized = false;
    const observer = new ResizeObserver(([entry]) => {
      const rect = entry?.contentRect;
      if (!rect || rect.width <= 0 || rect.height <= 0) return;
      const nextRatio = rect.width / rect.height;
      setRatio(nextRatio);
      if (!initialized) {
        initialized = true;
        setCamera(fittedCamera(latest.current, nextRatio));
      }
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, [svg, projectId]);
  return { camera, setCamera, ratio };
}
