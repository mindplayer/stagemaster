import {
  useEffect,
  useRef,
  useState,
  type PointerEvent,
  type RefObject,
} from "react";
import type { AudioTimeline } from "../../audio-types";
import type { AudioLoopRegion } from "../../audio-performance-types";
import type { WaveViewport } from "./waveform-data";
import type { LoopLaneSelection } from "./loop-lane-selection";
import {
  moveLoopGroup,
  moveLoopRegion,
  type LoopMotion,
} from "./performance-loop-motion";
import { useTimelineEdgeScroll } from "./useTimelineEdgeScroll";
import { timelinePoint } from "./timeline-edge-scroll";
type Drag = {
  pointer: number;
  region: AudioLoopRegion;
  mode: LoopMotion;
  ids: string[];
  x: number;
  clientX: number;
  clientY: number;
  anchor: number;
  moved: boolean;
  items: AudioLoopRegion[];
  destination: number;
  problem: string;
};
export function usePerformanceLoopDrag(
  surface: RefObject<HTMLDivElement | null>,
  track: AudioTimeline,
  viewport: WaveViewport,
  disabled: boolean,
  snap: boolean,
  selection: LoopLaneSelection,
  select: (id: string) => void,
  pan: (pixels: number) => void,
) {
  const active = useRef<Drag | null>(null);
  const [draft, setDraft] = useState<Drag | null>(null);
  const signature = `${selection.active}:${selection.ids.join("|")}:${selection.blocked}`;
  const pixels = viewport.width / Math.max(1, viewport.end - viewport.start);
  function cancel() {
    const d = active.current;
    active.current = null;
    setDraft(null);
    if (d && surface.current?.hasPointerCapture(d.pointer))
      surface.current.releasePointerCapture(d.pointer);
  }
  useEffect(cancel, [track, disabled, signature]);
  const point = (x: number) =>
    timelinePoint(
      x,
      surface.current?.getBoundingClientRect().left ?? 0,
      viewport,
    );
  function proposal(d: Drag, x: number) {
    const delta = point(x) - d.anchor,
      tolerance = snap ? 8 / pixels : 0;
    if (d.ids.length) return moveLoopGroup(track, d.ids, delta, tolerance);
    const next = moveLoopRegion(track, d.region, d.mode, delta, tolerance);
    return { items: [next], destination: next.startMs, problem: "" };
  }
  function update(clientX: number, clientY: number) {
    const d = active.current;
    if (!d) return;
    const moved = d.moved || Math.abs(clientX - d.x) >= 3;
    active.current = {
      ...d,
      clientX,
      clientY,
      moved,
      ...(moved ? proposal(d, clientX) : {}),
    };
    setDraft(active.current);
  }
  useTimelineEdgeScroll(
    surface,
    viewport,
    track.outMs - track.inMs,
    draft,
    pan,
    update,
    cancel,
  );
  return {
    draft,
    cancel,
    begin(
      e: PointerEvent,
      region: AudioLoopRegion,
      mode: LoopMotion,
      movingGroup: boolean,
    ) {
      e.stopPropagation();
      if (disabled || selection.blocked || e.button !== 0) return;
      e.preventDefault();
      (e.currentTarget as HTMLElement).focus();
      if (
        selection.active &&
        (!movingGroup || !selection.ids.includes(region.id))
      ) {
        selection.onPick(region.id, e.shiftKey);
        return;
      }
      if (!selection.active) select(region.id);
      if (region.locked) return;
      const ids = selection.active ? selection.ids : [];
      if (ids.length && moveLoopGroup(track, ids, 0).problem) return;
      surface.current?.setPointerCapture(e.pointerId);
      active.current = {
        pointer: e.pointerId,
        region,
        mode,
        ids,
        x: e.clientX,
        clientX: e.clientX,
        clientY: e.clientY,
        anchor: point(e.clientX),
        moved: false,
        items: [],
        destination: region.startMs,
        problem: "",
      };
      setDraft(active.current);
    },
    move(e: PointerEvent) {
      if (active.current?.pointer === e.pointerId) update(e.clientX, e.clientY);
    },
    end(e: PointerEvent) {
      const d = active.current;
      if (!d || d.pointer !== e.pointerId) return;
      const moved = d.moved || Math.abs(e.clientX - d.x) >= 3;
      const next = moved ? proposal(d, e.clientX) : null;
      cancel();
      if (!next || next.problem) return;
      if (d.ids.length) {
        const first = track.loopRegions?.find((r) =>
          d.ids.includes(r.id),
        )?.startMs;
        if (next.destination !== first)
          selection.onMove(d.ids, next.destination);
      } else if (
        next.items[0].startMs !== d.region.startMs ||
        next.items[0].endMs !== d.region.endMs
      )
        selection.onEdit(d.region, next.items[0], d.mode);
    },
  };
}
