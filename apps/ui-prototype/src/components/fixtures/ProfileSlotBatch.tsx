import { useRef, useState } from "react";
import {
  addSlotBatch,
  planSlotBatch,
  type SlotBatchDraft,
} from "../../fixture-slot-batch";
import { FixtureFieldError } from "../../fixture-field-error";
import type { ChannelDraft } from "../../fixture-function-draft";
export function ProfileSlotBatch({
  channel,
  prefix,
  label,
  onChange,
}: {
  channel: ChannelDraft;
  prefix: string;
  label: string;
  onChange(channel: ChannelDraft): void;
}) {
  const [open, setOpen] = useState(false),
    [error, setError] = useState("");
  const [draft, setDraft] = useState<SlotBatchDraft>({
    start: "0",
    width: "1",
    count: "1",
    name: `${label}档位`,
  });
  const root = useRef<HTMLDivElement>(null);
  let preview: ReturnType<typeof planSlotBatch> | undefined;
  try {
    preview = planSlotBatch(channel, draft, prefix);
  } catch {
    /* Shown at the field on explicit insertion. */
  }
  function insert() {
    try {
      onChange(addSlotBatch(channel, draft, prefix));
      setError("");
      setOpen(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      if (e instanceof FixtureFieldError)
        root.current
          ?.querySelector<HTMLInputElement>(`[name="${e.field}"]`)
          ?.focus();
    }
  }
  return (
    <div ref={root} className="profile-slot-batch">
      <button
        type="button"
        aria-expanded={open}
        onClick={() => {
          setOpen(!open);
          setError("");
        }}
      >
        批量创建{label}档位
      </button>
      {open && (
        <div
          className="slot-batch-fields"
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              e.preventDefault();
              e.stopPropagation();
              setOpen(false);
              setError("");
              root.current?.querySelector("button")?.focus();
            }
            if (e.key === "Enter" && e.target instanceof HTMLInputElement) {
              e.preventDefault();
              e.stopPropagation();
              insert();
            }
          }}
        >
          {(
            [
              ["start", "起始值"],
              ["width", "每档宽度"],
              ["count", "档位数量"],
              ["name", "名称前缀"],
            ] as const
          ).map(([key, text]) => (
            <label key={key}>
              {text}
              <input
                type="text"
                inputMode={key === "name" ? "text" : "numeric"}
                name={`${prefix}-batch-${key}`}
                aria-label={`${label}批量${text}`}
                value={draft[key]}
                onChange={(e) => {
                  setDraft({ ...draft, [key]: e.target.value });
                  setError("");
                }}
              />
            </label>
          ))}
          {preview && (
            <div className="slot-batch-preview">
              <p>
                将创建 {preview.slots.length} 个固定档位，代表值取各档中值。
                {preview.replaceEmpty ? "替换初始空白行。" : "保留已有区间。"}
              </p>
              <ol>
                {preview.slots.map((s) => (
                  <li key={s.from}>
                    {s.name}：{s.from}–{s.to}，代表值 {s.representative}
                  </li>
                ))}
              </ol>
            </div>
          )}
          {error && (
            <p role="alert" className="wb-error">
              {error}
            </p>
          )}
          <div className="profile-actions">
            <button type="button" onClick={insert}>
              添加到草稿
            </button>
            <button
              type="button"
              onClick={() => {
                setOpen(false);
                setError("");
              }}
            >
              取消批量创建
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
