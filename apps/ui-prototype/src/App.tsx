import { useCallback, useEffect, useReducer, useRef, useState } from "react";
import type { DragEvent } from "react";
import {
  MagnifyingGlassIcon,
  ArrowsOutCardinalIcon,
  PlayIcon,
  ArrowRightIcon,
  CircleIcon,
  CheckIcon,
  XIcon,
  PlusIcon,
  InfoIcon,
} from "@phosphor-icons/react";
import {
  demoReducer,
  createDemoState,
  PALETTE,
  GROUPS,
  hasChanges,
  formatTime,
  clamp,
} from "./demo-session";
import type { Clip, GroupId } from "./demo-session";
import { StagePreview } from "./components/StagePreview";
import { Timeline } from "./components/Timeline";
import type { TransientEdit } from "./components/Timeline";

function NumberField({
  label,
  value,
  min = 0,
  max,
  step = 0.1,
  suffix = "秒",
  onCommit,
}: {
  label: string;
  value: number;
  min?: number;
  max: number;
  step?: number;
  suffix?: string;
  onCommit: (n: number) => void;
}) {
  const [draft, setDraft] = useState(String(value));
  const skipCommit = useRef(false);
  useEffect(() => setDraft(String(value)), [value]);
  function commit() {
    if (skipCommit.current) {
      skipCommit.current = false;
      return;
    }
    const n = Number(draft);
    if (draft.trim() && Number.isFinite(n)) {
      const checked = clamp(n, min, max);
      setDraft(String(checked));
      onCommit(checked);
    } else setDraft(String(value));
  }
  return (
    <label className="number-field">
      <span>{label}</span>
      <div>
        <input
          aria-label={label}
          type="number"
          min={min}
          max={max}
          step={step}
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === "Enter") e.currentTarget.blur();
            if (e.key === "Escape") {
              skipCommit.current = true;
              setDraft(String(value));
              e.currentTarget.blur();
            }
          }}
        />
        <span>{suffix}</span>
      </div>
    </label>
  );
}

