import { CopyIcon, PlusIcon, StackIcon } from "@phosphor-icons/react";
import type { SceneView } from "../../application-host";
export function SceneLibrary({
  scenes,
  selected,
  query,
  busy,
  canCreate,
  onQuery,
  onSelect,
  onAdd,
  onDuplicate,
}: {
  scenes: SceneView[];
  selected: string;
  query: string;
  busy: boolean;
  canCreate: boolean;
  onQuery(value: string): void;
  onSelect(scene: SceneView): void;
  onAdd(): void;
  onDuplicate(): void;
}) {
  const visible = scenes.filter((s) =>
    s.name.toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()),
  );
  return (
    <aside className="wb-library">
      <div className="wb-section-title">
        <h2>
          <StackIcon />
          场景
        </h2>
        <span>{scenes.length}</span>
      </div>
      <input
        className="wb-scene-search"
        aria-label="搜索场景"
        placeholder="搜索场景"
        value={query}
        onChange={(e) => onQuery(e.target.value)}
      />
      <div className="wb-library-actions">
        <button disabled={busy || !canCreate} onClick={onAdd}>
          <PlusIcon />
          新建
        </button>
        <button
          disabled={busy || !selected}
          title="复制场景（⌘D / Ctrl+D）"
          onClick={onDuplicate}
        >
          <CopyIcon />
          复制
        </button>
      </div>
      <div className="wb-scene-list">
        {visible.map((scene) => (
          <button
            key={scene.id}
            className={selected === scene.id ? "active" : ""}
            aria-pressed={selected === scene.id}
            disabled={busy}
            onClick={() => onSelect(scene)}
          >
            <span className="wb-scene-index">
              {String(scenes.indexOf(scene) + 1).padStart(2, "0")}
            </span>
            <div>
              <strong>{scene.name}</strong>
              <small>
                {
                  new Set([
                    ...scene.values.map((v) => v.fixtureId),
                    ...scene.effects.flatMap((e) => e.fixtureIds),
                  ]).size
                }{" "}
                台灯具 · {scene.values.length} 项属性
                {!!scene.effects.length && ` · ${scene.effects.length} 个效果`}
              </small>
            </div>
          </button>
        ))}
        {!visible.length && (
          <p className="wb-dim">
            {scenes.length ? "未找到场景" : "尚未创建场景"}
          </p>
        )}
      </div>
    </aside>
  );
}
