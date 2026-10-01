import type { ProjectView } from "../../application-host";
import type { StepView } from "../../sequence-types";
import { seconds } from "../../sequence-tools";
export function SequenceGroupList({
  matches,
  scenes,
  ids,
  disabled,
  select,
}: {
  matches: StepView[];
  scenes: ProjectView["scenes"];
  ids: string[];
  disabled: boolean;
  select(id: string, range: boolean): void;
}) {
  return (
    <div className="wb-steps-region">
      <div
        className="sequence-group-list"
        role="group"
        aria-label="批量步骤选择"
      >
        {matches.map((s) => (
          <button
            key={s.id}
            id={`group-step-${s.id}`}
            disabled={disabled}
            role="checkbox"
            aria-checked={ids.includes(s.id)}
            aria-label={`选择步骤 ${s.number} ${s.name}`}
            onClick={(e) => select(s.id, e.shiftKey)}
            onKeyDown={(e) => {
              if (e.key === " " || e.key === "Enter") {
                e.preventDefault();
                select(s.id, e.shiftKey);
              } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
                e.preventDefault();
                const next =
                  matches[
                    matches.indexOf(s) + (e.key === "ArrowDown" ? 1 : -1)
                  ];
                if (next)
                  document.getElementById(`group-step-${next.id}`)?.focus();
              }
            }}
          >
            <span className="sequence-group-check" aria-hidden="true">
              {ids.includes(s.id) ? "✓" : ""}
            </span>
            <span className="wb-step-number">{s.number}</span>
            <span className="sequence-group-text">
              <strong>{s.name}</strong>
              <small>{scenes.find((c) => c.id === s.sceneId)?.name}</small>
              {s.script && (
                <small>
                  {[s.script.section, s.script.trigger]
                    .filter(Boolean)
                    .join(" · ") || "有排练备注"}
                </small>
              )}
            </span>
            <span className="wb-step-time">
              渐变 {seconds(s.fadeMs)} 秒
              <small>
                {s.delayMs ? `延时 ${seconds(s.delayMs)} 秒 · ` : ""}
                {s.waitMs === null
                  ? "手动推进"
                  : `等待 ${seconds(s.waitMs)} 秒后推进`}
              </small>
            </span>
          </button>
        ))}
        {!matches.length && (
          <p className="wb-dim">未找到步骤，试试其他关键词。</p>
        )}
      </div>
    </div>
  );
}
