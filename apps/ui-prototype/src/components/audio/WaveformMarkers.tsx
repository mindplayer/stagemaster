import type { ClipLaneSelection } from "./clip-selection";
import { AudioClipLane } from "./AudioClipLane";
import type { SceneView } from "../../application-host";
import { AudioLightingLane } from "./AudioLightingLane";
import { useWaveMarkerDrag } from "./useWaveMarkerDrag";
import { useEffect, useRef, useState, type RefObject } from "react";
import type {
  AudioMarker,
  AudioLightingClip,
  AudioPosition,
  AudioTimeline,
} from "../../audio-types";
import { audioTime } from "../../audio-tools";
import { viewTime, type WaveViewport } from "./waveform-data";
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
  onClipMove?(clip: AudioLightingClip, mode: "move" | "start" | "end"): void;
  clipSelection?: ClipLaneSelection;
  onSeek(time: number): void;
  onSelect(id: string): void;
  onZoom(factor: number, x: number): void;
  onPan(pixels: number): void;
}) {
  const surface = useRef<HTMLDivElement>(null);
  const [hover, setHover] = useState<number | null>(null);
  const handlers = useRef({ onZoom, onPan });
  handlers.current = { onZoom, onPan };
  const duration = track.outMs - track.inMs;
  const pixels = viewport.width / Math.max(1, viewport.end - viewport.start);
  const gesture = useWaveMarkerDrag(
    surface,
    track,
    viewport,
    disabled,
    snap,
    preview,
    onSelect,
    onMove,
    onSeek,
    onPan,
  );
  const drag = gesture.draft;
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
  return (
    <div
      ref={surface}
      className="audio-wave-interaction"
      data-lighting={!!scenes}
      tabIndex={0}
      aria-label="音乐波形：点击定位，左右键微调，Esc 取消拖动"
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          gesture.cancel();
          e.stopPropagation();
        }
        if (disabled) return;
        if (["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)) {
          e.preventDefault();
          if (drag) return;
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
      onPointerDown={gesture.begin}
      onPointerMove={(e) => {
        gesture.move(e);
        if ((e.target as HTMLElement).closest(".audio-lighting-lane")) {
          setHover(null);
          return;
        }
        setHover(Math.max(0, Math.min(duration, point(e.clientX))));
      }}
      onPointerUp={gesture.end}
      onPointerLeave={() => setHover(null)}
      onPointerCancel={gesture.cancel}
      onLostPointerCapture={gesture.cancel}
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
