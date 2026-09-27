import { useState, type Dispatch, type SetStateAction } from "react";
import type { ProjectView } from "../../application-host";
export function OrderedFixturePicker({
  project,
  ids,
  setIds,
  purpose = "布置",
}: {
  purpose?: "布置" | "配适";
  project: ProjectView;
  ids: string[];
  setIds: Dispatch<SetStateAction<string[]>>;
}) {
  const [query, setQuery] = useState("");
  const [scope, setScope] = useState("all");
  const placed = new Set(project.stage.placements.map((p) => p.fixtureId));
  const group = project.groups.find((g) => g.id === scope);
  const candidates = group
    ? group.fixtureIds.flatMap((id) =>
        project.fixtures.filter((f) => f.id === id),
      )
    : project.fixtures;
  const matches = candidates.filter(
    (f) =>
      (scope === "all" ||
        (scope === "unplaced" && !placed.has(f.id)) ||
        (scope === "selected" && ids.includes(f.id)) ||
        group?.fixtureIds.includes(f.id)) &&
      `${f.name} ${f.address ?? ""}`
        .toLocaleLowerCase()
        .includes(query.trim().toLocaleLowerCase()),
  );
  const rows = [...matches].sort(
    (a, b) =>
      (ids.includes(a.id) ? ids.indexOf(a.id) : ids.length) -
      (ids.includes(b.id) ? ids.indexOf(b.id) : ids.length),
  );

  return (
    <section className="placement-picker">
      <header>
        <strong>灯具与顺序</strong>
        <span>已选 {ids.length} 台</span>
      </header>
      <input
        type="search"
        aria-label={`搜索${purpose}灯具`}
        placeholder="搜索灯具名称或地址"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <select
        aria-label={`${purpose}范围`}
        value={scope}
        onChange={(e) => setScope(e.target.value)}
      >
        <option value="all">全部灯具</option>
        <option value="unplaced">未布置灯具</option>
        <option value="selected">当前已选</option>
        {project.groups.map((g) => (
          <option key={g.id} value={g.id}>
            灯组 · {g.name}
          </option>
        ))}
      </select>
      <div className="placement-picker-actions">
        <button
          type="button"
          onClick={() =>
            setIds((old) => [
              ...old,
              ...matches.map((f) => f.id).filter((id) => !old.includes(id)),
            ])
          }
        >
          选中筛选结果
        </button>
        <button type="button" onClick={() => setIds([])}>
          清空
        </button>
        <button
          type="button"
          disabled={ids.length < 2}
          onClick={() => setIds((old) => [...old].reverse())}
        >
          反转灯序
        </button>
      </div>
      <div
        className="placement-fixtures"
        role="group"
        aria-label={`${purpose}选择`}
      >
        {rows.map((f) => {
          const rank = ids.indexOf(f.id);
          return (
            <div key={f.id} className={rank >= 0 ? "selected" : ""}>
              <label>
                <input
                  type="checkbox"
                  aria-label={`${purpose} ${f.name}`}
                  checked={rank >= 0}
                  onChange={(e) =>
                    setIds((old) =>
                      e.target.checked
                        ? [...old, f.id]
                        : old.filter((id) => id !== f.id),
                    )
                  }
                />
                <span>
                  <strong>{f.name}</strong>
                  <small>
                    {placed.has(f.id) ? "已布置" : "未布置"} · 地址{" "}
                    {f.address ?? "未配适"}
                  </small>
                </span>
              </label>
              {rank >= 0 && (
                <>
                  <b>{rank + 1}</b>
                  <button
                    type="button"
                    aria-label={`提前 ${f.name}`}
                    disabled={rank === 0}
                    onClick={() =>
                      setIds((old) => {
                        const n = [...old];
                        [n[rank - 1], n[rank]] = [n[rank], n[rank - 1]];
                        return n;
                      })
                    }
                  >
                    ↑
                  </button>
                  <button
                    type="button"
                    aria-label={`延后 ${f.name}`}
                    disabled={rank === ids.length - 1}
                    onClick={() =>
                      setIds((old) => {
                        const n = [...old];
                        [n[rank + 1], n[rank]] = [n[rank], n[rank + 1]];
                        return n;
                      })
                    }
                  >
                    ↓
                  </button>
                </>
              )}
            </div>
          );
        })}
        {!matches.length && <p className="wb-dim">没有匹配的灯具</p>}
      </div>
      {ids.some((id) => !matches.some((f) => f.id === id)) && (
        <p className="wb-dim">
          另有 {ids.filter((id) => !matches.some((f) => f.id === id)).length}{" "}
          台已选灯具被筛选隐藏，仍参与本次布置。
        </p>
      )}
    </section>
  );
}
