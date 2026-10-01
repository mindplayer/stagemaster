import { useEffect, useRef, useState, type RefObject } from "react";
import type { ProjectView } from "../../application-host";
import type {
  SequenceEdit,
  SequenceView,
  StepGroupOperation,
} from "../../sequence-types";
import { stepMatches } from "../../sequence-script-tools";
import {
  addedStepIds,
  orderedStepIds,
  toggleStepRange,
} from "../../sequence-group-tools";
import { DockPane } from "../layout/DockPane";
import { ResourcePicker } from "../resources/ResourcePicker";
import { DeleteDialog } from "./DeleteDialog";
import {
  SequenceGroupTiming,
  type GroupTimingHandle,
} from "./SequenceGroupTiming";
import { SequenceGroupList } from "./SequenceGroupList";
import "./sequence-groups.css";

const END = "__end__";
export function SequenceGroupEditor({
  sequence,
  scenes,
  busy,
  visible,
  query,
  setQuery,
  onEdit,
  timingRef,
  beforeChange,
  onPending,
}: {
  timingRef: RefObject<GroupTimingHandle | null>;
  beforeChange(): Promise<boolean>;
  onPending(value: boolean): void;
  sequence: SequenceView;
  scenes: ProjectView["scenes"];
  busy: boolean;
  visible: boolean;
  query: string;
  setQuery(value: string): void;
  onEdit(command: SequenceEdit): Promise<ProjectView | null>;
}) {
  const [selected, setSelected] = useState<string[]>([]);
  const anchor = useRef<string | null>(null);
  const pending = useRef(false);
  const deleteButton = useRef<HTMLButtonElement>(null);
  const searchInput = useRef<HTMLInputElement>(null);
  const [working, setWorking] = useState(false);
  const [destination, setDestination] = useState(END);
  const [confirm, setConfirm] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  useEffect(() => {
    setSelected((current) => {
      const next = orderedStepIds(sequence.steps, current);
      return next.length === current.length ? current : next;
    });
  }, [sequence.steps]);
  const ids = orderedStepIds(sequence.steps, selected);
  const matches = sequence.steps.filter((s) =>
    stepMatches(
      s,
      scenes.find((scene) => scene.id === s.sceneId)?.name ?? "",
      query,
    ),
  );
  const hidden = ids.filter((id) => !matches.some((s) => s.id === id)).length;
  const disabled = busy || working;
  const options = [
    { id: END, label: "列表末尾", detail: "放在所有步骤之后" },
    ...sequence.steps
      .filter((s) => !ids.includes(s.id))
      .map((s) => ({
        id: s.id,
        label: `${s.number} · ${s.name} 之前`,
        detail: scenes.find((scene) => scene.id === s.sceneId)?.name,
        keywords: [s.script?.section, s.script?.trigger, s.script?.notes]
          .filter(Boolean)
          .join(" "),
      })),
  ];
  const validDestination = options.some((o) => o.id === destination);
  async function select(id: string, range: boolean) {
    if (!(await beforeChange())) return;
    const previousAnchor = anchor.current;
    setSelected((current) =>
      toggleStepRange(
        orderedStepIds(sequence.steps, current),
        matches,
        id,
        previousAnchor,
        range,
      ),
    );
    anchor.current = id;
    setNotice("");
    setError("");
  }
  async function apply(
    operation: Exclude<StepGroupOperation, { kind: "timing" }>,
  ) {
    if (pending.current || disabled || !ids.length) return;
    pending.current = true;
    setWorking(true);
    setError("");
    setNotice("");
    try {
      const next = await onEdit({
        kind: "editSteps",
        id: sequence.id,
        stepIds: ids,
        operation,
      });
      if (!next) {
        setError("未能整理步骤，请查看错误提示后重试");
        return;
      }
      const steps =
        next.sequences.find((s) => s.id === sequence.id)?.steps ?? [];
      setSelected(
        operation.kind === "copy"
          ? addedStepIds(sequence.steps, steps)
          : operation.kind === "remove"
            ? []
            : ids,
      );
      setConfirm(false);
      if (operation.kind === "remove")
        requestAnimationFrame(() => searchInput.current?.focus());
      setNotice(
        `${operation.kind === "copy" ? "已复制" : operation.kind === "move" ? "已移动" : "已删除"} ${ids.length} 个步骤`,
      );
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      pending.current = false;
      setWorking(false);
    }
  }
  return (
    <>
      <div className="sequence-group-toolbar">
        <input
          ref={searchInput}
          aria-label="搜索批量步骤"
          placeholder="搜索步骤、幕场或台词"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <button
          disabled={disabled || !matches.length}
          onClick={() =>
            void beforeChange().then((ok) => {
              if (!ok) return;
              setSelected((current) => [
                ...new Set([
                  ...orderedStepIds(sequence.steps, current),
                  ...matches.map((s) => s.id),
                ]),
              ]);
              setNotice("");
            })
          }
        >
          选择筛选结果
        </button>
        <button
          disabled={disabled || !ids.length}
          onClick={() =>
            void beforeChange().then((ok) => {
              if (!ok) return;
              setSelected([]);
              anchor.current = null;
              setNotice("");
            })
          }
        >
          清除选择
        </button>
        <span>
          {ids.length} 已选{hidden ? ` · ${hidden} 项不在筛选结果中` : ""}
        </span>
        {query && <button onClick={() => setQuery("")}>清除筛选</button>}
      </div>
      <SequenceGroupList
        matches={matches}
        scenes={scenes}
        ids={ids}
        disabled={disabled}
        select={select}
      />
      <DockPane region="inspector" visible={visible}>
        <aside className="wb-properties sequence-group-properties">
          <span className="wb-eyebrow">批量整理</span>
          <h2>已选 {ids.length} 个步骤</h2>
          <p className="wb-dim">按原执行顺序整理。按住 Shift 选择连续范围。</p>
          {hidden > 0 && (
            <p>{hidden} 个所选步骤已被筛选隐藏，仍参与本次操作。</p>
          )}
          <ResourcePicker
            label="插入位置"
            placeholder="选择位置"
            value={destination}
            options={options}
            disabled={disabled || !ids.length}
            onSelect={(value) => {
              setDestination(value);
              setError("");
            }}
          />
          {!validDestination && (
            <p role="alert">目标步骤已移除或被选中，请重新选择位置。</p>
          )}
          <div className="sequence-group-actions">
            <button
              disabled={
                disabled ||
                !ids.length ||
                !validDestination ||
                sequence.steps.length + ids.length > 1024
              }
              onClick={() =>
                void apply({
                  kind: "copy",
                  beforeId: destination === END ? null : destination,
                })
              }
            >
              复制到此处
            </button>
            <button
              disabled={disabled || !ids.length || !validDestination}
              onClick={() =>
                void apply({
                  kind: "move",
                  beforeId: destination === END ? null : destination,
                })
              }
            >
              移动到此处
            </button>
            <button
              ref={deleteButton}
              className="wb-danger"
              disabled={
                disabled || !ids.length || ids.length === sequence.steps.length
              }
              onClick={() => {
                setError("");
                setConfirm(true);
              }}
            >
              删除所选步骤
            </button>
          </div>
          {ids.length > 0 && ids.length === sequence.steps.length && (
            <p className="wb-dim">列表至少保留一个步骤。</p>
          )}
          {sequence.steps.length + ids.length > 1024 && (
            <p className="wb-dim">
              复制后会超过 1024 步，请减少选择或拆分列表。
            </p>
          )}
          <p className="wb-dim">
            复制保留场景引用、时间和剧本提示，生成独立步骤与新编号。
          </p>
          {sequence.tracking === "inherited" && (
            <p className="wb-dim">
              当前为继承模式，调整步骤顺序或数量可能改变后续灯光。整理后请重新预演。
            </p>
          )}
          {error && !confirm && <p role="alert">{error}</p>}
          <p role="status">{notice}</p>
          <SequenceGroupTiming
            ref={timingRef}
            sequenceId={sequence.id}
            steps={sequence.steps.filter((s) => ids.includes(s.id))}
            busy={disabled}
            beforeChange={beforeChange}
            onPending={onPending}
          />
        </aside>
      </DockPane>
      {confirm && (
        <DeleteDialog
          name={`${ids.length} 个步骤`}
          description="仅删除所选步骤，保留场景资源；可以撤销。"
          busy={disabled}
          error={error}
          onCancel={() => {
            setConfirm(false);
            requestAnimationFrame(() => deleteButton.current?.focus());
          }}
          onDelete={() => void apply({ kind: "remove" })}
        />
      )}
    </>
  );
}
