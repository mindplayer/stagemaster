import { useEffect, useRef, useState } from "react";
import type { ProjectView } from "../../application-host";
import type { SceneUsageTarget } from "./scene-usage";
import { sceneRemovalReview } from "./scene-removal";
import { SceneUsagePanel } from "./SceneUsagePanel";
import "./scene-removal.css";
export function SceneRemovalDialog({
  project,
  ids,
  busy,
  error,
  onCancel,
  onRemove,
  onLocate,
}: {
  project: ProjectView;
  ids: readonly string[];
  busy: boolean;
  error: string;
  onCancel(): void;
  onRemove(): void;
  onLocate(sceneId: string, target: SceneUsageTarget): void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const cancelButton = useRef<HTMLButtonElement>(null);
  const initialFocus = useRef(false);
  const [selected, setSelected] = useState(ids[0]);
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(0);
  useEffect(() => {
    dialog.current?.showModal();
  }, []);
  useEffect(() => {
    if (!busy && !initialFocus.current && dialog.current?.open) {
      cancelButton.current?.focus();
      initialFocus.current = true;
    }
  }, [busy]);
  let problem = "";
  let review: ReturnType<typeof sceneRemovalReview> = [];
  try {
    review = sceneRemovalReview(project, ids);
  } catch (e) {
    problem = e instanceof Error ? e.message : String(e);
  }
  const used = review.filter((scene) => scene.usages.length).length;
  const visible = review.filter((scene) =>
    scene.name.toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()),
  );
  const pages = Math.max(1, Math.ceil(visible.length / 20)),
    current = Math.min(page, pages - 1);
  const active = review.find((scene) => scene.id === selected) ?? review[0];
  return (
    <dialog
      ref={dialog}
      className="wb-dialog scene-removal-dialog"
      aria-labelledby="scene-removal-title"
      onCancel={(event) => {
        event.preventDefault();
        if (!busy) onCancel();
      }}
    >
      <h2 id="scene-removal-title">删除 {ids.length} 个场景？</h2>
      <p>将删除所列场景及其动态效果，可撤销恢复。</p>
      {used > 0 && (
        <p role="status">
          其中 {used} 个场景仍被使用，请先处理引用；本次不会删除任何场景。
        </p>
      )}
      {(problem || error) && (
        <p role="alert" className="wb-error">
          {problem || error}
        </p>
      )}
      {!problem && (
        <div className="scene-removal-content">
          <section aria-label="待删除场景">
            <input
              aria-label="筛选待删除场景"
              placeholder="搜索待删除名称"
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setPage(0);
              }}
              onKeyDown={(e) => {
                if (e.key === "Escape" && query) {
                  e.preventDefault();
                  e.stopPropagation();
                  setQuery("");
                  setPage(0);
                }
              }}
            />
            <div className="scene-removal-list">
              {visible.slice(current * 20, (current + 1) * 20).map((scene) => (
                <button
                  key={scene.id}
                  disabled={busy}
                  aria-pressed={active?.id === scene.id}
                  onClick={() => setSelected(scene.id)}
                >
                  <strong>{scene.name}</strong>
                  <small>
                    {scene.usages.length
                      ? `${scene.usages.length} 个使用位置`
                      : "未被引用"}
                  </small>
                </button>
              ))}
            </div>
            {!visible.length && <p>没有匹配的场景，筛选不改变删除范围</p>}
            {pages > 1 && (
              <nav aria-label="待删除场景分页">
                <button
                  disabled={!current}
                  onClick={() => setPage(current - 1)}
                >
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
          </section>
          {active && (
            <section aria-label={`检查引用：${active.name}`}>
              <h3>{active.name}</h3>
              <SceneUsagePanel
                key={active.id}
                project={project}
                sceneId={active.id}
                busy={busy}
                onLocate={(target) => onLocate(active.id, target)}
              />
            </section>
          )}
        </div>
      )}
      <div className="wb-dialog-actions">
        <button ref={cancelButton} autoFocus disabled={busy} onClick={onCancel}>
          取消
        </button>
        <button
          className="wb-danger"
          disabled={busy || !!problem || used > 0}
          onClick={onRemove}
        >
          删除所列 {ids.length} 个场景
        </button>
      </div>
    </dialog>
  );
}
