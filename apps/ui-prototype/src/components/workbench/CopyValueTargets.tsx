import { useState } from "react";
import type { CopyTargetPreview } from "../../copy-values-preview";
import { searchResources } from "../resources/resource-search";

export function CopyValueTargets({
  targets,
  selected,
  onChange,
  ready,
}: {
  ready: boolean;
  targets: CopyTargetPreview[];
  selected: string[];
  onChange(ids: string[]): void;
}) {
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(0);
  const matches = searchResources(
    targets.map(({ fixture: f, issues }) => ({
      id: f.id,
      label: f.name,
      detail: `${f.profileName} ${f.universe ?? ""}.${f.address ?? ""}`,
      keywords: issues.join(" "),
    })),
    query,
  );
  const matching = new Set(matches.map((m) => m.id));
  const selectedSet = new Set(selected);
  const shownPage = Math.min(
    page,
    Math.max(0, Math.ceil(matches.length / 50) - 1),
  );
  const byId = new Map(targets.map((t) => [t.fixture.id, t]));
  const chosen = targets.filter((t) => selectedSet.has(t.fixture.id));
  const hidden = chosen.filter((t) => !matching.has(t.fixture.id)).length;
  return (
    <section className="copy-value-targets" aria-label="复制写入目标">
      <strong>
        写入目标 · 已选 {chosen.length} / {targets.length} 台
      </strong>
      <div className="wb-resource-tools">
        <button
          type="button"
          onClick={() => onChange(targets.map((t) => t.fixture.id))}
        >
          全选目标
        </button>
        <button
          type="button"
          disabled={!ready}
          onClick={() =>
            onChange(
              targets.filter((t) => !t.issues.length).map((t) => t.fixture.id),
            )
          }
        >
          仅选可写入目标
        </button>
        <button
          type="button"
          disabled={!chosen.length}
          onClick={() => onChange([])}
        >
          清空目标
        </button>
      </div>
      <small>以上操作作用于全部目标；搜索仅筛选列表。</small>
      <input
        type="search"
        aria-label="搜索复制目标"
        placeholder="搜索名称、模式、地址或问题"
        value={query}
        onChange={(e) => {
          setQuery(e.target.value);
          setPage(0);
        }}
      />
      {hidden > 0 && (
        <p role="status">{hidden} 台已选目标被搜索隐藏，仍参与复制</p>
      )}
      <div className="copy-value-target-list">
        {matches.slice(shownPage * 50, (shownPage + 1) * 50).map(({ id }) => {
          const { fixture, issues } = byId.get(id)!;
          return (
            <label key={id} className="copy-value-target">
              <input
                type="checkbox"
                aria-label={`写入 ${fixture.name}`}
                checked={selectedSet.has(id)}
                onChange={(e) =>
                  onChange(
                    e.target.checked
                      ? [...new Set([...selected, id])]
                      : selected.filter((v) => v !== id),
                  )
                }
              />
              <span>
                <strong>{fixture.name}</strong>
                <small>
                  {fixture.profileName} · {fixture.universe ?? "—"}.
                  {fixture.address ?? "—"}
                </small>
                <small className={issues.length ? "copy-value-issue" : ""}>
                  {!ready
                    ? "请先选择有效的来源内容"
                    : issues.join("；") || "所选属性可写入"}
                </small>
              </span>
            </label>
          );
        })}
        {!matches.length && <p>没有匹配目标</p>}
      </div>
      {matches.length > 50 && (
        <div className="wb-resource-tools">
          <button
            type="button"
            disabled={!shownPage}
            onClick={() => setPage(shownPage - 1)}
          >
            上一页目标
          </button>
          <span>
            {shownPage + 1} / {Math.ceil(matches.length / 50)} 页 ·{" "}
            {matches.length} 台
          </span>
          <button
            type="button"
            disabled={(shownPage + 1) * 50 >= matches.length}
            onClick={() => setPage(shownPage + 1)}
          >
            下一页目标
          </button>
        </div>
      )}
    </section>
  );
}