export function App() {
  const [state, dispatch] = useReducer(demoReducer, undefined, createDemoState);
  const [time, setTime] = useState(8.4);
  const [playing, setPlaying] = useState(false);
  const [transient, setTransient] = useState<TransientEdit>(null);
  const [view, setView] = useState<"timeline" | "cues">("timeline");
  const [workspace, setWorkspace] = useState<"编排" | "演出">("编排");
  const [resource, setResource] = useState<"灯组" | "预设" | "效果">("灯组");
  const [search, setSearch] = useState("");
  const [notice, setNotice] = useState("");
  const [savedAt, setSavedAt] = useState<number | null>(null);
  const dialog = useRef<HTMLDialogElement>(null);
  const [dialogContent, setDialogContent] = useState<"patch" | "about">(
    "about",
  );
  const cue = state.drafts[state.editCueId];
  const selected = cue.clips.find((c) => c.id === state.selectedClipId);
  const clip =
    selected && transient?.id === selected.id
      ? { ...selected, ...transient.patch }
      : selected;
  const group = GROUPS.find((g) => g.id === clip?.group);
  const color = PALETTE.find((p) => p.id === clip?.color) ?? PALETTE[2];
  const dirty = hasChanges(state) || transient !== null;
  const onNotice = useCallback((s: string) => setNotice(s), []);
  useEffect(() => {
    if (notice) {
      const timer = setTimeout(() => setNotice(""), 3500);
      return () => clearTimeout(timer);
    }
  }, [notice]);
  useEffect(() => {
    setTime(8.4);
    setPlaying(false);
    setTransient(null);
    setSavedAt(null);
  }, [state.editCueId]);
  useEffect(() => {
    if (!playing) return;
    let previous = performance.now();
    let frame = 0;
    const tick = (now: number) => {
      const delta = (now - previous) / 1000;
      previous = now;
      setTime((t) => Math.min(cue.duration, t + delta));
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [playing, cue.duration]);
  useEffect(() => {
    if (time >= cue.duration && playing) setPlaying(false);
  }, [time, cue.duration, playing]);
  function seek(n: number) {
    setTime(clamp(n, 0, cue.duration));
  }
  function togglePreview() {
    if (time >= cue.duration) setTime(0);
    setPlaying((p) => !p);
  }
  function patch(values: Partial<Clip>) {
    if (clip) dispatch({ type: "patchClip", id: clip.id, patch: values });
  }
  function commitTransient() {
    if (transient)
      dispatch({ type: "patchClip", id: transient.id, patch: transient.patch });
    setTransient(null);
  }
  const save = useCallback(() => {
    dispatch({ type: "save" });
    setSavedAt(state.editCueId);
    onNotice(
      "场景 " + state.editCueId + " 已保存到本次演示；现场模拟播放保持不变",
    );
  }, [state.editCueId, onNotice]);
  useEffect(() => {
    const handle = (e: KeyboardEvent) => {
      if (dialog.current?.open) return;
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") {
        e.preventDefault();
        save();
        return;
      }
      const target = e.target as HTMLElement;
      if (
        ["INPUT", "TEXTAREA", "SELECT", "BUTTON"].includes(target.tagName) ||
        target.isContentEditable
      )
        return;
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "z") {
        e.preventDefault();
        dispatch({ type: e.shiftKey ? "redo" : "undo" });
      }
      if (e.code === "Space" && target.getAttribute("role") !== "button") {
        e.preventDefault();
        setPlaying((p) => !p);
      }
    };
    window.addEventListener("keydown", handle);
    return () => window.removeEventListener("keydown", handle);
  }, [save]);
  function selectGroup(id: GroupId) {
    const target =
      cue.clips.find(
        (c) => c.group === id && time >= c.start && time < c.start + c.duration,
      ) ?? cue.clips.find((c) => c.group === id);
    if (target) {
      dispatch({ type: "selectClip", id: target.id });
      if (!(time >= target.start && time < target.start + target.duration))
        setTime(target.start + target.duration / 2);
    } else onNotice("该灯组还没有片段，可拖到下方轨道或点击轨道的加号");
  }
  function dragResource(e: DragEvent, payload: object) {
    e.dataTransfer.setData("application/stagemaster", JSON.stringify(payload));
    e.dataTransfer.effectAllowed = "copy";
  }
  return (
    <div className="app-shell">
      <header className="app-header">
        <div className="brand">舞台大师</div>
        <span className="project-name">夏夜现场</span>
        <nav aria-label="工作区">
          <button
            onClick={() => {
              setDialogContent("patch");
              dialog.current?.showModal();
            }}
          >
            配适
          </button>
          {(["编排", "演出"] as const).map((label) => (
            <button
              key={label}
              className={workspace === label ? "active" : ""}
              aria-current={workspace === label ? "page" : undefined}
              onClick={() => {
                setWorkspace(label);
                setView(label === "演出" ? "cues" : "timeline");
              }}
            >
              {label}
            </button>
          ))}
        </nav>
        <button
          className="prototype-label"
          onClick={() => {
            setDialogContent("about");
            dialog.current?.showModal();
          }}
        >
          <InfoIcon /> 交互原型 · 模拟数据
        </button>
      </header>
      <main className="editor-workspace">
        <aside className="library panel">
          <h2>资源</h2>
          <div className="resource-tabs" role="tablist" aria-label="资源类型">
            {(["灯组", "预设", "效果"] as const).map((tab) => (
              <button
                key={tab}
                role="tab"
                aria-selected={resource === tab}
                onClick={() => {
                  setResource(tab);
                  setSearch("");
                }}
              >
                {tab}
              </button>
            ))}
          </div>
          <label className="search">
            <MagnifyingGlassIcon />
            <input
              placeholder={"搜索" + resource}
              aria-label={"搜索" + resource}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            {search && (
              <button
                className="icon-button"
                aria-label="清空搜索"
                onClick={() => setSearch("")}
              >
                <XIcon size={14} />
              </button>
            )}
          </label>
          {resource === "灯组" && (
            <div className="group-list">
              {GROUPS.filter((g) => g.name.includes(search)).map((g) => (
                <button
                  className={group?.id === g.id ? "selected" : ""}
                  key={g.id}
                  aria-pressed={group?.id === g.id}
                  draggable
                  onDragStart={(e) => dragResource(e, { group: g.id })}
                  onClick={() => selectGroup(g.id)}
                >
                  <strong>{g.name}</strong>
                  <span>8台</span>
                </button>
              ))}
              {!GROUPS.some((g) => g.name.includes(search)) && (
                <p className="empty-state">没有找到灯组</p>
              )}
            </div>
          )}
          {resource === "预设" && (
            <div className="preset-library">
              {PALETTE.filter((p) => p.name.includes(search)).map((p) => (
                <button
                  key={p.id}
                  draggable
                  onDragStart={(e) => dragResource(e, { color: p.id })}
                  onClick={() =>
                    clip
                      ? patch({ color: p.id })
                      : onNotice("请先选择一个灯光片段")
                  }
                >
                  <span style={{ backgroundColor: p.hex }} />
                  {p.name}
                </button>
              ))}
              {!PALETTE.some((p) => p.name.includes(search)) && (
                <p className="empty-state">没有找到预设</p>
              )}
            </div>
          )}
          {resource === "效果" && (
            <div className="effect-list">
              {["缓慢渐入", "快速亮起", "柔和淡出"]
                .filter((s) => s.includes(search))
                .map((name) => (
                  <button
                    key={name}
                    onClick={() => {
                      if (!clip) {
                        onNotice("请先选择一个灯光片段");
                        return;
                      }
                      patch({
                        fadeIn:
                          name === "缓慢渐入"
                            ? Math.min(3, clip.duration / 2)
                            : 0.2,
                        fadeOut:
                          name === "柔和淡出"
                            ? Math.min(3, clip.duration / 2)
                            : 0.2,
                      });
                      onNotice("已应用" + name + "，可在右侧精调");
                    }}
                  >
                    <span>{name}</span>
                    <PlusIcon />
                  </button>
                ))}
              <p className="muted library-note">效果应用于选中片段</p>
            </div>
          )}
          {resource === "灯组" && (
            <div className="quick-presets">
              <h3>常用预设</h3>
              <div>
                {[PALETTE[0], PALETTE[2], PALETTE[1]].map((p) => (
                  <button
                    key={p.id}
                    title={"应用" + p.name}
                    onClick={() =>
                      clip ? patch({ color: p.id }) : onNotice("请先选择片段")
                    }
                    draggable
                    onDragStart={(e) => dragResource(e, { color: p.id })}
                  >
                    <span style={{ backgroundColor: p.hex }} />
                    {p.name}
                  </button>
                ))}
              </div>
            </div>
          )}
          <div className="drag-hint">
            <ArrowsOutCardinalIcon size={23} />
            <span>拖入轨道，开始编排</span>
          </div>
        </aside>
        <StagePreview
          cue={cue}
          clip={clip}
          time={time}
          playing={playing}
          onSeek={seek}
          onPlay={togglePreview}
        />
        <aside className="inspector panel" aria-label="片段属性">
          <div className="inspector-title">
            <h2>{group ? group.name + " · 8 台" : "片段属性"}</h2>
            <p>
              {clip ? "选中片段：" + clip.name : "选择轨道上的片段开始编辑"}
            </p>
          </div>
          {clip ? (
            <div className="inspector-content">
              <div className="inspector-fields">
                <label className="field-label" htmlFor="intensity">
                  亮度（强度）
                </label>
                <div className="intensity-control">
                  <input
                    id="intensity"
                    type="range"
                    min={0}
                    max={100}
                    step={1}
                    aria-label="亮度滑块"
                    value={clip.intensity}
                    style={{ backgroundSize: clip.intensity + "% 100%" }}
                    onChange={(e) =>
                      setTransient({
                        id: clip.id,
                        patch: { intensity: Number(e.target.value) },
                      })
                    }
                    onPointerUp={commitTransient}
                    onKeyUp={commitTransient}
                    onBlur={commitTransient}
                    onPointerCancel={() => setTransient(null)}
                  />
                  <NumberField
                    label="亮度"
                    value={clip.intensity}
                    max={100}
                    step={1}
                    suffix="%"
                    onCommit={(n) => patch({ intensity: n })}
                  />
                </div>
                <label className="color-field">
                  <span>颜色</span>
                  <div>
                    <span
                      className="color-chip"
                      style={{ backgroundColor: color.hex }}
                    />
                    <select
                      aria-label="灯光颜色"
                      value={clip.color}
                      onChange={(e) =>
                        patch({ color: e.target.value as Clip["color"] })
                      }
                    >
                      {PALETTE.map((p) => (
                        <option key={p.id} value={p.id}>
                          {p.name}
                        </option>
                      ))}
                    </select>
                  </div>
                </label>
                <div className="field-pair">
                  <NumberField
                    label="开始时间"
                    value={clip.start}
                    max={cue.duration - clip.duration}
                    onCommit={(n) => patch({ start: n })}
                  />
                  <NumberField
                    label="持续时间"
                    value={clip.duration}
                    min={0.2}
                    max={cue.duration - clip.start}
                    onCommit={(n) => patch({ duration: n })}
                  />
                </div>
                <div className="field-pair">
                  <NumberField
                    label="淡入"
                    value={clip.fadeIn}
                    max={clip.duration - clip.fadeOut}
                    onCommit={(n) => patch({ fadeIn: n })}
                  />
                  <NumberField
                    label="淡出"
                    value={clip.fadeOut}
                    max={clip.duration - clip.fadeIn}
                    onCommit={(n) => patch({ fadeOut: n })}
                  />
                </div>
              </div>
              <div className="save-area">
                <button className="primary-button" onClick={save}>
                  {savedAt === cue.id && !dirty ? (
                    <>
                      <CheckIcon /> 已保存场景 {cue.id}
                    </>
                  ) : (
                    "保存到场景 " + cue.id
                  )}
                </button>
                <span>仅保存，不触发播放</span>
                <span className={"dirty-indicator " + (dirty ? "dirty" : "")}>
                  {dirty ? "有未保存修改" : "本次演示内保存 · 刷新后重置"}
                </span>
              </div>
            </div>
          ) : (
            <div className="inspector-empty">
              <PlusIcon size={28} />
              <p>
                从左侧拖入灯组，
                <br />
                或点击轨道旁的加号。
              </p>
            </div>
          )}
        </aside>
      </main>
      <Timeline
        state={state}
        dispatch={dispatch}
        time={time}
        playing={playing}
        onSeek={seek}
        transient={transient}
        onScrubStart={() => setPlaying(false)}
        setTransient={setTransient}
        view={view}
        setView={setView}
        onNotice={onNotice}
      />
      <footer className="live-bar" aria-label="独立现场模拟播放">
        <div className="live-label">
          <CircleIcon weight="fill" size={11} />
          <span>现场播放 · 模拟</span>
        </div>
        <div className="live-current">
          <PlayIcon weight="fill" />
          <span>正在播放</span>
          <strong data-testid="running-cue">
            {state.running?.cueId} <span>{state.running?.snapshot.name}</span>
          </strong>
        </div>
        <div className="live-next">
          <ArrowRightIcon />
          <span>下一条</span>
          <select
            aria-label="下次执行的场景"
            value={state.nextId ?? ""}
            onChange={(e) =>
              dispatch({ type: "standby", id: Number(e.target.value) })
            }
          >
            <option value="" disabled>
              序列结束
            </option>
            {Object.values(state.saved).map((c) => (
              <option key={c.id} value={c.id}>
                {c.id} {c.name}
              </option>
            ))}
          </select>
        </div>
        <button
          className="go-button"
          disabled={state.nextId === null}
          onClick={() => {
            dispatch({ type: "go" });
            onNotice(
              state.nextId === null
                ? "序列已结束"
                : "模拟播放已切换到场景 " + state.nextId + "；离线编辑不受影响",
            );
          }}
        >
          {state.nextId === null ? "序列结束" : "执行 → " + state.nextId}
        </button>
      </footer>
      <div
        className={"toast " + (notice ? "visible" : "")}
        role="status"
        aria-live="polite"
      >
        <CheckIcon />
        {notice}
      </div>
      <dialog ref={dialog} className="info-dialog">
        <button
          className="icon-button dialog-close"
          aria-label="关闭说明"
          onClick={() => dialog.current?.close()}
        >
          <XIcon />
        </button>
        {dialogContent === "patch" ? (
          <>
            <h2>本次演示的灯具</h2>
            <p>24 台示例灯具，仅用于界面体验，未连接任何设备。</p>
            <table>
              <thead>
                <tr>
                  <th>灯组</th>
                  <th>数量</th>
                  <th>标识</th>
                </tr>
              </thead>
              <tbody>
                {GROUPS.map((g) => (
                  <tr key={g.id}>
                    <td>{g.name}</td>
                    <td>8 台</td>
                    <td>
                      {g.prefix}01 — {g.prefix}08
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            <p className="muted">灯具档案与地址配适将在后续产品流程中接入。</p>
          </>
        ) : (
          <>
            <h2>舞台画布 · 交互原型</h2>
            <p>
              你可以拖动灯光片段、修改颜色与强度、调整渐变，并在独立预览中播放。
            </p>
            <p>
              底部“执行”只切换模拟播放。草稿、保存后的场景
              和模拟播放快照互相独立；刷新页面会恢复示例。
            </p>
            <p>
              舞台使用生成插图示意当前选中片段，不代表真实混光或光学预演。音乐参考只在本机读取和监听。
            </p>
            <p className="muted">
              空格：预览播放／暂停 · ⌘S：保存 · ⌘Z：撤销编辑
            </p>
          </>
        )}
        <button
          className="primary-button"
          onClick={() => dialog.current?.close()}
        >
          知道了
        </button>
      </dialog>
    </div>
  );
}
