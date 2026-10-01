import {
  useEffect,
  useRef,
  useState,
  type PointerEvent,
  type RefObject,
} from "react";
import type { AudioMarker, AudioTimeline } from "../../audio-types";
import { snapAudioTime } from "../../audio-tools";
import { markerMotion, type MarkerGrip } from "./marker-motion";
import { timelinePoint } from "./timeline-edge-scroll";
import { useTimelineEdgeScroll } from "./useTimelineEdgeScroll";
import type { WaveViewport } from "./waveform-data";

interface Drag extends MarkerGrip {
  pointer: number;
  time: number;
  clientX: number;
  clientY: number;
}

/** Marker and legacy boundary editing share one draft; seeking stays separate. */
export function useWaveMarkerDrag(
  surface: RefObject<HTMLDivElement | null>,
  track: AudioTimeline,
  viewport: WaveViewport,
  disabled: boolean,
  snap: boolean,
  preview: RefObject<number | null>,
  onSelect: (id: string) => void,
  onMove: (marker: AudioMarker) => void,
  onSeek: (time: number) => void,
  onPan: (pixels: number) => void,
) {
  const active = useRef<Drag | null>(null);
  const [draft, setDraft] = useState<Drag | null>(null);
  const pixels = viewport.width / Math.max(1, viewport.end - viewport.start);
  function cancel() {
    active.current = null;
    preview.current = null;
    setDraft(null);
  }
  useEffect(cancel, [track, disabled, snap]);
  const point = (x: number) =>
    timelinePoint(
      x,
      surface.current?.getBoundingClientRect().left ?? 0,
      viewport,
    );
  const proposal = (d: Drag, x: number) =>
    markerMotion(d, point(x), x, track, snap, 8 / pixels);
  function update(clientX: number, clientY: number) {
    const d = active.current;
    if (!d) return;
    active.current = { ...d, ...proposal(d, clientX), clientX, clientY };
    setDraft(active.current);
    preview.current = active.current.time;
  }
  useTimelineEdgeScroll(
    surface,
    viewport,
    track.outMs - track.inMs,
    draft,
    draft?.marker ? onPan : undefined,
    update,
    cancel,
  );
  return {
    draft,
    cancel,
    begin(e: PointerEvent<HTMLDivElement>) {
      if (disabled || e.button !== 0) return;
      if ((e.target as HTMLElement).closest("[data-lighting-segment]")) return;
      e.preventDefault();
      e.currentTarget.focus();
      e.currentTarget.setPointerCapture(e.pointerId);
      const button = (e.target as HTMLElement).closest<HTMLElement>(
        "[data-marker]",
      );
      const marker = track.markers.find((m) => m.id === button?.dataset.marker);
      const raw = point(e.clientX);
      const time =
        marker?.timeMs ??
        snapAudioTime(raw, track, snap, undefined, 8 / pixels);
      active.current = {
        pointer: e.pointerId,
        marker,
        time,
        original: time,
        anchor: raw,
        boundary: button?.dataset.boundary === "true",
        x: e.clientX,
        clientX: e.clientX,
        clientY: e.clientY,
        moved: false,
      };
      setDraft(active.current);
      preview.current = time;
      if (marker) onSelect(marker.id);
    },
    move(e: PointerEvent) {
      if (active.current?.pointer === e.pointerId) update(e.clientX, e.clientY);
    },
    end(e: PointerEvent<HTMLDivElement>) {
      const d = active.current;
      if (!d || d.pointer !== e.pointerId) return;
      const { time } = proposal(d, e.clientX);
      cancel();
      if (e.currentTarget.hasPointerCapture(e.pointerId))
        e.currentTarget.releasePointerCapture(e.pointerId);
      if (d.marker) {
        if (time !== d.original) onMove({ ...d.marker, timeMs: time });
      } else onSeek(time);
    },
  };
}
