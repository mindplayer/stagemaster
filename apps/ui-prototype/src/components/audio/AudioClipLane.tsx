import { useLightingClipDrag } from "./useLightingClipDrag";
import { useClipMarquee } from "./useClipMarquee";
import { clipsInRange, type ClipLaneSelection } from "./clip-selection";
import { useRef, type RefObject } from "react";
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
  clipSelection,
}: {
  clipSelection?: ClipLaneSelection;
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
  const surface = useRef<HTMLDivElement>(null);
  const group = !!clipSelection?.active;
  const moving = useLightingClipDrag(
    surface,
    track,
    viewport,
    disabled || group,
    snap,
    onSelect,
    onMove,
  );
  const marquee = useClipMarquee(
    surface,
    viewport,
    clipSelection,
    disabled,
    track,
  );
  const pixels = viewport.width / Math.max(1, viewport.end - viewport.start);
  const clips = (track.lightingClips ?? []).map((c) =>
    moving.draft?.clip.id === c.id ? moving.draft.next : c,
  );
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
          e.preventDefault();
          if (!marquee.draft && group && !disabled) clipSelection?.onClear();
          cancel();
        }
      }}
    >
      <header>
        <strong>灯光片段</strong>
        <span>{clips.length} 段</span>
        {clipSelection && (
          <button
            type="button"
            aria-label="时间线片段多选"
            aria-pressed={group}
            disabled={disabled}
            onClick={clipSelection.onMode}
          >
            多选{group ? ` · ${clipSelection.ids.length}` : ""}
          </button>
        )}
        <span>
          {group
            ? "单击增减 · Shift 连选／追加框选 · Esc 取消"
            : "拖动移动 · 两端调整长度 · 空隙为默认值"}
        </span>
      </header>
      <div
        className="audio-lighting-clips"
        ref={surface}
        data-selecting={group}
        onPointerDown={(e) => {
          if (group) marquee.begin(e);
        }}
        onPointerMove={group ? marquee.move : moving.move}
        onPointerUp={group ? marquee.end : moving.end}
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
              if (group) marquee.begin(e, c.id);
              else moving.begin(e, c, mode);
            };
            const label =
              scenes.find((s) => s.id === c.sceneId)?.name ?? c.name;
            return (
              <div
                key={c.id}
                className={`audio-lighting-clip ${chosen(c.id) ? "selected" : ""} ${c.locked ? "locked" : ""} ${c.enabled === false ? "inactive" : ""}`}
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
