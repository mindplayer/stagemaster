import { useState } from "react";
import type { ManualCapture } from "../../manual-capture-types";
import { manualReading } from "../../manual-readings";
export function ManualCaptureValues({ capture }: { capture: ManualCapture }) {
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(0);
  const rows = capture.readings.map((r) => {
    const f = capture.fixtures.find((f) => f.id === r.fixtureId);
    const a = f?.attributes.find((a) => a.key === r.attribute);
    return {
      ...r,
      change: capture.merge?.rows.find(
        (m) => m.fixtureId === r.fixtureId && m.attribute === r.attribute,
      ),
      attributeDefinition: a,
      fixture: f?.name ?? r.fixtureId,
      label: a?.label ?? r.attribute,
      reading: a ? manualReading(a, r.value)?.label : undefined,
    };
  });
  const filtered = rows.filter((r) =>
    `${r.fixture} ${r.label} ${r.reading}`
      .toLocaleLowerCase()
      .includes(search.trim().toLocaleLowerCase()),
  );
  const pages = Math.max(1, Math.ceil(filtered.length / 12));
  const current = Math.min(page, pages - 1);
  return (
    <section
      className="execution-manual-readings"
      aria-label="本次录入的冻结值"
    >
      <input
        type="search"
        aria-label="搜索待录入属性"
        placeholder="搜索灯具、属性或功能"
        value={search}
        onChange={(e) => {
          setSearch(e.target.value);
          setPage(0);
        }}
      />
      <ul>
        {filtered.slice(current * 12, (current + 1) * 12).map((r) => (
          <li key={`${r.fixtureId}:${r.attribute}`}>
            <span>
              {r.fixture}
              <small>{r.label}</small>
            </span>
            <span>
              <b>{r.reading ?? "数值无法显示"}</b>
              <small>原始值 {r.value}</small>
              {r.change && (
                <>
                  <small>
                    {
                      { added: "新增", replaced: "更新", unchanged: "相同" }[
                        r.change.change
                      ]
                    }{" "}
                    · 原：
                    {r.change.previousMode === "absent"
                      ? "未记录"
                      : r.change.previousMode === "release"
                        ? "释放"
                        : r.change.previousValue !== null &&
                            r.attributeDefinition
                          ? (manualReading(
                              r.attributeDefinition,
                              r.change.previousValue,
                            )?.label ?? "数值无法显示")
                          : "数值无法显示"}
                  </small>
                  {r.change.previousPreset && (
                    <small>脱离预设：{r.change.previousPreset}</small>
                  )}
                  {r.change.effectNames.length > 0 && (
                    <small>保留效果：{r.change.effectNames.join("、")}</small>
                  )}
                </>
              )}
            </span>
          </li>
        ))}
      </ul>
      {!filtered.length && <p>没有匹配的待录入属性</p>}
      {pages > 1 && (
        <div className="execution-buttons">
          <button disabled={current === 0} onClick={() => setPage(current - 1)}>
            上一页
          </button>
          <span>
            {current + 1} / {pages} 页 · {filtered.length} 项
          </span>
          <button
            disabled={current + 1 === pages}
            onClick={() => setPage(current + 1)}
          >
            下一页
          </button>
        </div>
      )}
    </section>
  );
}
