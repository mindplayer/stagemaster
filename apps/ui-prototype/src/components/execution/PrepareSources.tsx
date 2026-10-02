import { useState } from "react";
import type { ProjectView } from "../../application-host";
import type { ExecutionSelection } from "../../execution-types";

export function PrepareSources({
  project,
  disabled,
  onPrepare,
}: {
  project: ProjectView;
  disabled: boolean;
  onPrepare(selection: ExecutionSelection[]): void;
}) {
  const [selected, setSelected] = useState<string[]>([]);
  const [query, setQuery] = useState("");
  const choices = [
    ...project.sequences.map((s) => ({
      ...s,
      kind: "sequence" as const,
      label: "列表",
    })),
    ...project.scenes.map((s) => ({
      ...s,
      kind: "scene" as const,
      label: "场景",
    })),
  ];
  const valid = choices.filter((s) => selected.includes(`${s.kind}:${s.id}`));
  const visible = choices.filter((s) =>
    s.name.toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()),
  );
  return (
    <section className="execution-prepare" aria-label="载入后台节目">
      <header>
        <h3>载入后台节目</h3>
        <span>已选 {valid.length} / 63</span>
      </header>
      <input
        aria-label="搜索待载入节目"
        placeholder="搜索场景或场景列表"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <div className="execution-choices">
        {visible.map((s) => {
          const key = `${s.kind}:${s.id}`;
          return (
            <label key={key}>
              <input
                type="checkbox"
                checked={selected.includes(key)}
                disabled={
                  disabled || (!selected.includes(key) && valid.length >= 63)
                }
                onChange={(e) =>
                  setSelected((old) =>
                    e.target.checked
                      ? [...old, key]
                      : old.filter((k) => k !== key),
                  )
                }
              />
              <span>{s.name}</span>
              <small>{s.label}</small>
            </label>
          );
        })}
        {!visible.length && <p>没有匹配的节目</p>}
      </div>
      <footer>
        <button
          disabled={disabled || !valid.length}
          onClick={() => setSelected([])}
        >
          清空选择
        </button>
        <button
          className="wb-primary"
          disabled={disabled || !valid.length}
          onClick={() => onPrepare(valid.map(({ kind, id }) => ({ kind, id })))}
        >
          载入所选节目
        </button>
      </footer>
    </section>
  );
}
