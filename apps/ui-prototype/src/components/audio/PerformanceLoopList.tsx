import { useState } from "react";
import type { AudioTimeline } from "../../audio-types";
import { audioTime } from "../../audio-tools";
import { loopPlaysLabel } from "./performance-loop-draft";
import "./performance-loops.css";
import { DockPane } from "../layout/DockPane";
import { PerformanceLoopGroupInspector } from "./PerformanceLoopGroupInspector";
import type { OrderedSelection } from "../selection/ordered-selection";
import type { AudioLoopRegion } from "../../audio-performance-types";
import type { usePerformanceLoopBatch } from "./usePerformanceLoopBatch";

export function PerformanceLoopList({
  track,
  selected,
  busy,
  onSelect,
  onAdd,
  group,
}: {
  track: AudioTimeline;
  selected: string;
  busy: boolean;
  onSelect(id: string): void;
  onAdd(): void;
  group: {
    active: boolean;
    visible: boolean;
    selection: OrderedSelection<AudioLoopRegion>;
    actions: ReturnType<typeof usePerformanceLoopBatch>;
    onMode(): void;
  };
}) {
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState("all");
  const regions = track.loopRegions ?? [];
  const items = regions.filter(
    (r) =>
      r.name.toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()) &&
      (filter === "all" || (filter === "enabled" ? r.enabled : !r.enabled)),
  );
  return (
    <section className="audio-performance-library" aria-label="演出循环目录">
      <header>
        <h3>演出循环 · {regions.length}</h3>
        <button
          disabled={busy || group.actions.pending || regions.length >= 128}
          onClick={onAdd}
        >
          添加循环区段
        </button>
      </header>
      <div className="performance-loop-actions">
        <button
          disabled={busy || group.actions.pending}
          aria-pressed={group.active}
          onClick={group.onMode}
        >
          成组选择循环区段
        </button>
        {group.active && (
          <>
            <button
              disabled={busy || group.actions.pending || !items.length}
              onClick={() => group.selection.replace(items.map((r) => r.id))}
            >
              选择可见区段
            </button>
            <button
              disabled={
                busy || group.actions.pending || !group.selection.ids.length
              }
              onClick={() => group.selection.replace([])}
            >
              清空所选区段
            </button>
          </>
        )}
      </div>
      <input
        aria-label="搜索循环区段"
        placeholder="搜索区段名称"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <select
        aria-label="循环区段状态筛选"
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
      >
        <option value="all">全部区段</option>
        <option value="enabled">已启用区段</option>
        <option value="disabled">已停用区段</option>
      </select>
      {(query || filter !== "all") && (
        <button
          onClick={() => {
            setQuery("");
            setFilter("all");
          }}
        >
          清除区段筛选
        </button>
      )}
      {!group.active &&
        regions.some((r) => r.id === selected) &&
        !items.some((r) => r.id === selected) && (
          <p role="status">所选区段在筛选范围外，右侧属性仍可编辑</p>
        )}
      <div className="performance-loop-list">
        {items.map((r) => (
          <button
            key={r.id}
            className={
              (
                group.active
                  ? group.selection.ids.includes(r.id)
                  : selected === r.id
              )
                ? "selected"
                : ""
            }
            aria-pressed={
              group.active
                ? group.selection.ids.includes(r.id)
                : selected === r.id
            }
            disabled={busy || group.actions.pending}
            onClick={(e) =>
              group.active
                ? group.selection.toggle(r.id, items, e.shiftKey)
                : onSelect(r.id)
            }
          >
            <time>
              {audioTime(r.startMs)} — {audioTime(r.endMs)}
            </time>
            <strong>{r.name}</strong>
            <span>
              {loopPlaysLabel(r.plays)}
              {!r.enabled ? " · 已停用" : ""}
              {r.locked ? " · 已锁定" : ""}
            </span>
          </button>
        ))}
      </div>
      {!items.length && (
        <small>
          {regions.length
            ? "未找到区段"
            : "固定总次数或持续等待，循环区段以外继续原时间线"}
        </small>
      )}
      <DockPane region="inspector" visible={group.visible && group.active}>
        <PerformanceLoopGroupInspector
          regions={regions}
          ids={group.selection.ids}
          visibleItems={items}
          actions={group.actions}
          busy={busy}
          visible={group.visible && group.active}
        />
      </DockPane>
    </section>
  );
}
