import { DockPane } from "../layout/DockPane";
import { useEffect, useRef, useState, type ComponentProps } from "react";
import type { AudioEdit } from "../../audio-types";
import type { ProjectView } from "../../application-host";
import { audioTime } from "../../audio-tools";
import {
  filteredAudioMarkers,
  markerGroupCommand,
  markerSelection,
} from "../../audio-group-tools";
import { AudioMarkerList } from "./AudioMarkerList";
import "./audio-marker-batch.css";

type Props = ComponentProps<typeof AudioMarkerList> & {
  onEdit(command: AudioEdit): Promise<ProjectView | null>;
  beforeChange(): Promise<boolean>;
  batch: boolean;
  onBatch(value: boolean): void;
  workspaceVisible: boolean;
};
export function AudioMarkerLibrary(props: Props) {
  const { batch, onBatch } = props;
  return (
    <>
      <button
        disabled={props.busy}
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
}: Props) {
  const [ids, setIds] = useState<string[]>([]);
  const [destination, setDestination] = useState<string | null>(null);
  const [problem, setProblem] = useState("");
  const [removing, setRemoving] = useState(false);
  const [working, setWorking] = useState(false);
  const target = useRef<HTMLInputElement>(null);
  const deleteButton = useRef<HTMLButtonElement>(null);
  const cancelButton = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (removing && workspaceVisible) cancelButton.current?.focus();
  }, [removing, workspaceVisible]);
  function cancelRemoval() {
    setRemoving(false);
    deleteButton.current?.focus();
  }
  const visible = filteredAudioMarkers(track, scenes, query);
  const selection = markerSelection(track, ids, visible);
  const selected = new Set(selection.markers.map((m) => m.id));
  const value = destination ?? String(selection.first / 1000);
  const blocked = busy || working;
  function select(next: string[]) {
    setIds(next);
    setRemoving(false);
    setProblem("");
  }
  async function run(kind: "move" | "copy" | "remove") {
    if (blocked) return;
    setProblem("");
    setWorking(true);
    try {
      const command = markerGroupCommand(track, [...selected], kind, value);
      const previous = new Set(track.markers.map((m) => m.id));
      const next = await onEdit(command);
      if (!next?.audio) {
        setProblem("未能完成，请查看工程错误提示后调整操作");
        requestAnimationFrame(() => target.current?.focus());
        return;
      }
      if (kind === "copy")
        setIds(
          next.audio.markers
            .filter((m) => !previous.has(m.id))
            .map((m) => m.id),
        );
      if (kind === "remove") setIds([]);
      setRemoving(false);
      setDestination(null);
    } catch (error) {
      setProblem(error instanceof Error ? error.message : String(error));
      requestAnimationFrame(() => target.current?.focus());
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
        <div className="audio-batch-selection">
          <button
            disabled={blocked || !visible.length}
            onClick={() =>
              select([...new Set([...selected, ...visible.map((m) => m.id)])])
            }
          >
            选中筛选结果
          </button>
          <button
            disabled={blocked || !selected.size}
            onClick={() => select([])}
          >
            清空选择
          </button>
        </div>
        <p role="status">
          已选 {selected.size} 个
          {selection.hidden > 0 ? `，其中 ${selection.hidden} 个在筛选外` : ""}
        </p>
        <div className="audio-batch-items">
          {visible.map((marker) => (
            <label className="audio-marker-choice" key={marker.id}>
              <input
                type="checkbox"
                checked={selected.has(marker.id)}
                disabled={blocked}
                aria-label={`选择卡点：${marker.name}，${audioTime(marker.timeMs)}`}
                onChange={(e) =>
                  select(
                    e.target.checked
                      ? [...selected, marker.id]
                      : [...selected].filter((id) => id !== marker.id),
                  )
                }
              />
              <span>
                <time>{audioTime(marker.timeMs)}</time>
                <strong>{marker.name}</strong>
                <small>
                  {scenes.find((s) => s.id === marker.sceneId)?.name ??
                    "节奏标记"}
                </small>
              </span>
            </label>
          ))}
          {!visible.length && <p>未找到卡点</p>}
        </div>
      </section>
      <DockPane region="inspector" visible={workspaceVisible}>
        <section
          className="audio-marker-batch audio-batch-inspector"
          aria-label="卡点组属性"
          onKeyDown={(e) => {
            if (e.key === "Escape" && removing) {
              e.preventDefault();
              e.stopPropagation();
              cancelRemoval();
            }
          }}
        >
          <h3>卡点组属性</h3>
          <p>
            已选 {selected.size} 个
            {selection.hidden > 0
              ? `，其中 ${selection.hidden} 个在筛选外`
              : ""}
          </p>
          {!selected.size && <p>在左侧勾选需要整理的卡点。</p>}
          {selected.size > 0 && (
            <div className="audio-batch-operations">
              <p>
                原范围 {audioTime(selection.first)} —{" "}
                {audioTime(selection.last)}
              </p>
              <label>
                目标起点（秒）
                <input
                  ref={target}
                  name="groupDestination"
                  aria-label="卡点组目标起点（秒）"
                  inputMode="decimal"
                  value={value}
                  disabled={blocked}
                  onChange={(e) => {
                    setDestination(e.target.value);
                    setProblem("");
                    setRemoving(false);
                  }}
                  onKeyDown={(e) => {
                    if (e.key === "Escape") {
                      e.preventDefault();
                      e.stopPropagation();
                      setDestination(null);
                      setProblem("");
                      setRemoving(false);
                    }
                  }}
                />
              </label>
              <div className="audio-batch-selection">
                <button disabled={blocked} onClick={() => void run("move")}>
                  移动所选
                </button>
                <button disabled={blocked} onClick={() => void run("copy")}>
                  复制所选
                </button>
                <button
                  ref={deleteButton}
                  disabled={blocked}
                  onClick={() => setRemoving(true)}
                >
                  删除所选
                </button>
              </div>
              {removing && (
                <div
                  className="audio-batch-remove"
                  role="group"
                  aria-label="确认删除卡点"
                >
                  <p>
                    删除已选 {selected.size} 个卡点
                    {selection.hidden
                      ? `（含筛选外 ${selection.hidden} 个）`
                      : ""}
                    ？灯光场景保留，可以撤销。
                  </p>
                  <button disabled={blocked} onClick={() => void run("remove")}>
                    确认删除卡点
                  </button>
                  <button
                    ref={cancelButton}
                    disabled={blocked}
                    onClick={cancelRemoval}
                  >
                    取消删除
                  </button>
                </div>
              )}
              <small>
                保留相对间隔，场景引用和渐变随卡点调整；灯光区间由相邻切换点决定。
              </small>
            </div>
          )}
          {problem && (
            <p role="alert" className="audio-error">
              {problem}
            </p>
          )}
        </section>
      </DockPane>
    </>
  );
}
