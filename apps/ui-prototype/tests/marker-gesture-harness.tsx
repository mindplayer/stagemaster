// Deliberate held-pointer injection for hook lifecycle acceptance, not a native gesture.
// The real WaveSurfer component and real release are tested by timeline-scroll.html.
import { useRef, useState, type PointerEvent } from "react";
import { createRoot } from "react-dom/client";
import { useWaveMarkerDrag } from "../src/components/audio/useWaveMarkerDrag";
import type { AudioTimeline } from "../src/audio-types";
import "../src/base.css";
const initial: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "保持手势.wav",
    extension: "wav",
    durationMs: 60000,
  },
  inMs: 0,
  outMs: 60000,
  markers: [
    { id: "a", name: "起点", timeMs: 3000, sceneId: "scene", fadeMs: 500 },
    { id: "b", name: "终点", timeMs: 40000, sceneId: "scene", fadeMs: 1000 },
  ],
};
function Harness() {
  const surface = useRef<HTMLDivElement>(null),
    marker = useRef<HTMLButtonElement>(null);
  const preview = useRef<number | null>(null);
  const [track, setTrack] = useState(initial);
  const [viewport, setViewport] = useState({ start: 0, end: 6000, width: 600 });
  const [visible, setVisible] = useState(true),
    [disabled, setDisabled] = useState(false);
  const [edits, setEdits] = useState(0),
    [seeks, setSeeks] = useState(0);
  const gesture = useWaveMarkerDrag(
    surface,
    track,
    viewport,
    disabled,
    false,
    preview,
    () => {},
    () => setEdits((n) => n + 1),
    () => setSeeks((n) => n + 1),
    (dx) => {
      if (!dx) return;
      setViewport((v) => {
        const span = v.end - v.start;
        const start = Math.max(
          0,
          Math.min(60000 - span, v.start + (dx * span) / v.width),
        );
        return { ...v, start, end: start + span };
      });
    },
  );
  function hold(event: PointerEvent, boundary: boolean, direction: number) {
    if (!surface.current || !marker.current) return;
    const r = surface.current.getBoundingClientRect();
    marker.current.dataset.boundary = String(boundary);
    const x =
      r.left +
      ((3000 - viewport.start) * viewport.width) /
        (viewport.end - viewport.start);
    gesture.begin({
      button: 0,
      pointerId: event.pointerId,
      currentTarget: surface.current,
      target: marker.current,
      clientX: x,
      clientY: r.top + 20,
      preventDefault() {},
    } as unknown as PointerEvent<HTMLDivElement>);
    gesture.move({
      pointerId: event.pointerId,
      clientX: direction > 0 ? r.right - 1 : r.left + 1,
      clientY: r.top + 20,
    } as PointerEvent);
  }
  return (
    <main style={{ padding: 24 }}>
      <p>
        隔离注入保持态，释放事件故意不接线；只验证真实 hook
        的滚动／取消生命周期。
      </p>
      <button onPointerDown={(e) => hold(e, false, 1)}>卡点向右保持</button>
      <button onPointerDown={(e) => hold(e, true, 1)}>段落向右保持</button>
      <button
        onClick={() => setViewport((v) => ({ ...v, end: v.start + 3000 }))}
      >
        改变缩放
      </button>
      <button onClick={() => setVisible((v) => !v)}>切换显示</button>
      <button onClick={() => setDisabled((v) => !v)}>切换禁用</button>
      <button
        onClick={() => setTrack((t) => ({ ...t, markers: [...t.markers] }))}
      >
        替换卡点快照
      </button>
      <button
        onClick={() => {
          gesture.cancel();
          setViewport({ start: 0, end: 6000, width: 600 });
          setVisible(true);
          setDisabled(false);
        }}
      >
        复位
      </button>
      <output aria-label="手势结果">
        {JSON.stringify({
          active: !!gesture.draft,
          time: gesture.draft?.time ?? null,
          start: Math.round(viewport.start),
          end: Math.round(viewport.end),
          preview: preview.current,
          edits,
          seeks,
        })}
      </output>
      <div
        ref={surface}
        tabIndex={0}
        style={{
          width: 600,
          height: 120,
          background: "#24434b",
          display: visible ? "block" : "none",
        }}
      >
        <button ref={marker} data-marker="a">
          卡点目标
        </button>
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
