import { useEffect, useRef } from "react";
import { audioTime } from "../../audio-tools";
import type {
  clipGroupSelection,
  ClipGroupOperation,
} from "./clip-group-tools";
export function AudioClipGroupInspector({
  selection,
  value,
  problem,
  removing,
  blocked,
  visible,
  onValue,
  onReset,
  onRemove,
  onRun,
}: {
  selection: ReturnType<typeof clipGroupSelection>;
  value: string;
  problem: string;
  removing: boolean;
  blocked: boolean;
  visible: boolean;
  onValue(value: string): void;
  onReset(): void;
  onRemove(value: boolean): void;
  onRun(kind: ClipGroupOperation): void;
}) {
  const target = useRef<HTMLInputElement>(null),
    cancel = useRef<HTMLButtonElement>(null),
    remove = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (problem && visible) target.current?.focus();
  }, [problem, visible]);
  useEffect(() => {
    if (removing && visible) cancel.current?.focus();
  }, [removing, visible]);
  function cancelRemoval() {
    onRemove(false);
    remove.current?.focus();
  }
  return (
    <section
      className="audio-marker-batch audio-batch-inspector"
      aria-label="灯光片段组属性"
      onKeyDown={(e) => {
        if (e.key !== "Escape" || blocked) return;
        e.preventDefault();
        e.stopPropagation();
        if (removing) cancelRemoval();
        else onReset();
      }}
    >
      <h3>灯光片段组属性</h3>
      <p role="status">
        已选 {selection.items.length} 个
        {selection.hidden ? `，其中 ${selection.hidden} 个在筛选外` : ""}
        {selection.locked ? `，${selection.locked} 个已锁定` : ""}
        {selection.inactive ? `，${selection.inactive} 个已停用` : ""}
      </p>
      {!selection.items.length ? (
        <p>在左侧选择灯光片段。</p>
      ) : (
        <div className="audio-batch-operations">
          <p>
            原范围 {audioTime(selection.first)} — {audioTime(selection.last)}
          </p>
          <label>
            目标起点（秒）
            <input
              ref={target}
              aria-label="片段组目标起点（秒）"
              inputMode="decimal"
              value={value}
              disabled={blocked}
              onChange={(e) => onValue(e.target.value)}
            />
          </label>
          <div className="audio-batch-selection">
            <button
              disabled={blocked || !!selection.locked}
              onClick={() => onRun("move")}
            >
              移动所选片段
            </button>
            <button disabled={blocked} onClick={() => onRun("copy")}>
              复制所选片段
            </button>
            <button
              ref={remove}
              disabled={blocked || !!selection.locked}
              onClick={() => onRemove(true)}
            >
              删除所选片段
            </button>
            <button disabled={blocked} onClick={onReset}>
              取消目标修改
            </button>
          </div>
          <div className="audio-batch-selection">
            <button
              disabled={
                blocked ||
                !!selection.locked ||
                selection.inactive === selection.items.length
              }
              onClick={() => onRun("disable")}
            >
              停用所选片段
            </button>
            <button
              disabled={blocked || !!selection.locked || !selection.inactive}
              onClick={() => onRun("enable")}
            >
              恢复所选片段
            </button>
          </div>
          {selection.locked > 0 && (
            <small>
              锁定片段可复制；移动、删除或切换启停前请先解锁，或移出选择。
            </small>
          )}
          {removing && (
            <div
              className="audio-batch-remove"
              role="group"
              aria-label="确认删除灯光片段"
            >
              <p>
                删除所选 {selection.items.length} 个片段
                {selection.hidden ? `（含筛选外 ${selection.hidden} 个）` : ""}
                ？保留场景和卡点，删除处留空；可以撤销。
              </p>
              <button disabled={blocked} onClick={() => onRun("remove")}>
                确认删除片段
              </button>
              <button ref={cancel} disabled={blocked} onClick={cancelRemoval}>
                取消删除
              </button>
            </div>
          )}
          <small>保留片段长度和相对间隔；每个片段的效果从新起点开始。</small>
        </div>
      )}
      {problem && (
        <p role="alert" className="audio-error">
          {problem}
        </p>
      )}
    </section>
  );
}
