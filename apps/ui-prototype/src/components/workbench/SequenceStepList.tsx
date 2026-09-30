import type { ProjectView } from "../../application-host";
import type { SequenceView } from "../../sequence-types";
import { seconds } from "../../sequence-tools";
export function SequenceStepList({
  steps,
  selectedId,
  scenes,
  busy,
  onSelect,
}: {
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
          <span className="wb-step-number">{s.number}</span>
          <div>
            <strong>{s.name}</strong>
            <small>{scenes.find((c) => c.id === s.sceneId)?.name}</small>
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
