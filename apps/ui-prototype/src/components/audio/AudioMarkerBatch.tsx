import { DockPane } from "../layout/DockPane";
import { useEffect, useState, type ComponentProps } from "react";
import type { AudioEdit, AudioMarker } from "../../audio-types";
import type { ProjectView } from "../../application-host";
import { audioTime } from "../../audio-tools";
import {
  filteredAudioMarkers,
  markerGroupCommand,
  markerSelection,
} from "../../audio-group-tools";
import type { OrderedSelection } from "../selection/ordered-selection";
import { AudioMarkerList } from "./AudioMarkerList";
import { AudioMarkerGroupInspector } from "./AudioMarkerGroupInspector";
import "./audio-marker-batch.css";
type Props = ComponentProps<typeof AudioMarkerList> & {
  onEdit(command: AudioEdit): Promise<ProjectView | null>;
  beforeChange(): Promise<boolean>;
  batch: boolean;
  onBatch(value: boolean): void;
  workspaceVisible: boolean;
  selectionState: OrderedSelection<AudioMarker>;
  pending: boolean;
  onPending(pending: boolean): void;
};
export function AudioMarkerLibrary(props: Props) {
  const { batch, onBatch } = props;
  return (
    <>
      <button
        disabled={props.busy || props.pending}
        aria-pressed={batch}
        onClick={async () => {
          if (await props.beforeChange()) onBatch(!batch);
        }}
      >
        {batch ? "返回单点编辑" : "批量整理卡点"}
      </button>
      {batch ? <AudioMarkerBatch {...props} /> : <AudioMarkerList {...props} />}
    </>
  );
}
function AudioMarkerBatch({
  track,
  scenes,
  busy,
  query,
  onQuery,
  onEdit,
  workspaceVisible,
  selectionState,
  onPending,
}: Props) {
  const { ids, replace: setIds } = selectionState;
  const [destination, setDestination] = useState<string | null>(null);
  const [problem, setProblem] = useState("");
  const [removing, setRemoving] = useState(false);
  const [working, setWorking] = useState(false);
  const visible = filteredAudioMarkers(track, scenes, query);
  const selection = markerSelection(track, ids, visible);
  const selected = new Set(selection.markers.map((m) => m.id));
  const value = destination ?? (selection.first / 1000).toFixed(3);
  const blocked = busy || working;
  const pending = destination !== null || removing;
  useEffect(() => {
    onPending(pending || working);
  }, [pending, working, onPending]);
  useEffect(() => () => onPending(false), [onPending]);
  function reset() {
    setDestination(null);
    setProblem("");
    setRemoving(false);
  }
  useEffect(reset, [ids.join("\0")]);
  async function run(kind: "move" | "copy" | "remove") {
    if (blocked) return;
    setProblem("");
    setWorking(true);
    try {
      const command = markerGroupCommand(track, [...selected], kind, value);
      const previous = new Set(track.markers.map((m) => m.id));
      const next = await onEdit(command);
      if (!next?.audio)
        throw new Error("未能完成，请查看工程错误提示后调整操作");
      if (kind === "copy")
        setIds(
          next.audio.markers
            .filter((m) => !previous.has(m.id))
            .map((m) => m.id),
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
      <section className="audio-marker-batch" aria-label="卡点批量整理">
        <label>
          搜索卡点
          <input
            aria-label="批量搜索卡点"
            placeholder="名称或灯光场景"
            value={query}
            disabled={blocked}
            onChange={(e) => onQuery(e.target.value)}
          />
        </label>
        {query && (
          <button disabled={blocked} onClick={() => onQuery("")}>
            清除卡点筛选
          </button>
        )}
        <div className="audio-batch-selection">
          <button
            disabled={blocked || pending || !visible.length}
            onClick={() =>
              setIds([...new Set([...selected, ...visible.map((m) => m.id)])])
            }
          >
            选中筛选结果
          </button>
          <button
            disabled={blocked || pending || !selected.size}
            onClick={() => setIds([])}
          >
            清空选择
          </button>
        </div>
        <p role="status">
          已选 {selected.size} 个
          {selection.hidden > 0 ? `，其中 ${selection.hidden} 个在筛选外` : ""}
        </p>
        {pending && <p>请先完成或取消右侧目标输入／删除确认，再调整选择。</p>}
        <div className="audio-batch-items">
          {visible.map((marker) => (
            <button
              className="audio-marker-choice"
              key={marker.id}
              id={`batch-marker-${marker.id}`}
              role="checkbox"
              aria-checked={selected.has(marker.id)}
              disabled={blocked || pending}
              aria-label={`选择卡点：${marker.name}，${audioTime(marker.timeMs)}`}
              onClick={(e) =>
                selectionState.toggle(marker.id, visible, e.shiftKey)
              }
              onKeyDown={(e) => {
                if (e.key === " " || e.key === "Enter") {
                  e.preventDefault();
                  if (!e.repeat)
                    selectionState.toggle(marker.id, visible, e.shiftKey);
                }
                if (e.key === "ArrowUp" || e.key === "ArrowDown") {
                  e.preventDefault();
                  const next =
                    visible[
                      visible.indexOf(marker) + (e.key === "ArrowDown" ? 1 : -1)
                    ];
                  if (next)
                    document.getElementById(`batch-marker-${next.id}`)?.focus();
                }
              }}
            >
              <span aria-hidden="true">
                {selected.has(marker.id) ? "✓" : "○"}
              </span>
              <span>
                <time>{audioTime(marker.timeMs)}</time>
                <strong>{marker.name}</strong>
                <small>
                  {scenes.find((s) => s.id === marker.sceneId)?.name ??
                    "节奏标记"}
                </small>
              </span>
            </button>
          ))}
          {!visible.length && <p>未找到卡点</p>}
        </div>
      </section>
      <DockPane region="inspector" visible={workspaceVisible}>
        <AudioMarkerGroupInspector
          selection={selection}
          value={value}
          blocked={blocked}
          removing={removing}
          problem={problem}
          pending={destination !== null}
          visible={workspaceVisible}
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
