import { useRef, useState } from "react";
import type { PointerEvent, Dispatch } from "react";
import {
  SunIcon,
  LightbulbIcon,
  PaletteIcon,
  MusicNotesIcon,
  MagnetIcon,
  MinusIcon,
  PlusIcon,
  ArrowUUpLeftIcon,
  ArrowUUpRightIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { GROUPS, clamp, normalizeClip, formatTime } from "../demo-session";
import type { DemoState, Clip, Action, GroupId } from "../demo-session";
import { AudioReference } from "./AudioReference";
import { ClipEnvelope } from "./ClipEnvelope";

export type TransientEdit = { id: string; patch: Partial<Clip> } | null;
const groupIcons = { front: SunIcon, back: LightbulbIcon, wash: PaletteIcon };
export function Timeline({
  state,
  dispatch,
  time,
  playing,
  onSeek,
  transient,
  setTransient,
  view,
  setView,
  onNotice,
}: {
  state: DemoState;
  dispatch: Dispatch<Action>;
  time: number;
  playing: boolean;
  onSeek: (n: number) => void;
  transient: TransientEdit;
  setTransient: (p: TransientEdit) => void;
  view: "timeline" | "cues";
  setView: (v: "timeline" | "cues") => void;
  onNotice: (s: string) => void;
}) {
  const [snap, setSnap] = useState(true);
  const [zoom, setZoom] = useState(1);
  const cue = state.drafts[state.editCueId];
  const lane = useRef<HTMLDivElement>(null);
  const drag = useRef<{
    clip: Clip;
    mode: "move" | "left" | "right";
    x: number;
    width: number;
    patch: Partial<Clip>;
  } | null>(null);
  const [dragging, setDragging] = useState<string | null>(null);
  function begin(
    event: PointerEvent<HTMLDivElement>,
    clip: Clip,
    mode: "move" | "left" | "right",
  ) {
    if (event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    dispatch({ type: "selectClip", id: clip.id });
    event.currentTarget.setPointerCapture(event.pointerId);
    drag.current = {
      clip: { ...clip },
      mode,
      x: event.clientX,
      width: lane.current?.clientWidth ?? 1000,
      patch: {},
    };
    setDragging(clip.id);
  }
  function move(event: PointerEvent<HTMLDivElement>) {
    const current = drag.current;
    if (!current) return;
    const unit = snap ? 0.5 : 0.1;
    const delta =
      Math.round(
        (((event.clientX - current.x) / current.width) * cue.duration) / unit,
      ) * unit;
    let patch: Partial<Clip>;
    if (current.mode === "move") patch = { start: current.clip.start + delta };
    else if (current.mode === "right")
      patch = {
        duration: clamp(
          current.clip.duration + delta,
          0.2,
          cue.duration - current.clip.start,
        ),
      };
    else {
      const end = current.clip.start + current.clip.duration;
      const start = clamp(current.clip.start + delta, 0, end - 0.2);
      patch = { start, duration: end - start };
    }
    current.patch = normalizeClip({ ...current.clip, ...patch }, cue.duration);
    setTransient({ id: current.clip.id, patch: current.patch });
  }
  function end(cancel = false) {
    if (drag.current && !cancel)
      dispatch({
        type: "patchClip",
        id: drag.current.clip.id,
        patch: drag.current.patch,
      });
    drag.current = null;
    setDragging(null);
    setTransient(null);
  }
  function add(group: GroupId, at: number, color?: Clip["color"]) {
    dispatch({ type: "addClip", group, at, color, id: crypto.randomUUID() });
    onNotice("已添加灯光片段；可拖动两端调整长度");
  }
  return (
    <section className="timeline panel" aria-label="Cue 编排">
      <div className="timeline-toolbar">
        <strong>
          Cue {cue.id} · {cue.name}编排
        </strong>
        <div className="view-switch" role="tablist" aria-label="编排视图">
          <button
            role="tab"
            aria-selected={view === "timeline"}
            onClick={() => setView("timeline")}
          >
            时间线
          </button>
          <button
            role="tab"
            aria-selected={view === "cues"}
            onClick={() => setView("cues")}
          >
            Cue 列表
          </button>
        </div>
        <div className="timeline-tools">
          <button
            className={"tool-button " + (snap ? "active" : "")}
            aria-pressed={snap}
            onClick={() => setSnap(!snap)}
            title="吸附到 0.5 秒"
          >
            <MagnetIcon /> <span>吸附</span>
          </button>
          <i />
          <button
            className="icon-button"
            aria-label="缩小时间线"
            disabled={zoom <= 1}
            onClick={() => setZoom(Math.max(1, zoom - 0.25))}
          >
            <MinusIcon />
          </button>
          <input
            aria-label="时间线缩放"
            type="range"
            min="1"
            max="3"
            step=".25"
            value={zoom}
            onChange={(e) => setZoom(Number(e.target.value))}
          />
          <button
            className="icon-button"
            aria-label="放大时间线"
            disabled={zoom >= 3}
            onClick={() => setZoom(Math.min(3, zoom + 0.25))}
          >
            <PlusIcon />
          </button>
          <i />
          <button
            className="icon-button"
            aria-label="撤销编辑"
            title="撤销编辑 ⌘Z"
            disabled={!state.history.length}
            onClick={() => dispatch({ type: "undo" })}
          >
            <ArrowUUpLeftIcon />
          </button>
          <button
            className="icon-button"
            aria-label="重做编辑"
            title="重做编辑 ⇧⌘Z"
            disabled={!state.future.length}
            onClick={() => dispatch({ type: "redo" })}
          >
            <ArrowUUpRightIcon />
          </button>
        </div>
      </div>
      <div className="cue-list-scroll" hidden={view !== "cues"}>
        <table className="cue-list">
          <thead>
            <tr>
              <th>Cue</th>
              <th>名称</th>
              <th>状态</th>
              <th>编辑</th>
              <th>待执行</th>
            </tr>
          </thead>
          <tbody>
            {Object.values(state.saved).map((c) => (
              <tr
                key={c.id}
                className={state.editCueId === c.id ? "selected" : ""}
              >
                <td>{c.id}</td>
                <td>{c.name}</td>
                <td>
                  {state.running?.cueId === c.id ? (
                    <span className="status running">正在播放</span>
                  ) : state.nextId === c.id ? (
                    <span className="status next">下一条</span>
                  ) : (
                    "—"
                  )}
                </td>
                <td>
                  <button
                    className="text-button"
                    onClick={() => dispatch({ type: "selectCue", id: c.id })}
                  >
                    {state.editCueId === c.id ? "编辑中" : "离线编辑"}
                  </button>
                </td>
                <td>
                  <button
                    className="text-button"
                    disabled={state.nextId === c.id}
                    onClick={() => dispatch({ type: "standby", id: c.id })}
                  >
                    设为下一条
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="tracks-layout" hidden={view !== "timeline"}>
        <div className="track-headers">
          <div className="ruler-spacer" />
          {GROUPS.map((group) => {
            const Icon = groupIcons[group.id];
            return (
              <div
                className={
                  "track-header " +
                  (cue.clips.find((c) => c.id === state.selectedClipId)
                    ?.group === group.id
                    ? "selected"
                    : "")
                }
                key={group.id}
              >
                <Icon size={23} />
                <span>{group.name}</span>
                <button
                  className="icon-button add-clip"
                  aria-label={"添加" + group.name + "片段"}
                  title="添加片段"
                  onClick={() => add(group.id, time)}
                >
                  <PlusIcon />
                </button>
              </div>
            );
          })}
          <div className="track-header">
            <MusicNotesIcon size={23} />
            <span>音乐参考</span>
          </div>
        </div>
        <div className="tracks-scroll">
          <div className="track-content" style={{ width: zoom * 100 + "%" }}>
            <div
              className="ruler"
              role="slider"
              tabIndex={0}
              aria-label="时间线预览位置"
              aria-valuemin={0}
              aria-valuemax={20}
              aria-valuenow={Math.round(time * 10) / 10}
              onPointerDown={(e) => {
                const r = e.currentTarget.getBoundingClientRect();
                onSeek(clamp(((e.clientX - r.left) / r.width) * 20, 0, 20));
              }}
              onKeyDown={(e) => {
                if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
                  e.preventDefault();
                  onSeek(
                    clamp(time + (e.key === "ArrowRight" ? 0.5 : -0.5), 0, 20),
                  );
                }
              }}
            >
              {[0, 5, 10, 15, 20].map((n) => (
                <span key={n} style={{ left: (n / 20) * 100 + "%" }}>
                  {formatTime(n).slice(0, 5)}
                </span>
              ))}
            </div>
            {GROUPS.map((group, index) => (
              <div
                key={group.id}
                className="track-lane"
                ref={index === 0 ? lane : undefined}
                onDragOver={(e) => {
                  e.preventDefault();
                  e.dataTransfer.dropEffect = "copy";
                }}
                onDrop={(e) => {
                  e.preventDefault();
                  try {
                    const payload = JSON.parse(
                      e.dataTransfer.getData("application/stagemaster"),
                    );
                    if (payload.group && payload.group !== group.id) {
                      onNotice("请将灯组拖到对应轨道");
                      return;
                    }
                    const r = e.currentTarget.getBoundingClientRect();
                    add(
                      group.id,
                      ((e.clientX - r.left) / r.width) * 20,
                      payload.color,
                    );
                  } catch {
                    onNotice("请从左侧资源区拖入灯组或预设");
                  }
                }}
              >
                {[5, 10, 15].map((n) => (
                  <div
                    key={n}
                    className="time-gridline"
                    style={{ left: n * 5 + "%" }}
                  />
                ))}
                {cue.clips
                  .filter((c) => c.group === group.id)
                  .map((savedClip) => {
                    const clip =
                      transient?.id === savedClip.id
                        ? { ...savedClip, ...transient.patch }
                        : savedClip;
                    return (
                      <div
                        className={
                          "timeline-clip " +
                          (state.selectedClipId === clip.id
                            ? "selected "
                            : "") +
                          (dragging === clip.id ? "dragging" : "")
                        }
                        key={clip.id}
                        role="button"
                        tabIndex={0}
                        aria-label={
                          clip.name +
                          "，开始 " +
                          formatTime(clip.start) +
                          "，持续 " +
                          clip.duration +
                          " 秒"
                        }
                        aria-pressed={state.selectedClipId === clip.id}
                        style={{
                          left: (clip.start / 20) * 100 + "%",
                          width: (clip.duration / 20) * 100 + "%",
                          backgroundColor: group.color,
                        }}
                        onPointerDown={(e) => begin(e, savedClip, "move")}
                        onPointerMove={move}
                        onPointerUp={() => end()}
                        onPointerCancel={() => end(true)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter" || e.key === " ") {
                            e.preventDefault();
                            dispatch({ type: "selectClip", id: clip.id });
                          }
                          if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
                            e.preventDefault();
                            dispatch({
                              type: "patchClip",
                              id: clip.id,
                              patch: {
                                start:
                                  clip.start +
                                  (e.key === "ArrowRight" ? 0.5 : -0.5),
                              },
                            });
                          }
                          if (e.key === "Delete" || e.key === "Backspace") {
                            e.preventDefault();
                            dispatch({ type: "deleteClip", id: clip.id });
                          }
                        }}
                      >
                        <div
                          className="clip-handle left"
                          title="拖动起点"
                          onPointerDown={(e) => begin(e, savedClip, "left")}
                        />
                        <ClipEnvelope clip={clip} />
                        <span className="clip-name">{clip.name}</span>
                        <span className="fade-label">
                          {clip.fadeIn.toFixed(1)}s ↗ · ↘{" "}
                          {clip.fadeOut.toFixed(1)}s
                        </span>
                        <div
                          className="clip-handle right"
                          title="拖动终点"
                          onPointerDown={(e) => begin(e, savedClip, "right")}
                        />
                      </div>
                    );
                  })}
              </div>
            ))}
            <AudioReference
              time={time}
              playing={playing}
              width={zoom}
              duration={cue.duration}
              onNotice={onNotice}
            />
            <div
              className="playhead"
              style={{
                left:
                  "calc(8px + " +
                  clamp((time / 20) * 100, 0, 100) +
                  "% - " +
                  (time / 20) * 28 +
                  "px)",
              }}
            >
              <span>{formatTime(time)}</span>
            </div>
          </div>
        </div>
      </div>
      <div className="timeline-footnote">
        <span>拖动片段移动 · 拖动两端裁切 · 方向键微调</span>
        <button
          className="text-button"
          disabled={!state.selectedClipId}
          onClick={() =>
            state.selectedClipId &&
            dispatch({ type: "deleteClip", id: state.selectedClipId })
          }
        >
          <TrashIcon /> 删除片段
        </button>
      </div>
    </section>
  );
}
