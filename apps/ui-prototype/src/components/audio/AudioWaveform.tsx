import { useEffect, useRef, useState, type RefObject } from "react";
import type {
  AudioMarker,
  AudioPosition,
  AudioTimeline,
  AudioWaveform as Wave,
} from "../../audio-types";
import { audioTime, snapAudioTime } from "../../audio-tools";
type Drag = {
  pointer: number;
  marker?: AudioMarker;
  time: number;
  original: number;
  anchor: number;
};
export function AudioWaveform({
  track,
  waveform,
  sample,
  selected,
  disabled,
  onSeek,
  onSelect,
  onMove,
}: {
  track: AudioTimeline;
  waveform: Wave | null;
  sample: RefObject<{ position: AudioPosition; at: number }>;
  selected: string;
  disabled: boolean;
  onSeek(time: number): void;
  onSelect(id: string): void;
  onMove(marker: AudioMarker): void;
}) {
  const view = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const head = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(800);
  const [offset, setOffset] = useState(0);
  const [scale, setScale] = useState(60);
  const [snap, setSnap] = useState(true);
  const [follow, setFollow] = useState(true);
  const [drag, setDrag] = useState<Drag | null>(null);
  const active = useRef<Drag | null>(null);
  const duration = track.outMs - track.inMs;
  const px = Math.max(scale, width / (duration / 1000));
  const full = Math.max(width, (duration / 1000) * px);
  useEffect(() => {
    const el = view.current;
    if (!el) return;
    const observer = new ResizeObserver(() => setWidth(el.clientWidth));
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  useEffect(() => {
    const el = canvas.current;
    if (!el) return;
    const ratio = Math.min(2, window.devicePixelRatio || 1);
    el.width = Math.ceil(width * ratio);
    el.height = 180 * ratio;
    el.style.width = `${width}px`;
    const ctx = el.getContext("2d");
    if (!ctx) return;
    ctx.scale(ratio, ratio);
    ctx.fillStyle = "#102630";
    ctx.fillRect(0, 0, width, 180);
    const step =
      [0.2, 0.5, 1, 2, 5, 10, 15, 30, 60, 120, 300, 600].find(
        (value) => value * px >= 90,
      ) ?? 600;
    ctx.font = "11px system-ui";
    ctx.textBaseline = "top";
    for (
      let seconds = Math.ceil(offset / px / step) * step;
      seconds * px < offset + width;
      seconds += step
    ) {
      const x = seconds * px - offset;
      ctx.strokeStyle = "#23414c";
      ctx.beginPath();
      ctx.moveTo(x, 25);
      ctx.lineTo(x, 180);
      ctx.stroke();
      ctx.fillStyle = "#a5bfca";
      ctx.fillText(
        step >= 1
          ? audioTime(Math.round(seconds * 1000)).slice(0, 5)
          : audioTime(Math.round(seconds * 1000)).slice(0, -1),
        x + 4,
        7,
      );
    }
    if (!waveform) return;
    ctx.strokeStyle = "#48d4bc";
    ctx.lineWidth = 1;
    for (let x = 0; x < width; x++) {
      const start = Math.floor(
        (((x + offset) / px) * 1000 + track.inMs) / waveform.bucketMs,
      );
      const end = Math.ceil(
        (((x + offset + 1) / px) * 1000 + track.inMs) / waveform.bucketMs,
      );
      let value = 0;
      for (let i = start; i <= Math.min(end, waveform.peaks.length - 1); i++)
        value = Math.max(value, waveform.peaks[i] ?? 0);
      ctx.beginPath();
      ctx.moveTo(x, 108 - value * 54);
      ctx.lineTo(x, 108 + Math.max(1, value * 54));
      ctx.stroke();
    }
  }, [width, offset, px, waveform, track.inMs]);
  useEffect(() => {
    let frame = 0;
    function draw() {
      const current = sample.current;
      const pos =
        active.current?.time ??
        Math.min(
          duration,
          current.position.positionMs +
            (current.position.playing
              ? Math.max(0, Math.min(120, performance.now() - current.at))
              : 0),
        );
      if (head.current)
        head.current.style.transform = `translateX(${(pos / 1000) * px}px)`;
      if (
        follow &&
        current.position.playing &&
        !active.current &&
        view.current
      ) {
        const left = (pos / 1000) * px;
        if (
          left < view.current.scrollLeft ||
          left > view.current.scrollLeft + width * 0.85
        )
          view.current.scrollLeft = Math.max(0, left - width * 0.2);
      }
      frame = requestAnimationFrame(draw);
    }
    frame = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(frame);
  }, [sample, px, duration, follow, width]);
  function rawPoint(clientX: number) {
    const rect = view.current!.getBoundingClientRect();
    return ((clientX - rect.left + view.current!.scrollLeft) / px) * 1000;
  }
  function point(clientX: number, except?: string) {
    const held = active.current;
    const time = held?.marker
      ? held.original + rawPoint(clientX) - held.anchor
      : rawPoint(clientX);
    return snapAudioTime(time, track, snap, except, (8 / px) * 1000);
  }
  function cancel() {
    active.current = null;
    setDrag(null);
  }
  function center() {
    const pos = sample.current.position.positionMs;
    view.current?.scrollTo({
      left: Math.max(0, (pos / 1000) * px - width / 2),
    });
  }
  return (
    <section className="audio-timeline" aria-label="音乐时间线">
      <div className="audio-zoom">
        <label>
          <input
            type="checkbox"
            checked={follow}
            onChange={(e) => setFollow(e.target.checked)}
          />
          跟随播放
        </label>
        <label>
          <input
            type="checkbox"
            checked={snap}
            onChange={(e) => setSnap(e.target.checked)}
          />
          吸附卡点
        </label>
        <label>
          缩放
          <input
            aria-label="波形缩放"
            type="range"
            min="0"
            max="400"
            value={scale}
            onChange={(e) => setScale(Number(e.target.value))}
          />
        </label>
        <button
          onClick={() => {
            setScale(0);
            view.current?.scrollTo({ left: 0 });
          }}
        >
          显示全曲
        </button>
        <button onClick={center}>定位播放头</button>
      </div>
      <div
        ref={view}
        className="audio-wave-scroll"
        tabIndex={0}
        aria-label="波形；左右键定位，Esc 取消拖动"
        onScroll={(e) => setOffset(e.currentTarget.scrollLeft)}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            cancel();
            e.stopPropagation();
          } else if (
            !disabled &&
            (e.key === "ArrowLeft" || e.key === "ArrowRight")
          ) {
            e.preventDefault();
            onSeek(
              Math.max(
                0,
                Math.min(
                  duration,
                  sample.current.position.positionMs +
                    (e.key === "ArrowLeft" ? -1 : 1) * (e.shiftKey ? 1000 : 10),
                ),
              ),
            );
          }
        }}
        onPointerDown={(e) => {
          if (disabled || e.button !== 0) return;
          e.preventDefault();
          e.currentTarget.focus();
          e.currentTarget.setPointerCapture(e.pointerId);
          const m = (e.target as HTMLElement).closest<HTMLElement>(
            "[data-marker]",
          );
          const marker = track.markers.find((v) => v.id === m?.dataset.marker);
          const time = marker?.timeMs ?? point(e.clientX);
          const next = {
            pointer: e.pointerId,
            marker,
            time,
            original: time,
            anchor: rawPoint(e.clientX),
          };
          active.current = next;
          setDrag(next);
          if (marker) onSelect(marker.id);
        }}
        onPointerMove={(e) => {
          const d = active.current;
          if (!d || d.pointer !== e.pointerId) return;
          const next = { ...d, time: point(e.clientX, d.marker?.id) };
          active.current = next;
          setDrag(next);
        }}
        onPointerUp={(e) => {
          const d = active.current;
          if (!d || d.pointer !== e.pointerId) return;
          cancel();
          e.currentTarget.releasePointerCapture(e.pointerId);
          if (d.marker && d.time !== d.original)
            onMove({ ...d.marker, timeMs: d.time });
          else onSeek(d.time);
        }}
        onPointerCancel={cancel}
        onLostPointerCapture={cancel}
      >
        <div className="audio-wave-content" style={{ width: full }}>
          <canvas ref={canvas} />
          {track.markers.map((m) => (
            <button
              key={m.id}
              data-marker={m.id}
              className={`audio-marker ${selected === m.id ? "selected" : ""} ${m.sceneId ? "bound" : ""}`}
              style={{
                left:
                  ((drag?.marker?.id === m.id ? drag.time : m.timeMs) / 1000) *
                  px,
              }}
              title={`${m.name} · ${audioTime(m.timeMs)}`}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  onSelect(m.id);
                  onSeek(m.timeMs);
                }
              }}
            >
              <span>{m.name}</span>
            </button>
          ))}
          <div className="audio-playhead" ref={head}>
            <span />
          </div>
        </div>
      </div>
      <input
        className="audio-seek"
        type="range"
        aria-label="音乐播放位置"
        min="0"
        max={duration}
        step="1"
        disabled={disabled}
        value={sample.current.position.positionMs}
        onChange={(e) => onSeek(Number(e.target.value))}
      />
    </section>
  );
}
