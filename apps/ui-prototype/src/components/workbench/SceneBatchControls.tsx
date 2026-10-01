import type { SceneView } from "../../application-host";
import type { useSceneBatchCopy } from "./useSceneBatchCopy";
export function SceneBatchControls({
  batch,
  visible,
  onRemove,
}: {
  batch: ReturnType<typeof useSceneBatchCopy>;
  visible: SceneView[];
  onRemove(ids: string[]): void;
}) {
  const hidden = batch.selected.filter(
    (id) => !visible.some((s) => s.id === id),
  ).length;
  return (
    <section className="scene-batch-controls" aria-label="场景批量整理">
      <small role="status">
        已选 {batch.selected.length} 个{hidden ? ` · ${hidden} 个在筛选外` : ""}
      </small>
      <div className="wb-library-actions">
        <button
          disabled={batch.blocked || !visible.length}
          onClick={() =>
            batch.replace([...batch.selected, ...visible.map((s) => s.id)])
          }
        >
          选中筛选场景
        </button>
        <button
          disabled={batch.blocked || !batch.selected.length}
          onClick={() => batch.replace([])}
        >
          清空场景选择
        </button>
      </div>
      <button
        className="primary"
        title="复制所选场景（⌘D / Ctrl+D）"
        disabled={batch.blocked || !batch.selected.length}
        onClick={() => void batch.copy()}
      >
        复制所选场景
      </button>
      <button
        className="wb-danger"
        disabled={batch.blocked || !batch.selected.length}
        onClick={() => onRemove([...batch.selected])}
      >
        删除所选场景…
      </button>
      {batch.problem && (
        <p role="alert" className="wb-error">
          {batch.problem}
        </p>
      )}
    </section>
  );
}
