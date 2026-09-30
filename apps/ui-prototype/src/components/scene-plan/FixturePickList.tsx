import { useEffect, useRef } from "react";
import type { ProjectView } from "../../application-host";
import { displayMeters } from "../stage/stage-display";
export function FixturePickList({
  label,
  project,
  ids,
  selected,
  busy,
  onSelect,
  onClose,
}: {
  label: string;
  project: ProjectView;
  ids: string[];
  selected: string[];
  busy: boolean;
  onSelect(id: string, additive: boolean): void;
  onClose?(): void;
}) {
  const root = useRef<HTMLElement>(null);
  useEffect(() => {
    if (onClose)
      root.current
        ?.querySelector<HTMLButtonElement>("button[aria-pressed]")
        ?.focus();
  }, [ids.join("|")]);
  return (
    <section ref={root} className="fixture-pick-list" aria-label={label}>
      <header>
        <strong>
          {label} · {ids.length}
        </strong>
        {onClose && <button onClick={onClose}>关闭候选</button>}
      </header>
      <div>
        {ids.map((id) => {
          const f = project.fixtures.find((f) => f.id === id);
          if (!f) return null;
          const p = project.stage.placements.find((p) => p.fixtureId === id);
          return (
            <button
              key={id}
              disabled={busy}
              aria-pressed={selected.includes(id)}
              onClick={(e) =>
                onSelect(id, e.shiftKey || e.metaKey || e.ctrlKey)
              }
            >
              <strong>{f.name}</strong>
              <small>
                {f.profileName}
                {p ? ` · 高度 ${displayMeters(p.positionMeters.z)} 米` : ""}
              </small>
            </button>
          );
        })}
      </div>
    </section>
  );
}
