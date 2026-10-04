import { useEffect, useState } from "react";
import type { ExecutionAction } from "../../execution-types";
import type { LiveLevels } from "../../execution-level-gesture";
import { levelPercent } from "../../execution-level-context";
import { useSourceLevelInput } from "./useSourceLevelInput";
import "./source-level.css";

export function SourceLevelControls({
  id,
  name,
  level,
  disabled,
  active,
  observed,
  live,
  onDraftChange,
  onAction,
}: {
  id: string;
  name: string;
  level: number | undefined;
  disabled: boolean;
  active: boolean;
  observed: boolean;
  live?: LiveLevels;
  onDraftChange?(id: string, dirty: boolean): void;
  onAction(action: ExecutionAction): void;
}) {
  const [draft, setDraft] = useState<string | null>(null);
  const own = live?.view.key === id;
  const input = useSourceLevelInput(
    id,
    draft === null && active && observed,
    live,
  );
  const editable = own && live?.view.phase !== "cancelled";
  const value = draft ?? String(levelPercent(level ?? 65535));
  const target = own ? live!.view.target : (level ?? 65535);
  const valid =
    value.trim() !== "" &&
    Number.isFinite(Number(value)) &&
    Number(value) >= 0 &&
    Number(value) <= 100;
  const cancel = live?.cancel;
  useEffect(() => {
    if (draft !== null && Math.round(Number(draft) * 655.35) === level)
      setDraft(null);
  }, [level]);
  useEffect(() => {
    onDraftChange?.(id, draft !== null);
  }, [draft, id, onDraftChange]);
  useEffect(() => {
    if (!active) cancel?.(id, "节目已隐藏，已停止连续调整，请核对实际电平");
  }, [active, cancel, id]);
  useEffect(
    () => () => cancel?.(id, "节目控件已关闭，请核对实际电平"),
    [cancel, id],
  );
  return (
    <div className="source-level">
      {live && (
        <>
          <div className="source-level-reading">
            <span>亮度电平</span>
            <output aria-label={`${name}实际电平`}>
              {observed ? "实际" : "最后已知"}{" "}
              {level === undefined ? "—" : `${levelPercent(level)}%`}
            </output>
          </div>
          <input
            {...input}
            type="range"
            min="0"
            max="100"
            step="0.1"
            aria-label={`${name}亮度推子`}
            aria-valuetext={`${levelPercent(target)}%${own ? "，正在调整" : ""}`}
            value={levelPercent(target)}
            disabled={
              !active ||
              !observed ||
              draft !== null ||
              (!editable && (disabled || !!live.view.key))
            }
          />
          <div className="source-level-feedback">
            {own ? (
              <span role="status">
                目标 {levelPercent(target)}% ·{" "}
                {live.view.phase === "dragging"
                  ? "连续调整中"
                  : live.view.phase === "cancelled"
                    ? "等待在途操作返回"
                    : "正在确认最终值"}
              </span>
            ) : draft !== null ? (
              <span>先应用或取消数值输入，再拖动推子</span>
            ) : level === 0 ? (
              <span>电平为零 · 未停止或释放节目</span>
            ) : (
              <span>拖动实时调光</span>
            )}
          </div>
        </>
      )}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (valid && draft !== null && !disabled)
            onAction({
              kind: "level",
              value: Math.round(Number(value) * 655.35),
            });
        }}
      >
        <label>
          {live ? "精确设置" : "亮度电平"}{" "}
          <input
            aria-label={`${name}亮度电平`}
            type="number"
            min="0"
            max="100"
            step="0.1"
            value={value}
            disabled={disabled}
            onChange={(e) => setDraft(e.target.value)}
          />{" "}
          %
        </label>
        <button disabled={disabled || draft === null || !valid}>应用</button>
        {draft !== null && (
          <button type="button" onClick={() => setDraft(null)}>
            取消
          </button>
        )}
      </form>
    </div>
  );
}
