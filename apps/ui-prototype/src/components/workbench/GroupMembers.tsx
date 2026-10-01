import { useState } from "react";
import type { FixtureView } from "../../application-host";
import { moveMember } from "../../library-tools";
import { appendGroupMembers, removeGroupMembers } from "../../group-members";
import { searchResources } from "../resources/resource-search";
import "./group-members.css";

export function GroupMembers({
  fixtures,
  ids,
  onChange,
}: {
  fixtures: FixtureView[];
  ids: string[];
  onChange(ids: string[]): void;
}) {
  const [mode, setMode] = useState<"members" | "available">("members");
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(0);
  const members = new Set(ids);
  const byId = new Map(fixtures.map((f) => [f.id, f]));
  const rank = new Map(ids.map((id, index) => [id, index]));
  const candidates = fixtures.filter((f) => !members.has(f.id));
  const rows =
    mode === "members"
      ? ids.map((id) => ({ id, label: byId.get(id)?.name ?? "灯具已删除" }))
      : candidates.map((f) => ({ id: f.id, label: f.name }));
  const matches = searchResources(
    rows.map((entry) => {
      const f = byId.get(entry.id);
      return {
        ...entry,
        detail: `${f?.profileName ?? ""} ${f?.domainName ?? ""}`,
        keywords: `${f?.universe ?? ""}.${f?.address ?? ""}`,
      };
    }),
    query,
  );
  const lastPage = Math.max(0, Math.ceil(matches.length / 50) - 1);
  const currentPage = Math.min(page, lastPage);
  const hidden = ids.length - matches.length;
  function change(next: string[]) {
    onChange(next);
  }
  return (
    <section className="group-members" aria-label="灯组成员管理">
      <div className="wb-resource-tools" role="group" aria-label="灯组成员范围">
        <button
          type="button"
          aria-pressed={mode === "members"}
          onClick={() => {
            setMode("members");
            setPage(0);
          }}
        >
          已加入 {ids.length}
        </button>
        <button
          type="button"
          aria-pressed={mode === "available"}
          onClick={() => {
            setMode("available");
            setPage(0);
          }}
        >
          可加入 {candidates.length}
        </button>
      </div>
      <input
        type="search"
        aria-label="搜索灯组成员"
        placeholder="搜索名称、模式或地址"
        value={query}
        onChange={(e) => {
          setQuery(e.target.value);
          setPage(0);
        }}
      />
      <div className="wb-resource-tools">
        <span>匹配 {matches.length} 台</span>
        {mode === "available" ? (
          <button
            type="button"
            disabled={!matches.length || ids.length + matches.length > 10000}
            onClick={() =>
              change(
                appendGroupMembers(
                  ids,
                  matches.map((m) => m.id),
                ),
              )
            }
          >
            加入全部筛选结果
          </button>
        ) : (
          <button
            type="button"
            disabled={!matches.length}
            onClick={() =>
              change(
                removeGroupMembers(
                  ids,
                  matches.map((m) => m.id),
                ),
              )
            }
          >
            移出全部筛选结果
          </button>
        )}
        {query && (
          <button
            type="button"
            onClick={() => {
              setQuery("");
              setPage(0);
            }}
          >
            清除成员搜索
          </button>
        )}
      </div>
      {mode === "members" && hidden > 0 && (
        <p role="status">{hidden} 台成员被搜索隐藏，保存时仍保留</p>
      )}
      <ol
        className="group-member-list"
        aria-label={mode === "members" ? "完整灯序中的成员" : "可加入的灯具"}
      >
        {matches
          .slice(currentPage * 50, (currentPage + 1) * 50)
          .map((entry) => {
            const index = rank.get(entry.id) ?? -1;
            const fixture = byId.get(entry.id);
            function move(delta: number) {
              change(moveMember(ids, index, delta));
              if (!query) setPage(Math.floor((index + delta) / 50));
            }
            return (
              <li key={entry.id}>
                <span className="group-member-name">
                  <strong>
                    {mode === "members" && `${index + 1}. `}
                    {entry.label}
                  </strong>
                  <small>
                    {fixture
                      ? `${fixture.profileName} · ${fixture.universe ?? "—"}.${fixture.address ?? "—"}`
                      : "成员身份已失效"}
                  </small>
                </span>
                <div className="group-member-actions">
                  {mode === "members" ? (
                    <>
                      <button
                        type="button"
                        aria-label={`上移第 ${index + 1} 台灯具`}
                        title="在完整灯序中上移一位"
                        disabled={!index}
                        onClick={() => move(-1)}
                      >
                        ↑
                      </button>
                      <button
                        type="button"
                        aria-label={`下移第 ${index + 1} 台灯具`}
                        title="在完整灯序中下移一位"
                        disabled={index === ids.length - 1}
                        onClick={() => move(1)}
                      >
                        ↓
                      </button>
                      <button
                        type="button"
                        aria-label={`移出${entry.label}`}
                        onClick={() =>
                          change(removeGroupMembers(ids, [entry.id]))
                        }
                      >
                        移出
                      </button>
                    </>
                  ) : (
                    <button
                      type="button"
                      disabled={ids.length >= 10000}
                      onClick={() =>
                        change(appendGroupMembers(ids, [entry.id]))
                      }
                    >
                      加入 {entry.label}
                    </button>
                  )}
                </div>
              </li>
            );
          })}
        {!matches.length && (
          <li>
            {mode === "members" ? "没有匹配的成员" : "没有匹配的可加入灯具"}
          </li>
        )}
      </ol>
      {matches.length > 50 && (
        <div className="wb-resource-tools">
          <button
            type="button"
            disabled={!currentPage}
            onClick={() => setPage(currentPage - 1)}
          >
            上一页成员
          </button>
          <span>
            {currentPage + 1} / {lastPage + 1} 页
          </span>
          <button
            type="button"
            disabled={currentPage === lastPage}
            onClick={() => setPage(currentPage + 1)}
          >
            下一页成员
          </button>
        </div>
      )}
    </section>
  );
}
