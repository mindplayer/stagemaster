import type { WaveViewport } from "./waveform-data";
export interface TimelinePointer {
  clientX: number;
  clientY: number;
  moved: boolean;
}
export interface TimelineBounds {
  left: number;
  right: number;
  top: number;
  bottom: number;
}

/** View navigation only: units are CSS pixels, never playback time. */
export function edgeScrollPixels(
  pointer: TimelinePointer,
  bounds: TimelineBounds,
  view: WaveViewport,
  duration: number,
  elapsed: number,
): number {
  if (
    !pointer.moved ||
    view.width <= 0 ||
    elapsed <= 0 ||
    pointer.clientY < bounds.top - 24 ||
    pointer.clientY > bounds.bottom + 24
  )
    return 0;
  const zone = Math.min(48, view.width / 5);
  const left = Math.max(
    0,
    Math.min(1, (bounds.left + zone - pointer.clientX) / zone),
  );
  const right = Math.max(
    0,
    Math.min(1, (pointer.clientX - bounds.right + zone) / zone),
  );
  const velocity =
    (right * right - left * left) * Math.min(600, view.width * 0.85);
  const span = Math.max(1, view.end - view.start);
  const wanted = (velocity * Math.min(50, elapsed)) / 1000;
  return (
    Math.max(
      (-Math.max(0, view.start) * view.width) / span,
      Math.min((Math.max(0, duration - view.end) * view.width) / span, wanted),
    ) || 0
  );
}

/** The grip stays anchored in content time while the visible window moves. */
export function timelinePoint(
  x: number,
  left: number,
  view: WaveViewport,
): number {
  return (
    view.start +
    (Math.max(0, Math.min(view.width, x - left)) / Math.max(1, view.width)) *
      (view.end - view.start)
  );
}
export function sameTimelineScale(a: WaveViewport, b: WaveViewport): boolean {
  const span = a.end - a.start;
  return (
    a.width === b.width &&
    Math.abs(span - (b.end - b.start)) <= Math.max(0.001, span * 1e-6)
  );
}
