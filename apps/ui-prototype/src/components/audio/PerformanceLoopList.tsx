import { useState } from "react";
import type { AudioTimeline } from "../../audio-types";
import { audioTime } from "../../audio-tools";
import { loopPlaysLabel } from "./performance-loop-draft";
import "./performance-loops.css";

export function PerformanceLoopList({ track, selected, busy, onSelect, onAdd }: {
  track: AudioTimeline;
  selected: string;
  busy: boolean;
  onSelect(id: string): void;
  onAdd(): void;
}) {
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState("all");
  const regions = track.loopRegions ?? [];
  const items = regions.filter((r) => r.name.toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()) &&
    (filter === "all" || (filter === "enabled" ? r.enabled : !r.enabled)));
  return (
    <section className="audio-performance-library" aria-label="演出循环目录">
      <header>
        <h3>演出循环 · {regions.length}</h3>
        <button disabled={busy || regions.length >= 128} onClick={onAdd}>添加循环区段</button>
      </header>
      <input aria-label="搜索循环区段" placeholder="搜索区段名称" value={query} onChange={(e) => setQuery(e.target.value)} />
      <select aria-label="循环区段状态筛选" value={filter} onChange={(e) => setFilter(e.target.value)}>
        <option value="all">全部区段</option><option value="enabled">已启用区段</option><option value="disabled">已停用区段</option>
      </select>
      {(query || filter !== "all") && <button onClick={() => { setQuery(""); setFilter("all"); }}>清除区段筛选</button>}
      {regions.some((r) => r.id === selected) && !items.some((r) => r.id === selected) &&
        <p role="status">所选区段在筛选范围外，右侧属性仍可编辑</p>}
      <div className="performance-loop-list">
        {items.map((r) => (
          <button key={r.id} className={selected === r.id ? "selected" : ""} aria-pressed={selected === r.id}
            disabled={busy} onClick={() => onSelect(r.id)}>
            <time>{audioTime(r.startMs)} — {audioTime(r.endMs)}</time>
            <strong>{r.name}</strong>
            <span>{loopPlaysLabel(r.plays)}{!r.enabled ? " · 已停用" : ""}{r.locked ? " · 已锁定" : ""}</span>
          </button>
        ))}
      </div>
      {!items.length && <small>{regions.length ? "未找到区段" : "固定总次数或持续等待，循环区段以外继续原时间线"}</small>}
    </section>
  );
}
