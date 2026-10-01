import { useTimelineEdgeScroll } from "./useTimelineEdgeScroll";
import { timelinePoint } from "./timeline-edge-scroll";
import {
  useEffect,
  useRef,
  useState,
  type PointerEvent,
  type RefObject,
} from "react";
import type { WaveViewport } from "./waveform-data";
import type { ClipLaneSelection } from "./clip-selection";
type Marquee = {
  pointer: number;
  start: number;
  end: number;
  x: number;
  clientX: number;
  clientY: number;
  moved: boolean;
  append: boolean;
  id?: string;
};
/** A mode-specific selection gesture. Nothing is committed until pointer release. */
export function useClipMarquee(
  surface: RefObject<HTMLDivElement | null>,
  viewport: WaveViewport,
  selection: ClipLaneSelection | undefined,
  disabled: boolean,
  revision: unknown,
  duration: number,
  onPan?: (pixels: number) => void,
) {
  const active = useRef<Marquee | null>(null);
  const [draft, setDraft] = useState<Marquee | null>(null);
  function cancel() {
    active.current = null;
    setDraft(null);
  }
  useEffect(cancel, [disabled, selection?.active, revision]);
  const point = (x: number) =>
    timelinePoint(
      x,
      surface.current?.getBoundingClientRect().left ?? 0,
      viewport,
    );
  function update(clientX: number, clientY: number) {
    const d = active.current;
    if (!d) return;
    active.current = {
      ...d,
      clientX,
      clientY,
      end: point(clientX),
      moved: d.moved || Math.abs(clientX - d.x) >= 3,
    };
    setDraft(active.current);
  }
  useTimelineEdgeScroll(
    surface,
    viewport,
    duration,
    draft,
    onPan,
    update,
    cancel,
  );
  return {
    draft,
    cancel,
    begin(e: PointerEvent, id?: string) {
      if (!selection?.active || disabled || e.button !== 0) return;
      e.preventDefault();
      e.stopPropagation();
      if (e.target instanceof HTMLElement)
        e.target.closest<HTMLButtonElement>("button")?.focus();
      surface.current?.setPointerCapture(e.pointerId);
      active.current = {
        pointer: e.pointerId,
        start: point(e.clientX),
        end: point(e.clientX),
        x: e.clientX,
        clientX: e.clientX,
        clientY: e.clientY,
        moved: false,
        append: e.shiftKey,
        id,
      };
      setDraft(active.current);
    },
    move(e: PointerEvent) {
      const d = active.current;
      if (!d || d.pointer !== e.pointerId) return;
      update(e.clientX, e.clientY);
    },
    end(e: PointerEvent) {
      const d = active.current;
      if (!d || d.pointer !== e.pointerId) return;
      const end = point(e.clientX);
      cancel();
      if (surface.current?.hasPointerCapture(e.pointerId))
        surface.current.releasePointerCapture(e.pointerId);
      if (d.moved || Math.abs(e.clientX - d.x) >= 3)
        selection?.onRange(d.start, end, d.append);
      else if (d.id) selection?.onPick(d.id, d.append);
      else if (!d.append) selection?.onClear();
    },
  };
}
