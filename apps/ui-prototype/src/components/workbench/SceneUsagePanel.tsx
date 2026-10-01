import { useState } from "react";
import type { ProjectView } from "../../application-host";
import {
  sceneUsages,
  usageMatches,
  type SceneUsageTarget,
} from "./scene-usage";
import "./scene-usage.css";
const PAGE_SIZE = 20;
export function SceneUsagePanel({
  project,
  sceneId,
  busy,
  onLocate,
}: {
  project: ProjectView;
  sceneId: string;
  busy: boolean;
  onLocate(target: SceneUsageTarget): void;
}) {
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(0);
  const usages = sceneUsages(project, sceneId);
  const filtered = usages.filter((entry) => usageMatches(entry, query));
  const pages = Math.max(1, Math.ceil(filtered.length / PAGE_SIZE));
  const current = Math.min(page, pages - 1);
  return (
    <section className="scene-usage" aria-label="场景使用位置">
      <h3>
        使用位置 <span>{usages.length}</span>
      </h3>
      {!usages.length ? (
        <p>尚未被执行步骤或音乐编排引用</p>
      ) : (
        <>
          <input
            aria-label="搜索使用位置"
            placeholder="搜索列表、片段或时间"
            value={query}
            onChange={(event) => {
              setQuery(event.target.value);
              setPage(0);
            }}
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                event.preventDefault();
                event.stopPropagation();
                setQuery("");
                setPage(0);
              }
            }}
          />
          <div className="scene-usage-list">
            {filtered
              .slice(current * PAGE_SIZE, (current + 1) * PAGE_SIZE)
              .map((entry) => (
                <button
                  key={entry.key}
                  disabled={busy}
                  onClick={() => onLocate(entry.target)}
                  title={`编辑此使用位置：${entry.title}，${entry.detail}`}
                >
                  <strong>{entry.title}</strong>
                  <small>{entry.detail}</small>
                </button>
              ))}
            {!filtered.length && <p>没有匹配的使用位置</p>}
          </div>
          {pages > 1 && (
            <nav aria-label="使用位置分页">
              <button disabled={!current} onClick={() => setPage(current - 1)}>
                上一页
              </button>
              <span>
                {current + 1} / {pages}
              </span>
              <button
                disabled={current + 1 === pages}
                onClick={() => setPage(current + 1)}
              >
                下一页
              </button>
            </nav>
          )}
        </>
      )}
    </section>
  );
}
