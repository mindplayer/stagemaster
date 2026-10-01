import { useEffect, useRef, useState, type RefObject } from "react";
import type { AudioLightingClip, AudioTimeline } from "../../audio-types";
import type { SceneView } from "../../application-host";
import type { WaveViewport } from "./waveform-data";
import { audioTime } from "../../audio-tools";
import { moveLightingClip, type ClipMotion } from "./clip-motion";
import "./lighting-lane.css";
type Drag = {
  pointer: number;
  clip: AudioLightingClip;
  mode: ClipMotion;
  x: number;
  next: AudioLightingClip;
};
export function AudioClipLane({
  track,
  scenes,
  viewport,
  selected,
  disabled,
  snap,
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
  snap: boolean;
  cursor: RefObject<HTMLDivElement | null>;
  onSelect(id: string): void;
  onSeek(time: number): void;
  onMove(clip: AudioLightingClip): void;
}) {
  const surface = useRef<HTMLDivElement>(null),
    active = useRef<Drag | null>(null);
  const [drag, setDrag] = useState<Drag | null>(null);
  const pixels = viewport.width / Math.max(1, viewport.end - viewport.start);
  function cancel() {
    active.current = null;
    setDrag(null);
  }
  useEffect(() => {
    cancel();
  }, [track, disabled]);
  useEffect(() => {
    window.addEventListener("blur", cancel);
    return () => window.removeEventListener("blur", cancel);
  }, []);
  function proposal(d: Drag, x: number) {
    return moveLightingClip(
      track,
      d.clip,
      d.mode,
      (x - d.x) / pixels,
      snap,
      8 / pixels,
    );
  }
  const clips = (track.lightingClips ?? []).map((c) =>
    drag?.clip.id === c.id ? drag.next : c,
  );
  const duration = track.outMs - track.inMs;
  const gaps = clips.reduce<Array<{ start: number; end: number }>>(
    (out, c, i) => {
      const start = i ? clips[i - 1].endMs : 0;
      if (start < c.startMs) out.push({ start, end: c.startMs });
      return out;
    },
    [],
  );
  const last = clips.at(-1)?.endMs ?? 0;
  if (last < duration) gaps.push({ start: last, end: duration });
  const style = (start: number, end: number) => ({
    left: (Math.max(start, viewport.start) - viewport.start) * pixels,
    width: Math.max(
      1,
      (Math.min(end, viewport.end) - Math.max(start, viewport.start)) * pixels,
    ),
  });
  const visible = (start: number, end: number) =>
    end > viewport.start && start < viewport.end;
  function key(e: React.KeyboardEvent, c: AudioLightingClip, mode: ClipMotion) {
    if (e.key === "Escape") {
      e.stopPropagation();
      e.preventDefault();
      cancel();
      return;
    }
    if (disabled || c.locked) return;
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      e.preventDefault();
      e.stopPropagation();
      const next = moveLightingClip(
        track,
        c,
        mode,
        (e.key === "ArrowLeft" ? -1 : 1) * (e.shiftKey ? 1000 : 10),
      );
      if (next.startMs !== c.startMs || next.endMs !== c.endMs) onMove(next);
    }
  }
  return (
    <div
      className="audio-lighting-lane"
      role="group"
      aria-label="独立灯光片段"
      onPointerDown={(e) => e.stopPropagation()}
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.stopPropagation();
          cancel();
        }
      }}
    >
      <header>
        <strong>灯光片段</strong>
        <span>{clips.length} 段</span>
        <span>拖动移动 · 两端调整长度 · 空隙为默认值</span>
      </header>
      <div
        className="audio-lighting-clips"
        ref={surface}
        onPointerMove={(e) => {
          const d = active.current;
          if (!d || d.pointer !== e.pointerId) return;
          active.current = { ...d, next: proposal(d, e.clientX) };
          setDrag(active.current);
        }}
        onPointerUp={(e) => {
          const d = active.current;
          if (!d || d.pointer !== e.pointerId) return;
          const next = proposal(d, e.clientX);
          cancel();
          e.currentTarget.releasePointerCapture(e.pointerId);
          if (next.startMs !== d.clip.startMs || next.endMs !== d.clip.endMs)
            onMove(next);
        }}
        onPointerCancel={cancel}
        onLostPointerCapture={cancel}
      >
        {gaps
          .filter((g) => visible(g.start, g.end))
          .map((g) => (
            <div
              key={`gap-${g.start}`}
              className="audio-lighting-clip defaults"
              style={style(g.start, g.end)}
            >
              <span className="audio-lighting-default-label">灯具默认值</span>
            </div>
          ))}
        {clips
          .filter((c) => visible(c.startMs, c.endMs))
          .map((c) => {
            const begin = (e: React.PointerEvent, mode: ClipMotion) => {
              e.stopPropagation();
              if (disabled || e.button !== 0) return;
              onSelect(c.id);
              if (c.locked) return;
              e.preventDefault();
              (e.currentTarget as HTMLElement).focus();
              surface.current?.setPointerCapture(e.pointerId);
              active.current = {
                pointer: e.pointerId,
                clip: c,
                mode,
                x: e.clientX,
                next: c,
              };
              setDrag(active.current);
            };
            const label =
              scenes.find((s) => s.id === c.sceneId)?.name ?? c.name;
            return (
              <div
                key={c.id}
                className={`audio-lighting-clip ${selected === c.id ? "selected" : ""} ${c.locked ? "locked" : ""}`}
                style={style(c.startMs, c.endMs)}
              >
                <button
                  data-lighting-segment={c.id}
                  className="audio-lighting-select"
                  aria-pressed={selected === c.id}
                  aria-label={`${c.name}，${audioTime(c.startMs)} — ${audioTime(c.endMs)}${c.locked ? "，已锁定" : ""}`}
                  disabled={disabled}
                  onPointerDown={(e) => begin(e, "move")}
                  onClick={() => onSelect(c.id)}
                  onDoubleClick={() => onSeek(c.startMs)}
                  onKeyDown={(e) => key(e, c, "move")}
                >
                  <strong>
                    {c.locked ? "锁定 · " : ""}
                    {c.name}
                  </strong>
                  <small>{label}</small>
                </button>
                {c.fadeMs > 0 && c.startMs + c.fadeMs > viewport.start && (
                  <span
                    className="audio-lighting-fade"
                    aria-hidden="true"
                    style={{
                      width: Math.max(
                        0,
                        (Math.min(c.endMs, c.startMs + c.fadeMs, viewport.end) -
                          Math.max(c.startMs, viewport.start)) *
                          pixels,
                      ),
                    }}
                  />
                )}
                {(["start", "end"] as const).map(
                  (mode) =>
                    (mode === "start"
                      ? c.startMs >= viewport.start
                      : c.endMs <= viewport.end) && (
                      <button
                        key={mode}
                        className={`audio-lighting-boundary ${mode === "end" ? "end" : ""}`}
                        disabled={disabled || c.locked}
                        aria-label={`调整${c.name}的${mode === "start" ? "开始" : "结束"}时间`}
                        title="左右键调整 10 毫秒，Shift 调整 1 秒"
                        onPointerDown={(e) => begin(e, mode)}
                        onKeyDown={(e) => key(e, c, mode)}
                      />
                    ),
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
