import { useEffect, useRef, useState } from "react";
import type { AudioTimeline } from "../../audio-types";
import { audioTime } from "../../audio-tools";
import type { useAudio } from "./useAudio";
import { AudioPerformanceControls } from "./AudioPerformanceControls";
import { audioMediaKey } from "../../audio-performance-tools";
import "./audio-preview-transport.css";

/** Shared native audio voice, with one seek on release instead of a queue per pixel. */
export function AudioPreviewTransport({
  session,
  track,
  busy,
}: {
  session: ReturnType<typeof useAudio>;
  track: AudioTimeline | null;
  busy: boolean;
}) {
  const [draft, setDraft] = useState<number | null>(null);
  const gesture = useRef<"idle" | "dragging" | "cancelled">("idle");
  const pending = useRef<number | null>(null);
  const identity = audioMediaKey(track);
  function cancel() {
    if (gesture.current === "dragging") gesture.current = "cancelled";
    pending.current = null;
    setDraft(null);
  }
  useEffect(cancel, [identity]);
  if (!track) return null;
  const duration = track.outMs - track.inMs;
  const ready =
    session.waveform !== null && session.position.durationMs === duration;
  const blocked = busy || session.preparing || !ready;
  function commit() {
    const value = gesture.current === "dragging" ? pending.current : null;
    cancel();
    gesture.current = "idle";
    if (value !== null && !blocked)
      void session.command({ kind: "seek", positionMs: value });
  }
  return (
    <section className="audio-preview-transport" aria-label="舞台音乐播放控制">
      <div className="audio-preview-controls">
        <span title={track.asset.fileName}>音乐</span>
        <button
          disabled={(session.preparing || busy) && !session.position.playing}
          onClick={async () => {
            if (!ready && !(await session.prepare("load"))) return;
            await session.command({
              kind: session.position.playing ? "pause" : "play",
            });
          }}
        >
          {session.position.playing ? "暂停音乐" : "播放音乐"}
        </button>
        <button
          disabled={!ready && !session.preparing}
          onClick={() => {
            cancel();
            void session.command({ kind: "stop" });
          }}
        >
          停止音乐
        </button>
        <output>
          {audioTime(
            draft ?? session.requestedPosition ?? session.position.positionMs,
          )}{" "}
          / {audioTime(duration)}
          {session.position.loopRange && <small> · 局部循环</small>}
          {session.requestedPosition !== null && <small> · 定位中</small>}
        </output>
      </div>
      <input
        type="range"
        aria-label="舞台音乐进度"
        aria-valuetext={audioTime(
          draft ?? session.requestedPosition ?? session.position.positionMs,
        )}
        title="拖动后松手定位，方向键微调，Esc 取消"
        min={0}
        max={duration}
        step={10}
        disabled={blocked}
        value={
          draft ?? session.requestedPosition ?? session.position.positionMs
        }
        onPointerDown={(event) => {
          gesture.current = "dragging";
          event.currentTarget.setPointerCapture(event.pointerId);
        }}
        onChange={(event) => {
          const value = Number(event.target.value);
          if (gesture.current === "cancelled") return;
          if (gesture.current === "dragging") {
            pending.current = value;
            setDraft(value);
          } else void session.command({ kind: "seek", positionMs: value });
        }}
        onPointerUp={commit}
        onPointerCancel={cancel}
        onLostPointerCapture={() => {
          cancel();
          gesture.current = "idle";
        }}
        onBlur={cancel}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.preventDefault();
            cancel();
          }
        }}
      />
      {session.preparing && (
        <span role="status">
          正在准备音乐…
          <button onClick={() => void session.cancel()}>取消准备</button>
        </span>
      )}
      <AudioPerformanceControls
        track={track}
        position={session.position}
        disabled={!ready || session.preparing}
        command={session.command}
      />
      {(session.problem || session.position.problem) && (
        <span role="alert">{session.problem || session.position.problem}</span>
      )}
    </section>
  );
}
