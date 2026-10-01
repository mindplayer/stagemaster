import type { ClipSelection } from "./clip-selection";
import { useEffect, useState } from "react";
import type {
  AudioEdit,
  AudioLightingClip,
  AudioTimeline,
} from "../../audio-types";
import type { ProjectView } from "../../application-host";
import { audioTime } from "../../audio-tools";
import { DockPane } from "../layout/DockPane";
import { AudioClipGroupInspector } from "./AudioClipGroupInspector";
import {
  clipGroupCommand,
  clipGroupSelection,
  type ClipGroupOperation,
} from "./clip-group-tools";
import "./audio-marker-batch.css";
export function AudioClipBatch({
  track,
  items,
  busy,
  visible,
  onEdit,
  selectionState,
}: {
  selectionState: ClipSelection;
  track: AudioTimeline;
  items: AudioLightingClip[];
  busy: boolean;
  visible: boolean;
  onEdit(command: AudioEdit): Promise<ProjectView | null>;
}) {
  const { ids, replace: setIds } = selectionState;
  const [destination, setDestination] = useState<string | null>(null);
  const [problem, setProblem] = useState(""),
    [removing, setRemoving] = useState(false),
    [working, setWorking] = useState(false);
  const selection = clipGroupSelection(track.lightingClips ?? [], ids, items);
  const selected = selection.items.map((c) => c.id),
    blocked = busy || working;
  const value = destination ?? (selection.first / 1000).toFixed(3);
  function reset() {
    setDestination(null);
    setProblem("");
    setRemoving(false);
  }
  useEffect(reset, [ids.join("\0")]);
  function select(next: string[]) {
    setIds(next);
    reset();
  }
  function toggle(id: string, range: boolean) {
    selectionState.toggle(id, items, range);
    reset();
  }
  async function run(kind: ClipGroupOperation) {
    if (blocked) return;
    setProblem("");
    setWorking(true);
    try {
      const command = clipGroupCommand(track, selected, kind, value);
      const previous = new Set(track.lightingClips?.map((c) => c.id));
      const next = await onEdit(command);
      if (!next?.audio?.lightingClips)
        throw new Error("未能完成，请查看工程错误提示后调整操作");
      if (kind === "copy")
        setIds(
          next.audio.lightingClips
            .filter((c) => !previous.has(c.id))
            .map((c) => c.id),
        );
      if (kind === "remove") setIds([]);
      reset();
    } catch (error) {
      setProblem(error instanceof Error ? error.message : String(error));
    } finally {
      setWorking(false);
    }
  }
  return (
    <>
      <div className="audio-batch-selection">
        <button
          disabled={blocked || !items.length}
          onClick={() =>
            select([...new Set([...selected, ...items.map((c) => c.id)])])
          }
        >
          选中筛选片段
        </button>
        <button
          disabled={blocked || !selected.length}
          onClick={() => {
            select([]);
          }}
        >
          清空片段选择
        </button>
      </div>
      <small role="status">
        已选 {selected.length} 个
        {selection.hidden ? `，其中 ${selection.hidden} 个在筛选外` : ""}
      </small>
      <div className="audio-clip-list" role="group" aria-label="批量片段选择">
        {items.map((c) => (
          <button
            key={c.id}
            id={`batch-clip-${c.id}`}
            role="checkbox"
            aria-checked={selected.includes(c.id)}
            aria-label={`选择片段：${c.name}，${audioTime(c.startMs)}${c.enabled === false ? "，已停用" : ""}`}
            className={`${selected.includes(c.id) ? "selected" : ""} ${c.enabled === false ? "inactive" : ""}`}
            disabled={blocked}
            onClick={(e) => toggle(c.id, e.shiftKey)}
            onKeyDown={(e) => {
              if (e.key === " " || e.key === "Enter") {
                e.preventDefault();
                toggle(c.id, e.shiftKey);
              }
              if (e.key === "ArrowUp" || e.key === "ArrowDown") {
                e.preventDefault();
                const next =
                  items[items.indexOf(c) + (e.key === "ArrowDown" ? 1 : -1)];
                if (next)
                  document.getElementById(`batch-clip-${next.id}`)?.focus();
              }
            }}
          >
            <time>
              {audioTime(c.startMs)} — {audioTime(c.endMs)}
            </time>
            <strong>
              {selected.includes(c.id) ? "✓ " : ""}
              {c.name}
              {c.enabled === false ? " · 已停用" : ""}
              {c.locked ? " · 已锁定" : ""}
            </strong>
          </button>
        ))}
        {!items.length && <small>未找到片段</small>}
      </div>
      <DockPane region="inspector" visible={visible}>
        <AudioClipGroupInspector
          selection={selection}
          value={value}
          problem={problem}
          removing={removing}
          blocked={blocked}
          visible={visible}
          onValue={(v) => {
            setDestination(v);
            setProblem("");
            setRemoving(false);
          }}
          onReset={reset}
          onRemove={setRemoving}
          onRun={(kind) => void run(kind)}
        />
      </DockPane>
    </>
  );
}
