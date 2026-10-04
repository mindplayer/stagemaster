import { useEffect, useRef, useState } from "react";
import type { AudioTimeline } from "../../audio-types";
import { audioTime } from "../../audio-tools";
import type { WaveViewport } from "./waveform-data";
import type { LoopLaneSelection } from "./loop-lane-selection";
import { loopPlaysLabel } from "./performance-loop-draft";
import {
  moveLoopGroup,
  moveLoopRegion,
  type LoopMotion,
} from "./performance-loop-motion";
import { usePerformanceLoopDrag } from "./usePerformanceLoopDrag";
import "./performance-loop-lane.css";
export function PerformanceLoopLane({
  track,
  viewport,
  selection,
  selected,
  disabled,
  snap,
  lighting,
  onSelect,
  onSeek,
  onPan,
}: {
  track: AudioTimeline;
  viewport: WaveViewport;
  selection: LoopLaneSelection;
  selected: string;
  disabled: boolean;
  snap: boolean;
  lighting: boolean;
  onSelect(id: string): void;
  onSeek(time: number): void;
  onPan(pixels: number): void;
}) {
  const surface = useRef<HTMLDivElement>(null);
  const [moving, setMoving] = useState(false);
  useEffect(() => {
    setMoving(false);
  }, [selection.active]);
  const drag = usePerformanceLoopDrag(
    surface,
    track,
    viewport,
    disabled,
    snap,
    selection,
    onSelect,
    onPan,
  );
  const chosen = new Set(selection.active ? selection.ids : [selected]);
  const pixels = viewport.width / Math.max(1, viewport.end - viewport.start);
  const blocked = disabled || selection.blocked;
  function key(e: React.KeyboardEvent, id: string, mode: LoopMotion) {
    if (
      [
        "Enter",
        " ",
        "ArrowLeft",
        "ArrowRight",
        "Home",
        "End",
        "Escape",
      ].includes(e.key)
    ) {
      e.preventDefault();
      e.stopPropagation();
    }
    if (e.key === "Escape") {
      if (drag.draft) drag.cancel();
      else if (!blocked && selection.active) selection.onClear();
      return;
    }
    if (blocked || drag.draft || e.repeat) return;
    const r = track.loopRegions?.find((r) => r.id === id);
    if (!r) return;
    if (e.key === "Enter" || e.key === " ") {
      if (selection.active) selection.onPick(id, e.shiftKey);
      else onSelect(id);
    } else if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      const delta = (e.key === "ArrowLeft" ? -1 : 1) * (e.shiftKey ? 1000 : 10);
      if (selection.active) {
        if (!moving || !chosen.has(id)) return;
        const motion = moveLoopGroup(track, selection.ids, delta);
        if (!motion.problem && motion.delta)
          selection.onMove(selection.ids, motion.destination);
      } else {
        const next = moveLoopRegion(track, r, mode, delta);
        if (next.startMs !== r.startMs || next.endMs !== r.endMs)
          selection.onEdit(r, next, mode);
      }
    }
  }
  return (
    <section
      className="performance-loop-lane"
      style={{ bottom: lighting ? 84 : 0 }}
      aria-label="演出循环区段轨"
      onPointerDown={(e) => e.stopPropagation()}
      onPointerUp={(e) => e.stopPropagation()}
      onKeyDown={(e) => {
        e.stopPropagation();
      }}
    >
      <header>
        <strong>演出循环</strong>
        <span>选择／拖动不改变播放头；双击定位起点</span>
        {selection.active && (
          <button
            disabled={blocked || !selection.ids.length}
            aria-pressed={moving}
            onClick={() => setMoving((v) => !v)}
          >
            {moving ? "结束整组移动" : "整组移动工具"}
          </button>
        )}
      </header>
      <div
        ref={surface}
        className="performance-loop-track"
        onPointerMove={drag.move}
        onPointerUp={drag.end}
        onPointerCancel={drag.cancel}
        onLostPointerCapture={drag.cancel}
      >
        {(track.loopRegions ?? []).map((source) => {
          const r = drag.draft?.items.find((r) => r.id === source.id) ?? source;
          if (r.endMs <= viewport.start || r.startMs >= viewport.end)
            return null;
          const start = Math.max(viewport.start, r.startMs),
            end = Math.min(viewport.end, r.endMs);
          return (
            <div
              key={r.id}
              data-loop-region={r.id}
              className={`performance-loop-block${chosen.has(r.id) ? " selected" : ""}${!r.enabled ? " inactive" : ""}${r.locked ? " locked" : ""}${drag.draft?.problem && drag.draft.ids.includes(r.id) ? " invalid" : ""}`}
              style={{
                left: (start - viewport.start) * pixels,
                width: Math.max(2, (end - start) * pixels),
              }}
            >
              <button
                className="performance-loop-grip"
                aria-label={`循环区段 ${r.name}，${audioTime(r.startMs)} — ${audioTime(r.endMs)}`}
                aria-pressed={chosen.has(r.id)}
                disabled={blocked}
                title={`${r.name} · ${loopPlaysLabel(r.plays)}${r.locked ? " · 已锁定" : ""}`}
                onPointerDown={(e) => drag.begin(e, source, "move", moving)}
                onKeyDown={(e) => key(e, r.id, "move")}
                onClick={(e) => {
                  if (e.detail === 0) {
                    if (selection.active) selection.onPick(r.id, e.shiftKey);
                    else onSelect(r.id);
                  }
                }}
                onDoubleClick={() => {
                  if (!selection.active) onSeek(r.startMs);
                }}
              >
                <strong>{r.name}</strong>
                <small>
                  {loopPlaysLabel(r.plays)}
                  {r.locked ? " · 锁定" : ""}
                </small>
              </button>
              {!selection.active &&
                !r.locked &&
                r.startMs >= viewport.start && (
                  <button
                    className="performance-loop-boundary start"
                    aria-label={`调整 ${r.name} 开始`}
                    disabled={blocked}
                    onPointerDown={(e) => drag.begin(e, source, "start", false)}
                    onKeyDown={(e) => key(e, r.id, "start")}
                  />
                )}
              {!selection.active && !r.locked && r.endMs <= viewport.end && (
                <button
                  className="performance-loop-boundary end"
                  aria-label={`调整 ${r.name} 结束`}
                  disabled={blocked}
                  onPointerDown={(e) => drag.begin(e, source, "end", false)}
                  onKeyDown={(e) => key(e, r.id, "end")}
                />
              )}
            </div>
          );
        })}
        {drag.draft?.problem && (
          <output role="status" className="performance-loop-motion-error">
            {drag.draft.problem}
          </output>
        )}
      </div>
    </section>
  );
}
