import { useTimelineEdgeScroll } from "./useTimelineEdgeScroll";
import { timelinePoint } from "./timeline-edge-scroll";
import {
  useEffect,
  useRef,
  useState,
  type PointerEvent,
  type RefObject,
} from "react";
import type { AudioTimeline } from "../../audio-types";
import type { WaveViewport } from "./waveform-data";
import type { ClipLaneSelection } from "./clip-selection";
import { moveClipGroup, type ClipGroupMotion } from "./clip-group-motion";
type Gesture = {
  pointer: number;
  x: number;
  clientX: number;
  clientY: number;
  anchor: number;
  moved: boolean;
  id: string;
  ids: string[];
  next: ClipGroupMotion;
};

export function useClipGroupDrag(
  surface: RefObject<HTMLDivElement | null>,
  track: AudioTimeline,
  viewport: WaveViewport,
  selection: ClipLaneSelection | undefined,
  disabled: boolean,
  snap: boolean,
  onPan?: (pixels: number) => void,
) {
  const active = useRef<Gesture | null>(null);
  const [draft, setDraft] = useState<Gesture | null>(null);
  const [problem, setProblem] = useState("");
  const pixels = viewport.width / Math.max(1, viewport.end - viewport.start);
  const signature = selection?.ids.join("|");
  function cancel() {
    active.current = null;
    setDraft(null);
    setProblem("");
  }
  useEffect(() => {
    cancel();
    setProblem("");
  }, [track, disabled, signature]);
  const propose = (d: Gesture, x: number) =>
    moveClipGroup(track, d.ids, point(x) - d.anchor, snap ? 8 / pixels : 0);
  function commit(next: ClipGroupMotion) {
    setProblem(next.problem);
    if (!next.problem && next.delta !== 0)
      selection?.onMove(next.ids, next.destination);
  }
  const point = (x: number) =>
    timelinePoint(
      x,
      surface.current?.getBoundingClientRect().left ?? 0,
      viewport,
    );
  function update(clientX: number, clientY: number) {
    const d = active.current;
    if (!d) return;
    const moved = d.moved || Math.abs(clientX - d.x) >= 3;
    active.current = {
      ...d,
      clientX,
      clientY,
      moved,
      next: moved ? propose(d, clientX) : d.next,
    };
    setDraft(active.current);
  }
  useTimelineEdgeScroll(
    surface,
    viewport,
    track.outMs - track.inMs,
    draft,
    onPan,
    update,
    cancel,
  );
  return {
    draft,
    problem,
    cancel,
    captureLost() {
      if (active.current) cancel();
    },
    begin(e: PointerEvent, id: string) {
      if (!selection || e.button !== 0) return;
      e.preventDefault();
      e.stopPropagation();
      (e.currentTarget as HTMLElement).focus();
      if (disabled) return;
      // Only selected clips initiate movement; clicking other clips adds them first.
      if (!selection.ids.includes(id)) {
        selection.onPick(id, e.shiftKey);
        return;
      }
      if (e.shiftKey) {
        selection.onPick(id, true);
        return;
      }
      surface.current?.setPointerCapture(e.pointerId);
      setProblem("");
      active.current = {
        pointer: e.pointerId,
        x: e.clientX,
        clientX: e.clientX,
        clientY: e.clientY,
        anchor: point(e.clientX),
        moved: false,
        id,
        ids: [...selection.ids],
        next: moveClipGroup(track, selection.ids, 0),
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
      cancel();
      if (surface.current?.hasPointerCapture(e.pointerId))
        surface.current.releasePointerCapture(e.pointerId);
      if (d.moved || Math.abs(e.clientX - d.x) >= 3)
        commit(propose(d, e.clientX));
      else selection?.onPick(d.id, false);
    },
    nudge(delta: number) {
      if (!disabled && selection)
        commit(moveClipGroup(track, selection.ids, delta));
    },
  };
}
