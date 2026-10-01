import { rangeScrollTop } from "../layout/scroll-range";
import { useEffect, useRef, useImperativeHandle, type Ref } from "react";
import { useSceneBatchCopy } from "./useSceneBatchCopy";
import { SceneBatchControls } from "./SceneBatchControls";
import "./scene-batch.css";
import { CopyIcon, PlusIcon, StackIcon } from "@phosphor-icons/react";
import type { SceneView } from "../../application-host";
export interface SceneLibraryHandle {
  duplicateSelected(): boolean;
  selectVisible(): boolean;
}
export function SceneLibrary({
  ref,
  beforeChange,
  onCopyMany,
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
  ref?: Ref<SceneLibraryHandle>;
  beforeChange(): Promise<boolean>;
  onCopyMany(ids: string[]): Promise<string[] | null>;
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
  const list = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const frame = requestAnimationFrame(() => {
      const host = list.current;
      if (!host?.isConnected) return;
      const row = [
        ...host.querySelectorAll<HTMLElement>("[data-scene-id]"),
      ].find((el) => el.dataset.sceneId === selected);
      if (!row) return;
      const rect = row.getBoundingClientRect(),
        parent = host.getBoundingClientRect();
      host.scrollTop = rangeScrollTop(
        host.scrollTop,
        host.clientHeight,
        host.scrollHeight,
        {
          top: rect.top - parent.top - host.clientTop + host.scrollTop,
          bottom: rect.bottom - parent.top - host.clientTop + host.scrollTop,
        },
      );
    });
    return () => cancelAnimationFrame(frame);
  }, [selected]);
  const batch = useSceneBatchCopy(scenes, busy, beforeChange, onCopyMany);
  useImperativeHandle(ref, () => ({
    selectVisible() {
      if (!batch.active) return false;
      batch.replace([...batch.selected, ...visible.map((scene) => scene.id)]);
      return true;
    },
    duplicateSelected() {
      if (!batch.active) return false;
      void batch.copy();
      return true;
    },
  }));
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
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            e.preventDefault();
            e.stopPropagation();
            onQuery("");
          }
        }}
      />
      <div className="wb-library-actions">
        <button disabled={batch.blocked || !canCreate} onClick={onAdd}>
          <PlusIcon />
          新建
        </button>
        <button
          disabled={batch.blocked || !selected || batch.active}
          title="复制场景（⌘D / Ctrl+D）"
          onClick={onDuplicate}
        >
          <CopyIcon />
          复制
        </button>
      </div>
      <button
        disabled={batch.blocked}
        aria-pressed={batch.active}
        onClick={() => void batch.toggleMode()}
      >
        {batch.active ? "返回单场景编辑" : "批量整理场景"}
      </button>
      {batch.active && <SceneBatchControls batch={batch} visible={visible} />}
      <div
        ref={list}
        className="wb-scene-list"
        aria-label={batch.active ? "批量场景选择" : "场景目录"}
      >
        {visible.map((scene) => (
          <button
            key={scene.id}
            data-scene-id={scene.id}
            className={
              (
                batch.active
                  ? batch.selected.includes(scene.id)
                  : selected === scene.id
              )
                ? "active"
                : ""
            }
            role={batch.active ? "checkbox" : undefined}
            aria-label={batch.active ? `选择场景：${scene.name}` : undefined}
            aria-checked={
              batch.active ? batch.selected.includes(scene.id) : undefined
            }
            aria-pressed={batch.active ? undefined : selected === scene.id}
            disabled={batch.blocked}
            onClick={(e) =>
              batch.active
                ? batch.toggle(scene.id, visible, e.shiftKey)
                : onSelect(scene)
            }
            onKeyDown={(e) => {
              if (batch.active && (e.key === " " || e.key === "Enter")) {
                e.preventDefault();
                batch.toggle(scene.id, visible, e.shiftKey);
              }
            }}
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
