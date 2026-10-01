import { useState } from "react";
import type { FixtureView } from "../../application-host";
import type { EffectChannel } from "../../effect-types";
import { reorderEffect } from "../../effect-tools";
import {
  arrangeEffectFixtures,
  effectTargetIssues,
  type EffectOrder,
} from "../../effect-targets";
import { searchResources } from "../resources/resource-search";
import { EffectTargetReport } from "./EffectTargetReport";
import "./effect-targets.css";
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
  const currentIssues = effectTargetIssues(ids, fixtures, channels);
  const selectionIssues = selected.length
    ? effectTargetIssues(selected, fixtures, channels)
    : [];
  const candidates = searchResources(
    fixtures
      .filter((f) => !ids.includes(f.id))
      .map((f) => ({
        id: f.id,
        label: f.name,
        detail: `${f.profileName} ${f.domainName}`,
        keywords: `${f.universe ?? ""} ${f.address ?? ""}`,
      })),
    query,
  );
  function change(next: string[]) {
    if (next.length !== ids.length || next.some((id, i) => id !== ids[i]))
      onChange(next);
  }
  return (
    <details className="effect-order">
      <summary>灯具与顺序 · {ids.length} 台</summary>
      <button
        type="button"
        disabled={!selected.length || selectionIssues.length > 0}
        onClick={() => change([...selected])}
      >
        采用当前选灯顺序
      </button>
      {!!selectionIssues.length && (
        <EffectTargetReport issues={selectionIssues} />
      )}
      <div className="effect-order-actions" role="group" aria-label="整组灯序">
        {(
          [
            ["reverse", "反转排序"],
            ["oddFirst", "奇数位在前"],
            ["name", "按名称排序"],
            ["patch", "按配适排序"],
          ] as [EffectOrder, string][]
        ).map(([order, label]) => (
          <button
            key={order}
            type="button"
            disabled={ids.length < 2}
            title={
              order === "oddFirst"
                ? "按当前顺序先排第 1、3、5…位，再排第 2、4、6…位；保留所有灯具"
                : order === "patch"
                  ? "按控制域、输出路、地址排序，未配适灯具保留原顺序置后"
                  : undefined
            }
            onClick={() => change(arrangeEffectFixtures(ids, fixtures, order))}
          >
            {label}
          </button>
        ))}
      </div>
      <EffectTargetReport issues={currentIssues} />
      <ol>
        {ids.map((id, i) => (
          <li key={id}>
            <span
              title={fixtures.find((f) => f.id === id)?.name ?? "灯具已删除"}
            >
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
        data-editor-navigation="true"
        aria-label="搜索待添加灯具"
        placeholder="搜索待添加灯具"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            e.stopPropagation();
          }
        }}
      />
      <small>
        {candidates.length} 台待添加灯具
        {candidates.length > 50 && " · 显示前 50 台，请搜索缩小范围"}
      </small>
      <div className="effect-add-fixtures">
        {candidates.slice(0, 50).map((entry) => (
          <button
            type="button"
            key={entry.id}
            disabled={
              effectTargetIssues([entry.id], fixtures, channels).length > 0
            }
            title={
              effectTargetIssues([entry.id], fixtures, channels)
                .map((i) => i.reason)
                .join("；") || entry.detail
            }
            onClick={() => change([...ids, entry.id])}
          >
            添加 {entry.label}
          </button>
        ))}
      </div>
    </details>
  );
}
