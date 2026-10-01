import { useEffect, useRef, type RefObject } from "react";
import type { WaveViewport } from "./waveform-data";
import {
  edgeScrollPixels,
  sameTimelineScale,
  type TimelinePointer,
} from "./timeline-edge-scroll";

/** One bounded animation loop for the active gesture; no synthetic pointer events. */
export function useTimelineEdgeScroll(
  surface: RefObject<HTMLDivElement | null>,
  viewport: WaveViewport,
  duration: number,
  pointer: TimelinePointer | null,
  pan: ((pixels: number) => void) | undefined,
  update: (x: number, y: number) => void,
  cancel: () => void,
) {
  const latest = useRef({ viewport, duration, pointer, pan, update, cancel });
  latest.current = { viewport, duration, pointer, pan, update, cancel };
  const previous = useRef(viewport);
  useEffect(() => {
    const old = previous.current;
    previous.current = viewport;
    const current = latest.current;
    if (!current.pointer) return;
    if (!sameTimelineScale(old, viewport)) current.cancel();
    else if (current.pointer.moved)
      current.update(current.pointer.clientX, current.pointer.clientY);
  }, [viewport.start, viewport.end, viewport.width]);
  const active = pointer !== null;
  useEffect(() => {
    if (!active) return;
    let frame = 0;
    let previousTime: number | null = null;
    function tick(now: number) {
      const current = latest.current;
      const el = surface.current;
      if (!current.pointer) return;
      if (!el?.isConnected || document.hidden || !el.getClientRects().length) {
        current.cancel();
        return;
      }
      const elapsed = previousTime === null ? 0 : now - previousTime;
      previousTime = now;
      // Even a held grip suppresses playback-follow, without moving the viewport.
      current.pan?.(
        edgeScrollPixels(
          current.pointer,
          el.getBoundingClientRect(),
          current.viewport,
          current.duration,
          elapsed,
        ),
      );
      frame = requestAnimationFrame(tick);
    }
    const abort = () => latest.current.cancel();
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        abort();
      }
    };
    const hidden = () => {
      if (document.hidden) abort();
    };
    window.addEventListener("blur", abort);
    window.addEventListener("keydown", escape);
    document.addEventListener("visibilitychange", hidden);
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener("blur", abort);
      window.removeEventListener("keydown", escape);
      document.removeEventListener("visibilitychange", hidden);
    };
  }, [active, surface]);
}
