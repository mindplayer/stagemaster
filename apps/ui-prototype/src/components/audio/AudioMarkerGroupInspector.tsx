import { useEffect, useRef } from "react";
import { audioTime } from "../../audio-tools";
import type { markerSelection } from "../../audio-group-tools";
export function AudioMarkerGroupInspector({
  selection,
  value,
  blocked,
  removing,
  problem,
  pending,
  visible,
  onValue,
  onReset,
  onRemove,
  onRun,
}: {
  selection: ReturnType<typeof markerSelection>;
  value: string;
  blocked: boolean;
  removing: boolean;
  problem: string;
  pending: boolean;
  visible: boolean;
  onValue(value: string): void;
  onReset(): void;
  onRemove(value: boolean): void;
  onRun(kind: "move" | "copy" | "remove"): void;
}) {
  const target = useRef<HTMLInputElement>(null);
  const deleteButton = useRef<HTMLButtonElement>(null);
  const cancelButton = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (removing && visible) cancelButton.current?.focus();
  }, [removing, visible]);
  useEffect(() => {
    if (problem && visible) target.current?.focus();
  }, [problem, visible]);
  function cancelRemoval() {
    onRemove(false);
    deleteButton.current?.focus();
  }
  return (
    <section
      className="audio-marker-batch audio-batch-inspector"
      aria-label="卡点组属性"
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          e.stopPropagation();
          if (removing) cancelRemoval();
          else onReset();
        }
      }}
    >
      <h3>卡点组属性</h3>
      <p>
        已选 {selection.markers.length} 个
        {selection.hidden > 0 ? `，其中 ${selection.hidden} 个在筛选外` : ""}
      </p>
      {!selection.markers.length && (
        <p>在目录或波形上选择卡点，Shift 追加连续范围。</p>
      )}
      {!!selection.markers.length && (
        <div className="audio-batch-operations">
          <p>
            原范围 {audioTime(selection.first)} — {audioTime(selection.last)}
          </p>
          <label>
            目标起点（秒）
            <input
              ref={target}
              name="groupDestination"
              aria-label="卡点组目标起点（秒）"
              inputMode="decimal"
              value={value}
              disabled={blocked}
              onChange={(e) => onValue(e.target.value)}
            />
          </label>
          <div className="audio-batch-selection">
            <button disabled={blocked} onClick={() => onRun("move")}>
              移动所选
            </button>
            <button disabled={blocked} onClick={() => onRun("copy")}>
              复制所选
            </button>
            <button
              ref={deleteButton}
              disabled={blocked}
              onClick={() => onRemove(true)}
            >
              删除所选
            </button>
            {pending && (
              <button disabled={blocked} onClick={onReset}>
                取消目标输入
              </button>
            )}
          </div>
          {removing && (
            <div
              className="audio-batch-remove"
              role="group"
              aria-label="确认删除卡点"
            >
              <p>
                删除已选 {selection.markers.length} 个卡点
                {selection.hidden ? `（含筛选外 ${selection.hidden} 个）` : ""}
                ？灯光场景保留，可以撤销。
              </p>
              <button disabled={blocked} onClick={() => onRun("remove")}>
                确认删除卡点
              </button>
              <button
                ref={cancelButton}
                disabled={blocked}
                onClick={cancelRemoval}
              >
                取消删除
              </button>
            </div>
          )}
          <small>
            保留相对间隔，场景引用和渐变随卡点调整；灯光区间由相邻切换点决定。
          </small>
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
