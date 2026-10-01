import type { RefObject } from "react";
import type { SceneView } from "../../application-host";
import type { AudioMarker, AudioTimeline } from "../../audio-types";
import { audioTime } from "../../audio-tools";
import type { WaveViewport } from "./waveform-data";
import { lightingSegments, constrainBoundaryTime } from "./lighting-segments";
import "./lighting-lane.css";

export function AudioLightingLane({
  track,
  scenes,
  viewport,
  selected,
  disabled,
  cursor,
  onSelect,
  onSeek,
  onMove,
}: {
  track: AudioTimeline;
  scenes: SceneView[];
  viewport: WaveViewport;
  selected: string;
  disabled: boolean;
  cursor: RefObject<HTMLDivElement | null>;
  onSelect(id: string): void;
  onSeek(time: number): void;
  onMove(marker: AudioMarker): void;
}) {
  const segments = lightingSegments(track);
  const pixels = viewport.width / Math.max(1, viewport.end - viewport.start);
  return (
    <div className="audio-lighting-lane" role="group" aria-label="灯光段落">
      <header>
        <strong>灯光场景</strong>
        <span>{segments.filter((s) => s.markerId).length} 段</span>
        <span>拖动边界调整切换时间</span>
      </header>
      <div className="audio-lighting-clips">
        {segments.map((segment) => {
          if (segment.end < viewport.start || segment.start > viewport.end)
            return null;
          const start = Math.max(segment.start, viewport.start);
          const end = Math.min(segment.end, viewport.end);
          const left = (start - viewport.start) * pixels;
          const width = Math.max(1, (end - start) * pixels);
          const scene = scenes.find((s) => s.id === segment.sceneId);
          const label = scene?.name ?? "灯具默认值";
          const range = `${audioTime(segment.start)} — ${audioTime(segment.end)}`;
          const marker = track.markers.find((m) => m.id === segment.markerId);
          return (
            <div
              key={segment.markerId ?? "defaults"}
              className={`audio-lighting-clip ${selected === segment.markerId ? "selected" : ""} ${marker ? "" : "defaults"}`}
              style={{ left, width }}
            >
              {marker ? (
                <button
                  className="audio-lighting-select"
                  data-lighting-segment={marker.id}
                  aria-pressed={selected === marker.id}
                  aria-label={`${label}，${range}`}
                  title={`${label} · ${range} · ${marker.fadeMs ? `渐变 ${(marker.fadeMs / 1000).toFixed(3)} 秒` : "直接切换"} · 双击定位`}
                  disabled={disabled}
                  onClick={() => onSelect(marker.id)}
                  onDoubleClick={() => onSeek(marker.timeMs)}
                  onKeyDown={(e) => {
                    if (
                      ["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)
                    )
                      e.stopPropagation();
                  }}
                >
                  <strong>{label}</strong>
                  {width > 105 && (
                    <small>
                      {((segment.end - segment.start) / 1000).toLocaleString(
                        "zh-CN",
                      )}{" "}
                      秒
                    </small>
                  )}
                </button>
              ) : (
                <span className="audio-lighting-default-label">{label}</span>
              )}
              {marker?.fadeMs && segment.start + marker.fadeMs > start ? (
                <span
                  className="audio-lighting-fade"
                  aria-hidden="true"
                  style={{
                    width:
                      Math.max(
                        0,
                        Math.min(end, segment.start + marker.fadeMs) - start,
                      ) * pixels,
                  }}
                />
              ) : null}
              {marker && segment.start >= viewport.start && (
                <button
                  className="audio-lighting-boundary"
                  data-marker={marker.id}
                  data-boundary="true"
                  onClick={() => onSelect(marker.id)}
                  aria-label={`调整${label}的开始时间`}
                  title={`${audioTime(marker.timeMs)} · 左右键调整 10 毫秒，Shift 调整 1 秒`}
                  disabled={disabled}
                  onKeyDown={(e) => {
                    if (["Home", "End"].includes(e.key)) e.stopPropagation();
                    if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
                      e.preventDefault();
                      e.stopPropagation();
                      const timeMs = constrainBoundaryTime(
                        track,
                        marker.id,
                        marker.timeMs +
                          (e.key === "ArrowLeft" ? -1 : 1) *
                            (e.shiftKey ? 1000 : 10),
                      );
                      onSelect(marker.id);
                      if (timeMs !== marker.timeMs)
                        onMove({ ...marker, timeMs });
                    }
                  }}
                />
              )}
            </div>
          );
        })}
        <div
          className="audio-lighting-cursor"
          ref={cursor}
          aria-hidden="true"
        />
      </div>
    </div>
  );
}
