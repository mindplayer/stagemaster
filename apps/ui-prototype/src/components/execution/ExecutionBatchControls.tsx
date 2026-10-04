import { useLayoutEffect, useRef } from "react";
import type {
  ExecutionBatchRequest,
  ExecutionStatus,
  ExecutionView,
} from "../../execution-types";
import {
  batchNames,
  batchReady,
  batchEffect,
  ordinaryProgram,
} from "../../execution-batch";
import { sourceStates } from "../../execution-board";
import { useExecutionBatch } from "./useExecutionBatch";
import "./execution-batch.css";

export function ExecutionBatchControls({
  runtime,
  selected,
  visibleIds,
  disabled,
  active,
  observed,
  filterKey,
  onSelectVisible,
  onClear,
  onBatch,
}: {
  runtime: ExecutionView;
  selected: readonly string[];
  visibleIds: readonly string[];
  disabled: boolean;
  active: boolean;
  observed: boolean;
  filterKey: string;
  onSelectVisible(): void;
  onClear(): void;
  onBatch(request: ExecutionBatchRequest): Promise<ExecutionStatus | undefined>;
}) {
  const available = active && observed && !disabled && batchReady(runtime);
  const triggers = useRef<
    Partial<Record<keyof typeof batchNames, HTMLButtonElement | null>>
  >({});
  const batch = useExecutionBatch(
    runtime,
    selected,
    available,
    filterKey,
    onBatch,
    (kind) => triggers.current[kind]?.focus(),
  );
  const cancelButton = useRef<HTMLButtonElement>(null);
  useLayoutEffect(() => {
    if (batch.review) cancelButton.current?.focus();
  }, [batch.review]);
  function cancel() {
    const kind = batch.review?.request.action.kind;
    batch.cancel();
    if (kind) triggers.current[kind]?.focus();
  }
  const hidden = selected.filter((id) => !visibleIds.includes(id)).length;
  const visible = runtime.catalog.sources.filter(
    (s) => visibleIds.includes(s.id) && ordinaryProgram(s),
  );
  return (
    <section className="execution-batch" aria-label="所选节目批量操作">
      <p>
        已选 {selected.length} 个普通节目
        {hidden > 0 && ` · 筛选外 ${hidden} 个（仍包含在操作中）`}
      </p>
      <div className="execution-buttons">
        <button disabled={!active || !visible.length} onClick={onSelectVisible}>
          全选当前结果
        </button>
        <button disabled={!active || !selected.length} onClick={onClear}>
          清空选择
        </button>
        {(Object.keys(batchNames) as (keyof typeof batchNames)[]).map(
          (kind) => (
            <button
              key={kind}
              ref={(node) => {
                triggers.current[kind] = node;
              }}
              disabled={
                !available ||
                !selected.length ||
                !batchEffect(runtime, selected, kind)
              }
              onClick={() => batch.begin(kind)}
            >
              {batchNames[kind]}
            </button>
          ),
        )}
      </div>
      {!runtime.catalog.capabilities?.includes("sourceBatch") && (
        <p>此后台未提供节目批量控制；单节目操作仍可使用。</p>
      )}
      {batch.problem && <p role="alert">{batch.problem}</p>}
      {batch.receipt && <p role="status">{batch.receipt}</p>}
      {batch.review && (
        <div
          role="alertdialog"
          aria-label="审阅所选节目操作"
          className="execution-confirm"
          tabIndex={-1}
        >
          <h3>
            {batchNames[batch.review.request.action.kind]} ·{" "}
            {batch.review.members.length} 个节目
          </h3>
          <p>
            {batch.review.request.action.kind === "stop"
              ? "停止将归还这些节目的贡献；其他节目可能重新显示。音乐和手动层不释放，这不是熄灯或急停。"
              : "只改变符合此操作的节目状态；待执行节目不会启动，暂停保持当前贡献。"}
          </p>
          {hidden > 0 && <p>包含 {hidden} 个筛选外节目，请核对完整名单。</p>}
          <ul>
            {batch.review.members.map((m) => (
              <li key={m.id}>
                {m.number} · {m.name} ·{" "}
                {sourceStates[m.status as keyof typeof sourceStates] ??
                  "状态未知"}
              </li>
            ))}
          </ul>
          <div className="execution-buttons">
            <button disabled={!available} onClick={batch.confirm}>
              确认{batchNames[batch.review.request.action.kind]}
            </button>
            <button ref={cancelButton} onClick={cancel}>
              取消批量操作
            </button>
          </div>
        </div>
      )}
    </section>
  );
}
