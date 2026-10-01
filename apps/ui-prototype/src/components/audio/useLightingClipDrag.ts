import {
  useEffect,
  useRef,
  useState,
  type PointerEvent,
  type RefObject,
} from "react";
import type { AudioLightingClip, AudioTimeline } from "../../audio-types";
import type { WaveViewport } from "./waveform-data";
import { moveLightingClip, type ClipMotion } from "./clip-motion";
type Drag = {
  pointer: number;
  clip: AudioLightingClip;
  mode: ClipMotion;
  x: number;
  moved: boolean;
  next: AudioLightingClip;
};
/** Existing single-clip movement/trim; exactly one edit on release. */
export function useLightingClipDrag(
  surface: RefObject<HTMLDivElement | null>,
  track: AudioTimeline,
  viewport: WaveViewport,
  disabled: boolean,
  snap: boolean,
  onSelect: (id: string) => void,
  onMove: (clip: AudioLightingClip) => void,
) {
  const active = useRef<Drag | null>(null);
  const [draft, setDraft] = useState<Drag | null>(null);
  const pixels = viewport.width / Math.max(1, viewport.end - viewport.start);
  function cancel() {
    active.current = null;
    setDraft(null);
  }
  useEffect(cancel, [
    track,
    disabled,
    viewport.start,
    viewport.end,
    viewport.width,
  ]);
  useEffect(() => {
    window.addEventListener("blur", cancel);
    return () => window.removeEventListener("blur", cancel);
  }, []);
  const proposal = (d: Drag, x: number) =>
    moveLightingClip(
      track,
      d.clip,
      d.mode,
      (x - d.x) / pixels,
      snap,
      8 / pixels,
    );
  return {
    draft,
    cancel,
    begin(e: PointerEvent, clip: AudioLightingClip, mode: ClipMotion) {
      e.stopPropagation();
      if (disabled || e.button !== 0) return;
      e.preventDefault();
      (e.currentTarget as HTMLElement).focus();
      onSelect(clip.id);
      if (clip.locked) return;
      surface.current?.setPointerCapture(e.pointerId);
      active.current = {
        pointer: e.pointerId,
        clip,
        mode,
        x: e.clientX,
        moved: false,
        next: clip,
      };
      setDraft(active.current);
    },
    move(e: PointerEvent) {
      const d = active.current;
      if (!d || d.pointer !== e.pointerId) return;
      const moved = d.moved || Math.abs(e.clientX - d.x) >= 3;
      active.current = {
        ...d,
        moved,
        next: moved ? proposal(d, e.clientX) : d.clip,
      };
      setDraft(active.current);
    },
    end(e: PointerEvent) {
      const d = active.current;
      if (!d || d.pointer !== e.pointerId) return;
      const next =
        d.moved || Math.abs(e.clientX - d.x) >= 3
          ? proposal(d, e.clientX)
          : d.clip;
      cancel();
      if (surface.current?.hasPointerCapture(e.pointerId))
        surface.current.releasePointerCapture(e.pointerId);
      if (next.startMs !== d.clip.startMs || next.endMs !== d.clip.endMs)
        onMove(next);
    },
  };
}
