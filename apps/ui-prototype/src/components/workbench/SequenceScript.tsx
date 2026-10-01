import type { StepScript } from "../../sequence-types";
import type { SequenceDraft } from "../../sequence-tools";
import { scriptFields } from "../../sequence-script-tools";
import "./sequence-script.css";

export function SequenceScriptEditor({
  data,
  change,
}: {
  data: SequenceDraft;
  change(patch: Partial<SequenceDraft>): void;
}) {
  return (
    <section className="sequence-script-editor" aria-label="剧本提示编辑">
      <h2>剧本提示</h2>
      {scriptFields.map(([field, , label, limit]) => (
        <label key={field}>
          {label}
          {field === "scriptSection" ? (
            <input
              name={field}
              value={data[field]}
              onChange={(e) => change({ [field]: e.target.value })}
            />
          ) : (
            <textarea
              name={field}
              rows={field === "scriptTrigger" ? 3 : 4}
              value={data[field]}
              onChange={(e) => change({ [field]: e.target.value })}
            />
          )}
          <small>
            {Array.from(data[field]).length} / {limit}
          </small>
        </label>
      ))}
    </section>
  );
}

/** Loaded metadata for the live run; never substitute the edited document here. */
export function SequenceScriptPrompt({ script }: { script?: StepScript }) {
  if (
    !script ||
    ![script.section, script.trigger, script.notes].some((v) => v.trim())
  )
    return null;
  return (
    <div className="sequence-script-prompt">
      {script.section && (
        <span className="sequence-script-section">{script.section}</span>
      )}
      {script.trigger && (
        <p className="sequence-script-trigger" tabIndex={0}>
          {script.trigger}
        </p>
      )}
      {script.notes && (
        <details>
          <summary>排练备注</summary>
          <p tabIndex={0}>{script.notes}</p>
        </details>
      )}
    </div>
  );
}
