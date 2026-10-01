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
) {
  const active = useRef<Marquee | null>(null);
  const [draft, setDraft] = useState<Marquee | null>(null);
  function cancel() {
    active.current = null;
    setDraft(null);
  }
  useEffect(cancel, [
    disabled,
    selection?.active,
    viewport.start,
    viewport.end,
    viewport.width,
    revision,
  ]);
  useEffect(() => {
    window.addEventListener("blur", cancel);
    return () => window.removeEventListener("blur", cancel);
  }, []);
  function point(x: number) {
    const left = surface.current?.getBoundingClientRect().left ?? 0;
    return (
      viewport.start +
      (Math.max(0, Math.min(viewport.width, x - left)) /
        Math.max(1, viewport.width)) *
        (viewport.end - viewport.start)
    );
  }
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
        moved: false,
        append: e.shiftKey,
        id,
      };
      setDraft(active.current);
    },
    move(e: PointerEvent) {
      const d = active.current;
      if (!d || d.pointer !== e.pointerId) return;
      active.current = {
        ...d,
        end: point(e.clientX),
        moved: d.moved || Math.abs(e.clientX - d.x) >= 3,
      };
      setDraft(active.current);
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
