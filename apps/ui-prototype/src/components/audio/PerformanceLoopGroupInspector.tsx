import { useEffect, useRef } from "react";
import { audioTime } from "../../audio-tools";
import type { AudioLoopRegion } from "../../audio-performance-types";
import { loopGroupSelection } from "./performance-loop-group";
import type { usePerformanceLoopBatch } from "./usePerformanceLoopBatch";
export function PerformanceLoopGroupInspector({
  regions,
  ids,
  visibleItems,
  actions,
  busy,
  visible,
}: {
  regions: AudioLoopRegion[];
  ids: string[];
  visibleItems: AudioLoopRegion[];
  actions: ReturnType<typeof usePerformanceLoopBatch>;
  busy: boolean;
  visible: boolean;
}) {
  const group = loopGroupSelection(regions, ids, visibleItems);
  const target = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (visible && actions.problem) target.current?.focus();
  }, [visible, actions.problem]);
  const blocked = busy || actions.working || !group.items.length;
  return (
    <div className="performance-loop-properties">
      <h2>循环区段成组编辑 · {group.items.length}</h2>
      {!!group.hidden && (
        <p role="status">含 {group.hidden} 个筛选范围外的所选区段</p>
      )}
      {!!group.locked && (
        <p>
          含 {group.locked}{" "}
          个锁定区段；移动、删除、启停不可用，复制会创建未锁定的新区段。
        </p>
      )}
      {!!group.items.length && (
        <p>
          {audioTime(group.first)} — {audioTime(group.last)} · {group.inactive}{" "}
          个已停用
        </p>
      )}
      <label>
        区段组目标起点（秒）
        <input
          ref={target}
          aria-label="区段组目标起点（秒）"
          value={actions.destination ?? (group.first / 1000).toFixed(3)}
          disabled={blocked || actions.removing}
          onChange={(e) => actions.setDestination(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              e.preventDefault();
              actions.cancel();
            }
          }}
        />
      </label>
      <small>
        保持相对间隔。移动／复制一次提交整组，重叠或越界时全部拒绝。
      </small>
      {actions.problem && (
        <p role="alert" className="audio-error">
          {actions.problem}
        </p>
      )}
      <div className="performance-loop-actions">
        <button
          disabled={blocked || !!group.locked || actions.removing}
          onClick={() =>
            void actions.run(
              "move",
              actions.destination ?? (group.first / 1000).toFixed(3),
            )
          }
        >
          移动区段组
        </button>
        <button
          disabled={blocked || actions.removing}
          onClick={() =>
            void actions.run(
              "copy",
              actions.destination ?? (group.first / 1000).toFixed(3),
            )
          }
        >
          复制区段组
        </button>
        {(["enable", "disable", "lock", "unlock"] as const).map((kind, i) => (
          <button
            key={kind}
            disabled={blocked || actions.pending || (i < 2 && !!group.locked)}
            onClick={() => void actions.run(kind)}
          >
            {["启用区段组", "停用区段组", "锁定区段组", "解锁区段组"][i]}
          </button>
        ))}
        <button
          disabled={blocked || actions.pending || !!group.locked}
          onClick={() => actions.setRemoving(true)}
        >
          删除区段组
        </button>
        {actions.pending && (
          <button disabled={busy || actions.working} onClick={actions.cancel}>
            取消区段组输入
          </button>
        )}
      </div>
      {actions.removing && (
        <section role="alertdialog" aria-label="确认删除循环区段组">
          <p>
            删除 {group.items.length}{" "}
            个循环区段？音乐、卡点和灯光片段保持，可一次撤销。
          </p>
          <button disabled={blocked} onClick={() => void actions.run("remove")}>
            确认删除区段组
          </button>
          <button disabled={busy || actions.working} onClick={actions.cancel}>
            取消删除区段组
          </button>
        </section>
      )}
    </div>
  );
}
