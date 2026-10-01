import "./sequence-script.css";
import type { ExecutionPosition } from "./execution-position";
import type { ProjectView } from "../../application-host";
import type { SequenceView } from "../../sequence-types";
import { seconds } from "../../sequence-tools";
export function SequenceStepList({
  steps,
  selectedId,
  scenes,
  busy,
  onSelect,
  position,
}: {
  position?: ExecutionPosition;
  steps: SequenceView["steps"];
  selectedId: string;
  scenes: ProjectView["scenes"];
  busy: boolean;
  onSelect(id: string): Promise<boolean>;
}) {
  return (
    <div className="wb-steps" role="group" aria-label="列表步骤">
      {steps.map((s) => (
        <button
          aria-pressed={s.id === selectedId}
          key={s.id}
          disabled={busy}
          className={s.id === selectedId ? "active" : ""}
          data-running={position?.currentId === s.id}
          data-next={position?.nextId === s.id && position?.currentId !== s.id}
          onClick={() => void onSelect(s.id)}
          onKeyDown={(e) => {
            if (e.key === "ArrowDown" || e.key === "ArrowUp") {
              e.preventDefault();
              const next =
                steps[steps.indexOf(s) + (e.key === "ArrowDown" ? 1 : -1)];
              if (next)
                void onSelect(next.id).then(
                  (ok) =>
                    ok && document.getElementById(`step-${next.id}`)?.focus(),
                );
            }
          }}
          id={`step-${s.id}`}
        >
          {position && (
            <span className="execution-row-state">
              {position.currentId === s.id
                ? position.status === "paused"
                  ? "已暂停"
                  : position.status === "finished"
                    ? "已结束"
                    : "当前"
                : position.nextId === s.id
                  ? "下一步"
                  : ""}
            </span>
          )}
          <span className="wb-step-number">{s.number}</span>
          <div>
            <strong>{s.name}</strong>
            <small>{scenes.find((c) => c.id === s.sceneId)?.name}</small>
            {s.script && (
              <small
                className="sequence-step-script"
                title={[s.script.section, s.script.trigger]
                  .filter(Boolean)
                  .join(" · ")}
              >
                {[s.script.section, s.script.trigger]
                  .filter(Boolean)
                  .join(" · ") || (s.script.notes ? "有排练备注" : "")}
              </small>
            )}
          </div>
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
      {!steps.length && <p className="wb-dim">未找到步骤</p>}
    </div>
  );
}
