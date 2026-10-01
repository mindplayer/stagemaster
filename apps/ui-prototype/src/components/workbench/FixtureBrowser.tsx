import {
  LightbulbIcon,
  MagnifyingGlassIcon,
  XIcon,
} from "@phosphor-icons/react";
import type { FixtureView, SceneView } from "../../application-host";
import { fixtureMatches, selectRange } from "../../editor-tools";
import { useRef } from "react";
import { fixtureAddress } from "../../fixture-plan-display";

export function FixtureBrowser({
  fixtures,
  selected,
  scene,
  query,
  onlySelected,
  busy,
  onQuery,
  onFilter,
  onSelect,
  table = false,
}: {
  fixtures: FixtureView[];
  selected: string[];
  scene?: SceneView;
  query: string;
  onlySelected: boolean;
  busy: boolean;
  table?: boolean;
  onQuery(value: string): void;
  onFilter(value: boolean): void;
  onSelect(ids: string[]): void;
}) {
  const anchor = useRef("");
  const visible = fixtures.filter(
    (f) =>
      fixtureMatches(f, query) && (!onlySelected || selected.includes(f.id)),
  );
  const ids = visible.map((f) => f.id);
  function choose(id: string, range: boolean, toggle: boolean) {
    onSelect(selectRange(ids, selected, id, anchor.current, range, toggle));
    if (!range) anchor.current = id;
  }
  return (
    <section
      className="wb-fixture-browser"
      aria-label={table ? "灯具配适表" : "编排灯具"}
    >
      <div className="wb-selection-toolbar">
        <div className="wb-search">
          <MagnifyingGlassIcon />
          <input
            aria-label="搜索灯具"
            placeholder="搜索名称、模式或地址"
            value={query}
            onChange={(e) => onQuery(e.target.value)}
          />
          {query && (
            <button aria-label="清除灯具搜索" onClick={() => onQuery("")}>
              <XIcon />
            </button>
          )}
        </div>
        {
          <button
            aria-pressed={onlySelected}
            className={onlySelected ? "active" : ""}
            onClick={() => onFilter(!onlySelected)}
          >
            仅已选
          </button>
        }
        {
          <button
            disabled={busy || !visible.length}
            onClick={() => onSelect(ids)}
          >
            全选结果
          </button>
        }
        {
          <button
            disabled={busy || !selected.length}
            onClick={() => onSelect([])}
          >
            清空
          </button>
        }
        <span className="wb-selection-count">
          {visible.length} 台
          {` · 已选 ${selected.length}${selected.some((id) => !ids.includes(id)) ? `（${selected.filter((id) => !ids.includes(id)).length} 台隐藏）` : ""}`}
        </span>
      </div>
      {!visible.length ? (
        <div className="wb-empty">
          <LightbulbIcon size={32} />
          <h3>{fixtures.length ? "没有符合条件的灯具" : "尚未添加灯具"}</h3>
          {fixtures.length > 0 && (
            <button
              onClick={() => {
                onQuery("");
                onFilter(false);
              }}
            >
              显示全部灯具
            </button>
          )}
        </div>
      ) : table ? (
        <div className="wb-patch-table">
          <div className="wb-patch-head">
            <span>灯具</span>
            <span>灯具模式</span>
            <span>输出域</span>
            <span>线路 / 地址</span>
          </div>
          {visible.map((f) => (
            <button
              className={`wb-patch-row ${selected.includes(f.id) ? "selected" : ""}`}
              key={f.id}
              aria-pressed={selected.includes(f.id)}
              disabled={busy}
              onClick={(e) => choose(f.id, e.shiftKey, e.metaKey || e.ctrlKey)}
            >
              <strong>
                <LightbulbIcon />
                {f.name}
              </strong>
              <span>{f.profileName}</span>
              <span>{f.domainName}</span>
              <code>
                {fixtureAddress(f)}
                {f.address !== null && f.footprint > 1
                  ? `–${f.address + f.footprint - 1}`
                  : ""}
              </code>
            </button>
          ))}
        </div>
      ) : (
        <div className="wb-fixture-grid" role="group" aria-label="灯具选择">
          {visible.map((f, index) => {
            const dimmer = scene?.values.find(
              (v) => v.fixtureId === f.id && v.attribute === "dimmer",
            );
            const colors = ["red", "green", "blue"].map((key) =>
              scene?.values.find(
                (v) => v.fixtureId === f.id && v.attribute === key,
              ),
            );
            const color = colors.every(
              (v) => v?.value != null && v.mode !== "release",
            )
              ? `rgb(${colors.map((v) => Math.round(v!.value! / 257)).join(",")})`
              : "#90a5b5";
            return (
              <button
                key={f.id}
                className={`wb-fixture-card ${selected.includes(f.id) ? "selected" : ""}`}
                aria-pressed={selected.includes(f.id)}
                aria-label={`${f.name}，${selected.includes(f.id) ? "已选" : "未选"}`}
                disabled={busy}
                onClick={(e) =>
                  choose(f.id, e.shiftKey, e.metaKey || e.ctrlKey)
                }
                onKeyDown={(e) => {
                  const offset =
                    e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
                  if (offset) {
                    e.preventDefault();
                    const buttons =
                      e.currentTarget.parentElement!.querySelectorAll<HTMLButtonElement>(
                        "button",
                      );
                    buttons[
                      Math.max(0, Math.min(buttons.length - 1, index + offset))
                    ]?.focus();
                  }
                }}
              >
                <div className="wb-card-top">
                  <span className="wb-card-number">
                    {String(fixtures.indexOf(f) + 1).padStart(2, "0")}
                  </span>
                  <span className="wb-selection-dot">
                    {selected.includes(f.id) ? selected.indexOf(f.id) + 1 : ""}
                  </span>
                </div>
                <LightbulbIcon
                  className="wb-fixture-symbol"
                  weight="duotone"
                  style={{ color }}
                  size={34}
                />
                <strong>{f.name}</strong>
                <span className="wb-card-mode">{f.profileName}</span>
                <div className="wb-card-bottom">
                  <code>{fixtureAddress(f)}</code>
                  <span>
                    {dimmer?.mode === "release"
                      ? "释放"
                      : dimmer?.value != null
                        ? `${Math.round((dimmer.value / 65535) * 100)}%`
                        : "未记录"}
                  </span>
                </div>
              </button>
            );
          })}
        </div>
      )}
    </section>
  );
}
