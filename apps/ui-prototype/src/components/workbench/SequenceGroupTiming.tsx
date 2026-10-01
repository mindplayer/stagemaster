import { forwardRef, useImperativeHandle, useRef, useState } from "react";
import type { EditOperation } from "../../application-host";
import type { StepView } from "../../sequence-types";
import {
  GroupTimingError,
  groupTimingCommand,
  groupTimingDraft,
  type GroupTimingDraft,
} from "../../sequence-group-timing";
import "./sequence-group-timing.css";
export interface GroupTimingHandle {
  collect(): EditOperation[];
  accept(): void;
}
export const SequenceGroupTiming = forwardRef<
  GroupTimingHandle,
  {
    sequenceId: string;
    steps: StepView[];
    busy: boolean;
    beforeChange(): Promise<boolean>;
    onPending(value: boolean): void;
  }
>(function SequenceGroupTiming(
  { sequenceId, steps, busy, beforeChange, onPending },
  ref,
) {
  const [draft, setDraft] = useState<GroupTimingDraft | null>(null);
  const current = useRef<GroupTimingDraft | null>(null);
  const [error, setError] = useState("");
  const form = useRef<HTMLFormElement>(null);
  const details = useRef<HTMLDetailsElement>(null);
  const data = draft ?? groupTimingDraft(steps);
  function change(patch: Partial<GroupTimingDraft>) {
    const next = { ...data, ...patch };
    current.current = next;
    setDraft(next);
    setError("");
    onPending(true);
  }
  function cancel() {
    current.current = null;
    setDraft(null);
    setError("");
    onPending(false);
  }
  useImperativeHandle(ref, () => ({
    accept: cancel,
    collect() {
      if (!current.current) return [];
      try {
        const command = groupTimingCommand(sequenceId, current.current);
        return command ? [{ op: "sequence", command }] : [];
      } catch (reason) {
        setError(reason instanceof Error ? reason.message : String(reason));
        if (details.current) details.current.open = true;
        if (reason instanceof GroupTimingError)
          requestAnimationFrame(() =>
            form.current
              ?.querySelector<HTMLElement>(`[name="${reason.field}"]`)
              ?.focus(),
          );
        throw reason;
      }
    },
  }));
  return (
    <details className="sequence-group-timing" ref={details}>
      <summary>统一时间{draft ? " · 未应用" : ""}</summary>
      <form
        ref={form}
        noValidate
        onSubmit={(e) => {
          e.preventDefault();
          void beforeChange();
        }}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            e.preventDefault();
            cancel();
          }
        }}
      >
        <fieldset disabled={busy || !steps.length}>
          <p className="wb-dim">仅修改勾选的项目，其他值分别保留。</p>
          {(
            [
              ["delay", "delayEnabled", "延时"],
              ["fade", "fadeEnabled", "渐变"],
            ] as const
          ).map(([field, enabled, label]) => (
            <div key={field} className="group-timing-row">
              <label className="group-timing-toggle">
                <input
                  type="checkbox"
                  checked={data[enabled]}
                  onChange={(e) => change({ [enabled]: e.target.checked })}
                />
                {`修改${label}`}
              </label>
              <input
                name={field}
                aria-label={`批量${label}（秒）`}
                inputMode="decimal"
                disabled={!data[enabled]}
                placeholder="多个值"
                value={data[field]}
                onChange={(e) => change({ [field]: e.target.value })}
              />
              <span>秒</span>
            </div>
          ))}
          <label className="group-timing-toggle">
            <input
              type="checkbox"
              checked={data.advanceEnabled}
              onChange={(e) => change({ advanceEnabled: e.target.checked })}
            />
            修改推进方式
          </label>
          <select
            name="advance"
            aria-label="批量推进方式"
            disabled={!data.advanceEnabled}
            value={data.advance}
            onChange={(e) =>
              change({ advance: e.target.value as GroupTimingDraft["advance"] })
            }
          >
            <option value="mixed" disabled>
              多个值，请选择
            </option>
            <option value="manual">手动执行下一步</option>
            <option value="after">渐变结束后自动推进</option>
          </select>
          {data.advanceEnabled && data.advance === "after" && (
            <label>
              自动等待（秒）
              <input
                name="wait"
                aria-label="批量自动等待（秒）"
                inputMode="decimal"
                placeholder="多个值或未设置"
                value={data.wait}
                onChange={(e) => change({ wait: e.target.value })}
              />
            </label>
          )}
          {error && <p role="alert">{error}</p>}
          <div className="wb-form-actions">
            <button className="wb-primary" disabled={!draft} type="submit">
              应用时间
            </button>
            <button type="button" disabled={!draft} onClick={cancel}>
              取消时间修改
            </button>
          </div>
        </fieldset>
      </form>
    </details>
  );
});
