import { TrashIcon } from "@phosphor-icons/react";
import type { SequenceView } from "../../sequence-types";
import type { ExecutionPosition } from "./execution-position";
export function SequenceWorkspaceHeading({
  sequence,
  sequences,
  execution,
  batch,
  busy,
  position,
  onExecution,
  onBatch,
  onChoose,
  onDelete,
}: {
  sequence?: SequenceView;
  sequences: SequenceView[];
  execution: boolean;
  batch: boolean;
  busy: boolean;
  position: ExecutionPosition;
  onExecution(value: boolean): void;
  onBatch(): void;
  onChoose(id: string): void;
  onDelete(): void;
}) {
  return (
    <>
      <div className="wb-content-heading">
        <div>
          <span className="wb-eyebrow">节目编排</span>
          <h1>{sequence?.name ?? "场景列表"}</h1>
        </div>
        <div
          className="execution-view-switch"
          role="group"
          aria-label="列表工作方式"
        >
          <button
            disabled={busy}
            aria-pressed={!execution}
            onClick={() => onExecution(false)}
          >
            步骤编排
          </button>
          <button
            disabled={busy}
            aria-pressed={execution}
            onClick={() => onExecution(true)}
          >
            执行视图
          </button>
        </div>
        {sequence && !execution && (
          <>
            <button disabled={busy} aria-pressed={batch} onClick={onBatch}>
              {batch ? "返回单步编辑" : "批量整理"}
            </button>
            <button aria-label="删除列表" disabled={busy} onClick={onDelete}>
              <TrashIcon />
            </button>
          </>
        )}
      </div>
      {execution && (
        <div className="execution-list-picker">
          <label htmlFor="execution-list">
            {position.stale && position.sequenceId === sequence?.id
              ? "编排列表（待载入）"
              : "执行列表"}
          </label>
          <select
            id="execution-list"
            disabled={busy}
            value={sequence?.id ?? ""}
            onChange={(e) => onChoose(e.target.value)}
          >
            {sequences.map((s) => (
              <option key={s.id} value={s.id}>
                {s.name}
              </option>
            ))}
          </select>
          <span className="wb-dim">{sequence?.steps.length ?? 0} 步</span>
        </div>
      )}
    </>
  );
}
