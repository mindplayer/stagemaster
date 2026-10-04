import { useState } from "react";
import type { ManualFixture } from "../../execution-manual";
import type { ExecutionSourceState } from "../../execution-source-progress";
import { manualRows, selectedManualReading } from "../../manual-readings";
import { WheelSwatch } from "../fixtures/WheelSwatch";
import "./manual-readings.css";

export function ManualReadings({
  fixtures,
  state,
  selected,
  attribute,
  observed,
  supported,
}: {
  fixtures: ManualFixture[];
  state?: ExecutionSourceState;
  selected: string[];
  attribute?: string;
  observed: boolean;
  supported: boolean;
}) {
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(0);
  const rows = supported ? manualRows(fixtures, state) : null;
  const query = search.trim().toLocaleLowerCase();
  const filtered =
    rows?.filter((r) =>
      `${r.fixture.name} ${r.fixture.profileName} ${r.attribute.label} ${r.reading.label}`
        .toLocaleLowerCase()
        .includes(query),
    ) ?? [];
  const pages = Math.max(1, Math.ceil(filtered.length / 12));
  const current = Math.min(page, pages - 1);
  const summary =
    rows && attribute && selected.length
      ? selectedManualReading(rows, selected, attribute)
      : null;
  return (
    <section className="execution-manual-readings" aria-label="手动设定值监看">
      <strong>{observed ? "手动设定" : "最后已知手动设定"}（电平前）</strong>
      {!rows ? (
        <p>
          {supported
            ? "数值暂不可用"
            : "此后台未提供设定值，请重新载入以使用新版能力"}
        </p>
      ) : (
        <>
          {summary && (
            <p
              className="manual-selection-reading"
              title={
                summary.raw === undefined
                  ? undefined
                  : `原始值 ${summary.raw} / 65535`
              }
            >
              所选属性：<b>{summary.label}</b> · {summary.held} 台持有
              {summary.unheld > 0 ? `，${summary.unheld} 台未持有` : ""}
            </p>
          )}
          {state?.level === 0 && rows.length > 0 && (
            <p>本层电平为 0，设定值仍保留。</p>
          )}
          {!observed && <p>状态更新已暂停，以下为最后一次成功读取的数值。</p>}
          <details>
            <summary>持有属性明细（{rows.length}）</summary>
            <input
              aria-label="搜索手动持有属性"
              type="search"
              placeholder="搜索灯具、属性或功能"
              value={search}
              onChange={(e) => {
                setSearch(e.target.value);
                setPage(0);
              }}
            />
            {filtered.length ? (
              <ul>
                {filtered.slice(current * 12, (current + 1) * 12).map((r) => (
                  <li key={`${r.target.fixtureId}:${r.target.attribute}`}>
                    <span>
                      {r.fixture.name}
                      <small>{r.attribute.label}</small>
                    </span>
                    <span
                      title={`原始值 ${r.reading.raw} / 65535${r.reading.native === undefined ? "" : `；通道值 ${r.reading.native}`}`}
                    >
                      {attributeBase(r.attribute.key) === "color-wheel" &&
                        r.reading.appearance && (
                          <WheelSwatch value={r.reading.appearance} />
                        )}
                      <b>{r.reading.label}</b>
                      <small>原始值 {r.reading.raw}</small>
                    </span>
                  </li>
                ))}
              </ul>
            ) : (
              <p>{rows.length ? "没有匹配的持有属性" : "本层没有持有属性"}</p>
            )}
            {pages > 1 && (
              <div className="execution-buttons">
                <button
                  disabled={current === 0}
                  onClick={() => setPage(current - 1)}
                >
                  上一页
                </button>
                <span>
                  {current + 1} / {pages} 页 · {filtered.length} 项
                </span>
                <button
                  disabled={current + 1 >= pages}
                  onClick={() => setPage(current + 1)}
                >
                  下一页
                </button>
              </div>
            )}
          </details>
        </>
      )}
    </section>
  );
}
import { attributeBase } from "../../fixture-emitter-keys";
