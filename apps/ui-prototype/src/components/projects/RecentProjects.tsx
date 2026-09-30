import { useRef, useState } from "react";
import { FolderOpenIcon, XIcon } from "@phosphor-icons/react";
import type { ApplicationHost } from "../../application-host";
import { useRecentProjects } from "./useRecentProjects";
import "./projects.css";

export function RecentProjects({
  host,
  busy,
  onOpen,
  onBrowse,
}: {
  host: ApplicationHost;
  busy: boolean;
  onOpen(id: string): Promise<boolean>;
  onBrowse(): void;
}) {
  const { entries, loading, error, request } = useRecentProjects(host);
  const [query, setQuery] = useState("");
  const search = useRef<HTMLInputElement>(null);
  const matches = entries.filter((e) =>
    `${e.name} ${e.path}`
      .toLocaleLowerCase()
      .includes(query.trim().toLocaleLowerCase()),
  );
  const disabled = busy || loading;
  return (
    <section className="recent-projects" aria-label="最近工程">
      <header>
        <h2>最近工程</h2>
        <button
          disabled={disabled}
          onClick={() => void request({ kind: "list" })}
        >
          刷新
        </button>
      </header>
      <input
        ref={search}
        type="search"
        aria-label="搜索最近工程"
        placeholder="搜索工程名称或路径"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      {error && (
        <p role="alert" className="recent-error">
          {error}
        </p>
      )}
      {loading && <p role="status">正在读取最近记录…</p>}
      {!loading && !error && entries.length === 0 && (
        <p className="recent-empty">成功打开或保存的工程会出现在这里。</p>
      )}
      {!loading && entries.length > 0 && matches.length === 0 && (
        <p>
          没有匹配的工程<button onClick={() => setQuery("")}>清空搜索</button>
        </p>
      )}
      <ul>
        {matches.map((entry) => (
          <li key={entry.id}>
            <button
              className="recent-open"
              disabled={disabled || !entry.available}
              aria-label={`打开最近工程：${entry.name}，${entry.path}`}
              title={entry.path}
              onClick={() => void onOpen(entry.id)}
            >
              <FolderOpenIcon size={22} />
              <span>
                <strong>{entry.name}</strong>
                <small>{entry.path}</small>
              </span>
            </button>
            <div className="recent-meta">
              {entry.available ? (
                <time dateTime={new Date(entry.openedAtMs).toISOString()}>
                  {new Date(entry.openedAtMs).toLocaleString("zh-CN", {
                    month: "numeric",
                    day: "numeric",
                    hour: "2-digit",
                    minute: "2-digit",
                    hour12: false,
                  })}
                </time>
              ) : (
                <span className="recent-missing">文件不可用</span>
              )}
              {!entry.available && (
                <button disabled={disabled} onClick={onBrowse}>
                  重新选择文件
                </button>
              )}
            </div>
            <button
              className="recent-forget"
              aria-label={`移除最近记录：${entry.name}，${entry.path}`}
              title="只移除最近记录，不删除工程"
              disabled={disabled}
              onClick={async () => {
                await request({ kind: "forget", id: entry.id });
                search.current?.focus();
              }}
            >
              <XIcon />
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}
