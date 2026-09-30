import { useState } from "react";
import type { FixtureView } from "../../application-host";
import type { EffectChannel } from "../../effect-types";
import { reorderEffect } from "../../effect-tools";
export function EffectFixtureOrder({
  ids,
  fixtures,
  selected,
  channels,
  onChange,
}: {
  ids: string[];
  fixtures: FixtureView[];
  selected: string[];
  channels: EffectChannel[];
  onChange(ids: string[]): void;
}) {
  const [query, setQuery] = useState("");
  return (
    <details className="effect-order">
      <summary>灯具与顺序 · {ids.length} 台</summary>
      <button
        type="button"
        disabled={!selected.length}
        onClick={() => onChange([...selected])}
      >
        采用当前选灯顺序
      </button>
      <ol>
        {ids.map((id, i) => (
          <li key={id}>
            <span>
              {fixtures.find((f) => f.id === id)?.name ?? "灯具已删除"}
            </span>
            <button
              type="button"
              aria-label={`上移第 ${i + 1} 台灯具`}
              disabled={i === 0}
              onClick={() => onChange(reorderEffect(ids, i, -1))}
            >
              ↑
            </button>
            <button
              type="button"
              aria-label={`下移第 ${i + 1} 台灯具`}
              disabled={i === ids.length - 1}
              onClick={() => onChange(reorderEffect(ids, i, 1))}
            >
              ↓
            </button>
            <button
              type="button"
              aria-label={`移除${fixtures.find((f) => f.id === id)?.name ?? "灯具"}`}
              onClick={() => onChange(ids.filter((f) => f !== id))}
            >
              移除
            </button>
          </li>
        ))}
      </ol>
      <input
        aria-label="搜索待添加灯具"
        placeholder="搜索待添加灯具"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <div className="effect-add-fixtures">
        {fixtures
          .filter(
            (f) =>
              !ids.includes(f.id) &&
              f.name.toLowerCase().includes(query.trim().toLowerCase()),
          )
          .map((f) => (
            <button
              type="button"
              key={f.id}
              disabled={
                !channels.every(
                  (c) =>
                    (c.amplitudeDegrees === undefined || !!f.positioning) &&
                    f.attributes.some((a) => a.key === c.attribute),
                )
              }
              onClick={() => onChange([...ids, f.id])}
            >
              添加 {f.name}
            </button>
          ))}
      </div>
    </details>
  );
}
