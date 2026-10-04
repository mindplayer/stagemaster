import { useState } from "react";
import type { ManualFixture, ManualTarget } from "../../execution-manual";
export function ManualFixturePicker({
  fixtures,
  selected,
  held,
  disabled,
  onSelect,
}: {
  fixtures: ManualFixture[];
  selected: string[];
  held: ManualTarget[];
  disabled: boolean;
  onSelect(ids: string[]): void;
}) {
  const [query, setQuery] = useState("");
  const [count, setCount] = useState(30);
  const rows = fixtures.filter((f) =>
    `${f.name} ${f.profileName} ${f.universe}.${f.address}`
      .toLocaleLowerCase()
      .includes(query.trim().toLocaleLowerCase()),
  );
  const hidden = selected.filter((id) => !rows.some((f) => f.id === id)).length;
  return (
    <section className="execution-manual-fixtures" aria-label="后台灯具选择">
      <input
        aria-label="搜索后台灯具"
        placeholder="搜索灯具、模式或地址"
        value={query}
        onChange={(e) => {
          setQuery(e.target.value);
          setCount(30);
        }}
      />
      <p>
        已选 {selected.length} 台{hidden > 0 && ` · 筛选外 ${hidden} 台`}
      </p>
      <div className="execution-buttons">
        <button
          disabled={disabled || !rows.length}
          onClick={() =>
            onSelect([...new Set([...selected, ...rows.map((f) => f.id)])])
          }
        >
          选中搜索结果
        </button>
        <button
          disabled={disabled || !selected.length}
          onClick={() => onSelect([])}
        >
          清空选灯
        </button>
      </div>
      <div className="execution-manual-fixture-list">
        {rows.slice(0, count).map((f) => (
          <label key={f.id}>
            <input
              type="checkbox"
              aria-label={`手动选择${f.name}`}
              checked={selected.includes(f.id)}
              disabled={disabled}
              onChange={(e) =>
                onSelect(
                  e.target.checked
                    ? [...selected, f.id]
                    : selected.filter((id) => id !== f.id),
                )
              }
            />
            <span>
              {f.name}
              <small>
                {f.universe}.{f.address} · {f.profileName}
              </small>
            </span>
            {held.some((t) => t.fixtureId === f.id) && <strong>已持有</strong>}
          </label>
        ))}
        {!rows.length && <p>没有匹配的后台灯具</p>}
        {rows.length > count && (
          <button onClick={() => setCount((n) => n + 30)}>
            再显示 30 台（余 {rows.length - count} 台）
          </button>
        )}
      </div>
    </section>
  );
}
