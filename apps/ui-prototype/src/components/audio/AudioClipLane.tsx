import { trimmedEffectOffset } from "./clip-trim-tools";
import { useClipGroupDrag } from "./useClipGroupDrag";
import { AudioClipLaneHeader } from "./AudioClipLaneHeader";
import { useLightingClipDrag } from "./useLightingClipDrag";
import { useClipMarquee } from "./useClipMarquee";
import { clipsInRange, type ClipLaneSelection } from "./clip-selection";
import { useEffect, useRef, useState, type RefObject } from "react";
import type { AudioLightingClip, AudioTimeline } from "../../audio-types";
import type { SceneView } from "../../application-host";
import type { WaveViewport } from "./waveform-data";
import { audioTime } from "../../audio-tools";
import { moveLightingClip, type ClipMotion } from "./clip-motion";
import "./lighting-lane.css";
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
  onPan,
  clipSelection,
}: {
  clipSelection?: ClipLaneSelection;
  onPan?(pixels: number): void;
  track: AudioTimeline;
  scenes: SceneView[];
  viewport: WaveViewport;
  selected: string;
  disabled: boolean;
  snap: boolean;
  cursor: RefObject<HTMLDivElement | null>;
  onSelect(id: string): void;
  onSeek(time: number): void;
  onMove(clip: AudioLightingClip, mode: ClipMotion): void;
}) {
  const surface = useRef<HTMLDivElement>(null);
  const group = !!clipSelection?.active;
  const [moveTool, setMoveTool] = useState(false);
  useEffect(() => {
    if (!group) setMoveTool(false);
  }, [group]);
  const movingGroup = useClipGroupDrag(
    surface,
    track,
    viewport,
    clipSelection,
    disabled || !group || !moveTool || !!clipSelection?.movementBlocked,
    snap,
    onPan,
  );
  const moving = useLightingClipDrag(
    surface,
    track,
    viewport,
    disabled || group,
    snap,
    onSelect,
    onMove,
    onPan,
  );
  const marquee = useClipMarquee(
    surface,
    viewport,
    clipSelection,
    disabled || moveTool,
    track,
    track.outMs - track.inMs,
    onPan,
  );
  const pixels = viewport.width / Math.max(1, viewport.end - viewport.start);
  const blockedMotion = group ? clipSelection?.movementBlocked : "";
  const proposed = movingGroup.draft?.moved ? movingGroup.draft.next : null;
  const replacements = new Map(proposed?.clips.map((c) => [c.id, c]));
  const clips = (track.lightingClips ?? [])
    .map(
      (c) =>
        replacements.get(c.id) ??
        (moving.draft?.clip.id === c.id ? moving.draft.next : c),
    )
    .sort((a, b) => a.startMs - b.startMs);
  const movingClip = moving.draft?.moved ? moving.draft.next : null;
  const singleStatus = movingClip
    ? `${movingClip.name} · ${audioTime(movingClip.startMs)} — ${audioTime(movingClip.endMs)} · ${moving.draft?.mode === "start" ? `效果起点 ${(trimmedEffectOffset(moving.draft.clip, movingClip.startMs) / 1000).toFixed(3)} 秒 · ` : ""}松手应用`
    : "";
  const box = marquee.draft?.moved ? marquee.draft : null;
  const selectionIds = box
    ? [
        ...(box.append ? (clipSelection?.ids ?? []) : []),
        ...clipsInRange(clips, box.start, box.end),
      ]
    : (clipSelection?.ids ?? []);
  const chosen = (id: string) =>
    group ? selectionIds.includes(id) : selected === id;
  function cancel() {
    movingGroup.cancel();
    moving.cancel();
    marquee.cancel();
  }
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
    if (disabled) return;
    if (group) {
      if (moveTool && (e.key === "ArrowLeft" || e.key === "ArrowRight")) {
        e.preventDefault();
        e.stopPropagation();
        if (clipSelection?.ids.includes(c.id))
          movingGroup.nudge(
            (e.key === "ArrowLeft" ? -1 : 1) * (e.shiftKey ? 1000 : 10),
          );
        return;
      }
      if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
        e.preventDefault();
        e.stopPropagation();
        const rendered = clips.filter((c) => visible(c.startMs, c.endMs));
        const next =
          rendered[
            rendered.findIndex((item) => item.id === c.id) +
              (e.key === "ArrowLeft" ? -1 : 1)
          ];
        if (next) {
          surface.current
            ?.querySelector<HTMLButtonElement>(
              `[data-lighting-segment="${next.id}"]`,
            )
            ?.focus();
          if (e.shiftKey) clipSelection?.onPick(next.id, true);
        }
      }
      return;
    }
    if (c.locked) return;
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      e.preventDefault();
      e.stopPropagation();
      const next = moveLightingClip(
        track,
        c,
        mode,
        (e.key === "ArrowLeft" ? -1 : 1) * (e.shiftKey ? 1000 : 10),
      );
      if (next.startMs !== c.startMs || next.endMs !== c.endMs)
        onMove(next, mode);
    }
  }
  return (
    <div
      className="audio-lighting-lane"
      role="group"
      aria-label="独立灯光片段"
      onPointerDown={(e) => e.stopPropagation()}
      onKeyDown={(e) => {
        // Clip navigation must never fall through to the parent music seek handler.
        if (["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)) {
          e.preventDefault();
          e.stopPropagation();
        }
        if (e.key === "Escape") {
          e.stopPropagation();
          e.preventDefault();
          if (!marquee.draft && !movingGroup.draft && group && !disabled)
            clipSelection?.onClear();
          cancel();
        }
      }}
    >
      <AudioClipLaneHeader
        count={clips.length}
        selection={clipSelection}
        disabled={disabled}
        moving={moveTool}
        onTool={(value) => {
          cancel();
          setMoveTool(value);
        }}
      />
      {(blockedMotion || proposed || movingGroup.problem || singleStatus) && (
        <div
          className="audio-clip-motion-status"
          data-error={!!(proposed?.problem || movingGroup.problem)}
          role="status"
          aria-live={proposed || movingClip ? "off" : "polite"}
        >
          {blockedMotion ||
            proposed?.problem ||
            movingGroup.problem ||
            singleStatus ||
            `移动 ${proposed!.ids.length} 段 · 起点 ${audioTime(proposed!.destination)} · 松手应用`}
        </div>
      )}
      <div
        className="audio-lighting-clips"
        ref={surface}
        data-selecting={group}
        data-moving={group && moveTool}
        onPointerDown={(e) => {
          if (group && !moveTool) marquee.begin(e);
        }}
        onPointerMove={
          group ? (moveTool ? movingGroup.move : marquee.move) : moving.move
        }
        onPointerUp={
          group ? (moveTool ? movingGroup.end : marquee.end) : moving.end
        }
        onPointerCancel={cancel}
        onLostPointerCapture={() => {
          movingGroup.captureLost();
          moving.cancel();
          marquee.cancel();
        }}
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
              if (group && moveTool) movingGroup.begin(e, c.id);
              else if (group) marquee.begin(e, c.id);
              else moving.begin(e, c, mode);
            };
            const label =
              scenes.find((s) => s.id === c.sceneId)?.name ?? c.name;
            return (
              <div
                key={c.id}
                className={`audio-lighting-clip ${chosen(c.id) ? "selected" : ""} ${c.locked ? "locked" : ""} ${c.enabled === false ? "inactive" : ""}`}
                data-invalid={!!proposed?.problem && replacements.has(c.id)}
                style={style(c.startMs, c.endMs)}
              >
                <button
                  data-lighting-segment={c.id}
                  className="audio-lighting-select"
                  aria-pressed={chosen(c.id)}
                  aria-label={`${c.name}，${audioTime(c.startMs)} — ${audioTime(c.endMs)}${c.locked ? "，已锁定" : ""}${c.enabled === false ? "，已停用，灯具默认值" : ""}`}
                  disabled={disabled}
                  onPointerDown={(e) => begin(e, "move")}
                  onClick={(e) => {
                    e.currentTarget.focus();
                    if (group) {
                      if (e.detail === 0)
                        clipSelection?.onPick(c.id, e.shiftKey);
                    } else onSelect(c.id);
                  }}
                  onDoubleClick={() => {
                    if (!group) onSeek(c.startMs);
                  }}
                  onKeyDown={(e) => key(e, c, "move")}
                >
                  <strong>
                    {c.enabled === false ? "停用 · " : ""}
                    {c.locked ? "锁定 · " : ""}
                    {c.name}
                  </strong>
                  <small>{c.enabled === false ? "灯具默认值" : label}</small>
                </button>
                {c.enabled !== false &&
                  c.fadeMs > 0 &&
                  c.startMs + c.fadeMs > viewport.start && (
                    <span
                      className="audio-lighting-fade"
                      aria-hidden="true"
                      style={{
                        width: Math.max(
                          0,
                          (Math.min(
                            c.endMs,
                            c.startMs + c.fadeMs,
                            viewport.end,
                          ) -
                            Math.max(c.startMs, viewport.start)) *
                            pixels,
                        ),
                      }}
                    />
                  )}
                {!group &&
                  (["start", "end"] as const).map(
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
        {box && (
          <div
            className="audio-clip-marquee"
            aria-hidden="true"
            style={style(
              Math.min(box.start, box.end),
              Math.max(box.start, box.end),
            )}
          />
        )}
        <div
          className="audio-lighting-cursor"
          ref={cursor}
          aria-hidden="true"
        />
      </div>
    </div>
  );
}
