import type { ClipLaneSelection } from "./clip-selection";
import { AudioClipLane } from "./AudioClipLane";
import type { SceneView } from "../../application-host";
import { AudioLightingLane } from "./AudioLightingLane";
import { constrainBoundaryTime } from "./lighting-segments";
import { useEffect, useRef, useState, type RefObject } from "react";
import type {
  AudioMarker,
  AudioLightingClip,
  AudioPosition,
  AudioTimeline,
} from "../../audio-types";
import { audioTime, snapAudioTime } from "../../audio-tools";
import { viewTime, type WaveViewport } from "./waveform-data";
type Drag = {
  pointer: number;
  marker?: AudioMarker;
  time: number;
  anchor: number;
  original: number;
  boundary: boolean;
};
export function WaveformMarkers({
  track,
  scenes,
  laneCursor,
  viewport,
  selected,
  disabled,
  snap,
  sample,
  preview,
  onMove,
  onClipMove,
  clipSelection,
  onSeek,
  onSelect,
  onZoom,
  onPan,
}: {
  track: AudioTimeline;
  scenes?: SceneView[];
  laneCursor: RefObject<HTMLDivElement | null>;
  viewport: WaveViewport;
  selected: string;
  disabled: boolean;
  snap: boolean;
  sample: RefObject<{ position: AudioPosition; at: number }>;
  preview: RefObject<number | null>;
  onMove(marker: AudioMarker): void;
  onClipMove?(clip: AudioLightingClip): void;
  clipSelection?: ClipLaneSelection;
  onSeek(time: number): void;
  onSelect(id: string): void;
  onZoom(factor: number, x: number): void;
  onPan(pixels: number): void;
}) {
  const surface = useRef<HTMLDivElement>(null);
  const active = useRef<Drag | null>(null);
  const [drag, setDrag] = useState<Drag | null>(null);
  const [hover, setHover] = useState<number | null>(null);
  const handlers = useRef({ onZoom, onPan });
  handlers.current = { onZoom, onPan };
  const duration = track.outMs - track.inMs;
  const pixels = viewport.width / Math.max(1, viewport.end - viewport.start);
  function cancel() {
    active.current = null;
    preview.current = null;
    setDrag(null);
  }
  useEffect(() => {
    if (disabled) cancel();
  }, [disabled]);
  useEffect(() => {
    const el = surface.current!;
    const wheel = (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey || e.altKey) {
        e.preventDefault();
        handlers.current.onZoom(
          Math.exp(-e.deltaY * 0.005),
          e.clientX - el.getBoundingClientRect().left,
        );
      } else if (e.shiftKey || Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
        e.preventDefault();
        handlers.current.onPan(e.deltaX || e.deltaY);
      }
    };
    el.addEventListener("wheel", wheel, { passive: false });
    return () => {
      el.removeEventListener("wheel", wheel);
      preview.current = null;
    };
  }, []);
  function point(clientX: number) {
    return viewTime(
      clientX,
      surface.current!.getBoundingClientRect().left,
      viewport,
    );
  }
  function dragTime(d: Drag, raw: number) {
    const time = snapAudioTime(
      d.marker ? d.original + raw - d.anchor : raw,
      track,
      snap,
      d.marker?.id,
      8 / pixels,
    );
    return d.boundary && d.marker
      ? constrainBoundaryTime(track, d.marker.id, time)
      : time;
  }
  return (
    <div
      ref={surface}
      className="audio-wave-interaction"
      data-lighting={!!scenes}
      tabIndex={0}
      aria-label="音乐波形：点击定位，左右键微调，Esc 取消拖动"
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          cancel();
          e.stopPropagation();
        }
        if (disabled) return;
        if (["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)) {
          e.preventDefault();
          const time =
            e.key === "Home"
              ? 0
              : e.key === "End"
                ? duration
                : sample.current.position.positionMs +
                  (e.key === "ArrowLeft" ? -1 : 1) * (e.shiftKey ? 1000 : 10);
          onSeek(Math.max(0, Math.min(duration, time)));
        }
      }}
      onPointerDown={(e) => {
        if (disabled || e.button !== 0) return;
        if ((e.target as HTMLElement).closest("[data-lighting-segment]"))
          return;
        e.preventDefault();
        e.currentTarget.focus();
        e.currentTarget.setPointerCapture(e.pointerId);
        const button = (e.target as HTMLElement).closest<HTMLElement>(
          "[data-marker]",
        );
        const marker = track.markers.find(
          (m) => m.id === button?.dataset.marker,
        );
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
        };
        setDrag(active.current);
        preview.current = time;
        if (marker) onSelect(marker.id);
      }}
      onPointerMove={(e) => {
        if ((e.target as HTMLElement).closest(".audio-lighting-lane")) {
          setHover(null);
          return;
        }
        const raw = point(e.clientX);
        setHover(Math.max(0, Math.min(duration, raw)));
        const d = active.current;
        if (!d || d.pointer !== e.pointerId) return;
        const time = dragTime(d, raw);
        active.current = { ...d, time };
        setDrag(active.current);
        preview.current = time;
      }}
      onPointerUp={(e) => {
        const d = active.current;
        if (!d || d.pointer !== e.pointerId) return;
        const time = dragTime(d, point(e.clientX));
        cancel();
        e.currentTarget.releasePointerCapture(e.pointerId);
        if (d.marker) {
          if (time !== d.original) onMove({ ...d.marker, timeMs: time });
        } else onSeek(time);
      }}
      onPointerLeave={() => setHover(null)}
      onPointerCancel={cancel}
      onLostPointerCapture={cancel}
    >
      {track.markers.map((marker, i) => {
        const time = drag?.marker?.id === marker.id ? drag.time : marker.timeMs;
        const x = (time - viewport.start) * pixels;
        if (x < -10 || x > viewport.width) return null;
        const previous = track.markers[i - 1];
        const label =
          selected === marker.id ||
          !previous ||
          (time - previous.timeMs) * pixels > 100;
        return (
          <button
            key={marker.id}
            data-marker={marker.id}
            disabled={disabled}
            className={`audio-wave-marker ${marker.sceneId ? "bound" : ""} ${selected === marker.id ? "selected" : ""} ${label ? "with-label" : ""}`}
            style={{ left: x }}
            aria-label={`${marker.name}，${audioTime(marker.timeMs)}`}
            title={`${marker.name} · ${audioTime(marker.timeMs)}`}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.stopPropagation();
                onSelect(marker.id);
                onSeek(marker.timeMs);
              }
            }}
          >
            <span>{marker.name}</span>
          </button>
        );
      })}
      {scenes && track.lightingClips && onClipMove ? (
        <AudioClipLane
          onPan={onPan}
          clipSelection={clipSelection}
          track={track}
          scenes={scenes}
          viewport={viewport}
          selected={selected}
          disabled={disabled}
          snap={snap}
          cursor={laneCursor}
          onSelect={onSelect}
          onSeek={onSeek}
          onMove={onClipMove}
        />
      ) : (
        scenes && (
          <AudioLightingLane
            track={
              drag?.marker
                ? {
                    ...track,
                    markers: track.markers.map((m) =>
                      m.id === drag.marker!.id
                        ? { ...m, timeMs: drag.time }
                        : m,
                    ),
                  }
                : track
            }
            scenes={scenes}
            viewport={viewport}
            selected={selected}
            disabled={disabled}
            cursor={laneCursor}
            onSelect={onSelect}
            onSeek={onSeek}
            onMove={onMove}
          />
        )
      )}
      {(drag || hover !== null) && (
        <output
          className="audio-wave-tooltip"
          style={{
            left: Math.max(
              4,
              Math.min(
                viewport.width - 94,
                ((drag?.time ?? hover!) - viewport.start) * pixels + 8,
              ),
            ),
          }}
        >
          {audioTime(Math.round(drag?.time ?? hover!))}
        </output>
      )}
    </div>
  );
}
