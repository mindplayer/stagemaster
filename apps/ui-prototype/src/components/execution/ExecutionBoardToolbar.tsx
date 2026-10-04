import {
  allSources,
  sourceKinds,
  sourceStates,
  type BoardFilter,
  type SourceStateFilter,
} from "../../execution-board";
export function ExecutionBoardToolbar({
  filter,
  onFilter,
  counts,
  shown,
  total,
  dirty,
  hiddenRunning,
  hiddenPaused,
  observed,
  missingPins,
  onPrune,
}: {
  filter: BoardFilter;
  onFilter(value: BoardFilter): void;
  counts: Record<string, number>;
  shown: number;
  total: number;
  dirty: number;
  hiddenRunning: number;
  hiddenPaused: number;
  observed: boolean;
  missingPins: number;
  onPrune(): void;
}) {
  return (
    <div className="execution-board-toolbar">
      <div className="execution-board-overview" aria-label="后台节目概览">
        <span>{observed ? "节目状态" : "最后已知状态"}</span>
        {(["Running", "Paused", "Finished"] as const).map((status) => (
          <button
            key={status}
            aria-pressed={filter.status === status}
            onClick={() => onFilter({ ...allSources, status })}
          >
            {sourceStates[status]} {counts[status] ?? 0}
          </button>
        ))}
        <button
          aria-pressed={filter.scope === "drafts"}
          onClick={() => onFilter({ ...allSources, scope: "drafts" })}
        >
          未应用输入 {dirty}
        </button>
      </div>
      <div className="execution-board-filters">
        <input
          aria-label="搜索已准备节目"
          placeholder="搜索节目名称或类型"
          value={filter.query}
          onChange={(e) => onFilter({ ...filter, query: e.target.value })}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              onFilter({ ...filter, query: "" });
              e.preventDefault();
              e.stopPropagation();
            }
          }}
        />
        <select
          aria-label="节目类型"
          value={filter.kind}
          onChange={(e) =>
            onFilter({ ...filter, kind: e.target.value as BoardFilter["kind"] })
          }
        >
          {Object.entries(sourceKinds).map(([key, name]) => (
            <option key={key} value={key}>
              {name}
            </option>
          ))}
        </select>
        <select
          aria-label="节目状态筛选"
          value={filter.status}
          onChange={(e) =>
            onFilter({ ...filter, status: e.target.value as SourceStateFilter })
          }
        >
          {Object.entries(sourceStates).map(([key, name]) => (
            <option key={key} value={key}>
              {name}
            </option>
          ))}
        </select>
        <button
          aria-pressed={filter.scope === "pinned"}
          onClick={() =>
            onFilter({
              ...filter,
              scope: filter.scope === "pinned" ? "all" : "pinned",
            })
          }
        >
          只看常用
        </button>
        <button
          disabled={Object.keys(allSources).every(
            (k) =>
              filter[k as keyof BoardFilter] ===
              allSources[k as keyof BoardFilter],
          )}
          onClick={() => onFilter({ ...allSources })}
        >
          显示全部
        </button>
      </div>
      <p className="execution-board-count">
        显示 {shown} / {total} 个节目
        {(hiddenRunning > 0 || hiddenPaused > 0) && (
          <span>
            {" "}
            · 筛选外{!observed ? "最后已知" : ""}：{hiddenRunning} 个运行中，
            {hiddenPaused} 个已暂停
          </span>
        )}
      </p>
      {missingPins > 0 && (
        <p className="execution-board-count">
          {missingPins} 个常用节目未载入{" "}
          <button onClick={onPrune}>清理未载入固定项</button>
        </p>
      )}
    </div>
  );
}
