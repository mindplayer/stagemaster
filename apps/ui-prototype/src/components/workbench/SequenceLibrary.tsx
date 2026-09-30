import { PlusIcon, CopyIcon, ListNumbersIcon } from "@phosphor-icons/react";
import type { SequenceView } from "../../sequence-types";
export function SequenceLibrary({
  lists,
  count,
  selectedId,
  hasScenes,
  query,
  setQuery,
  busy,
  onAdd,
  onDuplicate,
  onSelect,
}: {
  lists: SequenceView[];
  count: number;
  selectedId?: string;
  hasScenes: boolean;
  query: string;
  setQuery(value: string): void;
  busy: boolean;
  onAdd(): void;
  onDuplicate(): void;
  onSelect(id: string): void;
}) {
  return (
    <aside className="wb-library">
      <div className="wb-section-title">
        <h2>
          <ListNumbersIcon />
          场景列表
        </h2>
        <span>{count}</span>
      </div>
      <input
        aria-label="搜索列表"
        placeholder="搜索列表"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <div className="wb-library-actions">
        <button disabled={busy || !hasScenes} onClick={() => onAdd()}>
          <PlusIcon />
          新建列表
        </button>
        <button
          aria-label="复制列表"
          disabled={busy || !selectedId}
          onClick={() => onDuplicate()}
        >
          <CopyIcon />
        </button>
      </div>
      <div className="wb-scene-list">
        {lists.map((s) => (
          <button
            key={s.id}
            aria-pressed={selectedId === s.id}
            className={selectedId === s.id ? "active" : ""}
            disabled={busy}
            onClick={() => onSelect(s.id)}
          >
            <div>
              <strong>{s.name}</strong>
              <small>
                {s.steps.length} 步 · {s.repeat === "loop" ? "循环" : "单次"}
              </small>
            </div>
          </button>
        ))}
        {!lists.length && (
          <p className="wb-dim">
            {count
              ? "未找到列表"
              : hasScenes
                ? "尚未创建列表"
                : "请先在编排中创建场景"}
          </p>
        )}
      </div>
    </aside>
  );
}
