import { forwardRef, useImperativeHandle, useRef, useState } from "react";
import type { StepScript, StepView } from "../../sequence-types";
import { scriptFields } from "../../sequence-script-tools";
import {
  groupScriptDraft,
  groupScriptCommand,
  GroupScriptError,
  type GroupScriptDraft,
  type GroupScriptField,
  type ScriptChangeMode,
} from "../../sequence-group-script";
import type { GroupTimingHandle } from "./SequenceGroupTiming";
import "./sequence-group-script.css";

export const SequenceGroupScript = forwardRef<
  GroupTimingHandle,
  {
    sequenceId: string;
    steps: StepView[];
    busy: boolean;
    beforeChange(): Promise<boolean>;
    onPending(value: boolean): void;
  }
>(function SequenceGroupScript(
  { sequenceId, steps, busy, beforeChange, onPending },
  ref,
) {
  const [draft, setDraft] = useState<GroupScriptDraft | null>(null);
  const current = useRef<GroupScriptDraft | null>(null);
  const [error, setError] = useState("");
  const form = useRef<HTMLFormElement>(null),
    details = useRef<HTMLDetailsElement>(null);
  const data = draft ?? groupScriptDraft(steps);
  function change(key: keyof StepScript, patch: Partial<GroupScriptField>) {
    const next = {
      ...data,
      fields: { ...data.fields, [key]: { ...data.fields[key], ...patch } },
    };
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
        const command = groupScriptCommand(sequenceId, current.current);
        return command ? [{ op: "sequence", command }] : [];
      } catch (reason) {
        setError(reason instanceof Error ? reason.message : String(reason));
        if (details.current) details.current.open = true;
        if (reason instanceof GroupScriptError)
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
    <details ref={details} className="sequence-group-script">
      <summary>统一剧本提示{draft ? " · 未应用" : ""}</summary>
      <form
        aria-label="批量剧本提示编辑"
        ref={form}
        noValidate
        onSubmit={(event) => {
          event.preventDefault();
          void beforeChange();
        }}
        onKeyDown={(event) => {
          if (event.key === "Escape" && !busy) {
            event.preventDefault();
            event.stopPropagation();
            cancel();
          }
        }}
      >
        <fieldset disabled={busy || !steps.length}>
          {scriptFields.map(([, key, label, limit]) => (
            <div className="group-script-field" key={key}>
              <label>
                {label}
                <select
                  aria-label={`批量${label}修改方式`}
                  value={data.fields[key].mode}
                  onChange={(event) =>
                    change(key, {
                      mode: event.target.value as ScriptChangeMode,
                    })
                  }
                >
                  <option value="keep">
                    保留原值{data.fields[key].mixed ? " · 多个值" : ""}
                  </option>
                  <option value="replace">统一填写</option>
                  <option value="clear">清空</option>
                </select>
              </label>
              {data.fields[key].mode !== "clear" &&
                (key === "section" ? (
                  <input
                    name={key}
                    aria-label={`批量${label}`}
                    maxLength={limit * 2}
                    disabled={data.fields[key].mode !== "replace"}
                    placeholder={data.fields[key].mixed ? "多个值" : "未填写"}
                    value={data.fields[key].value}
                    onChange={(event) =>
                      change(key, { value: event.target.value })
                    }
                  />
                ) : (
                  <textarea
                    name={key}
                    aria-label={`批量${label}`}
                    rows={key === "trigger" ? 3 : 4}
                    maxLength={limit * 2}
                    disabled={data.fields[key].mode !== "replace"}
                    placeholder={data.fields[key].mixed ? "多个值" : "未填写"}
                    value={data.fields[key].value}
                    onChange={(event) =>
                      change(key, { value: event.target.value })
                    }
                  />
                ))}
              {data.fields[key].mode === "clear" && (
                <small>
                  将清空所选 {data.ids.length} 个步骤的{label}。
                </small>
              )}
            </div>
          ))}
          {error && <p role="alert">{error}</p>}
          <div className="wb-form-actions">
            <button className="wb-primary" type="submit" disabled={!draft}>
              应用批量修改
            </button>
            <button type="button" disabled={!draft} onClick={cancel}>
              取消提示修改
            </button>
          </div>
        </fieldset>
      </form>
    </details>
  );
});
