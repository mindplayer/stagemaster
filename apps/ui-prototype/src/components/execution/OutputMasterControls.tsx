import { useEffect, useRef, useState } from "react";
import type {
  ExecutionOutputAction,
  ExecutionView,
} from "../../execution-types";
import type { LiveLevels } from "../../execution-level-gesture";
import {
  outputAvailable,
  outputMasterKey,
  outputMasterTarget,
} from "../../execution-output-target";
import { levelPercent } from "../../execution-level-context";
import { useSourceLevelInput } from "./useSourceLevelInput";
import "./source-level.css";

export function OutputMasterControls({
  runtime,
  live,
  disabled,
  active,
  observed,
  onAction,
}: {
  runtime: ExecutionView;
  live: LiveLevels;
  disabled: boolean;
  active: boolean;
  observed: boolean;
  onAction(action: ExecutionOutputAction): void;
}) {
  const [draft, setDraft] = useState<string | null>(null);
  const precise = useRef<HTMLInputElement>(null);
  const output = runtime.observation.snapshot?.state.output;
  const supported = outputAvailable(runtime);
  const own = live.view.key === outputMasterKey;
  const enabled =
    active && observed && supported && draft === null && (!disabled || own);
  const latest = useRef(enabled);
  latest.current = enabled;
  const target = supported
    ? outputMasterTarget(runtime, () => latest.current)
    : null;
  const input = useSourceLevelInput(outputMasterKey, enabled, {
    ...live,
    begin: () => !!target && enabled && live.beginTarget(target),
  });
  const cancel = live.cancel;
  useEffect(() => {
    if (!enabled)
      cancel(outputMasterKey, "后台总控或操作状态已变化，请核对实际设定");
    return () => cancel(outputMasterKey, "后台总控已关闭，请核对实际设定");
  }, [enabled, cancel]);
  useEffect(() => {
    if (draft !== null && draft.trim() && Number(draft) === output?.percent)
      setDraft(null);
  }, [output?.percent]);
  const value = draft ?? String(output?.percent ?? 100);
  const valid =
    value.trim() !== "" &&
    Number.isInteger(Number(value)) &&
    Number(value) >= 0 &&
    Number(value) <= 100;
  const count = runtime.catalog.output?.uncontrolledFixtures ?? 0;
  return (
    <section
      className="execution-source execution-output-master"
      aria-label="后台输出总控"
    >
      <h3>后台输出总控</h3>
      {!supported ? (
        <p role="status">当前后台未提供有效输出总控；节目仍可按原能力观察。</p>
      ) : (
        <>
          <div className="source-level">
            <div className="source-level-reading">
              <span>总亮度</span>
              <output aria-label="后台实际总亮度">
                {observed ? "实际设定" : "最后已知设定"} {output!.percent}% ·{" "}
                {output!.blackout ? "熄灯已开启" : "未熄灯"}
              </output>
            </div>
            <input
              {...input}
              type="range"
              min="0"
              max="100"
              step="1"
              aria-label="后台总亮度推子"
              value={
                own
                  ? Math.round(levelPercent(live.view.target))
                  : output!.percent
              }
              aria-valuetext={`${own ? Math.round(levelPercent(live.view.target)) : output!.percent}%${output!.blackout ? "，熄灯已开启" : ""}`}
              disabled={!enabled || (own && live.view.phase === "cancelled")}
            />
            <div className="source-level-feedback" role="status">
              {own
                ? `目标 ${Math.round(levelPercent(live.view.target))}% · ${live.view.phase === "dragging" ? "连续调整中" : live.view.phase === "cancelled" ? "等待在途操作返回" : "正在确认最终值"}`
                : draft !== null
                  ? "先应用或取消精确输入，再拖动推子"
                  : output!.blackout || output!.percent === 0
                    ? "总控归零不会停止节目或音乐，也不释放手动值"
                    : "只影响后台已识别亮度，不联动编辑预演总控"}
            </div>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                if (!disabled && valid && draft !== null)
                  onAction({ kind: "level", percent: Number(value) });
              }}
            >
              <label>
                精确设置{" "}
                <input
                  ref={precise}
                  type="number"
                  min="0"
                  max="100"
                  step="1"
                  aria-label="后台总亮度精确值"
                  value={value}
                  disabled={disabled}
                  onChange={(e) => setDraft(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Escape") {
                      e.preventDefault();
                      e.stopPropagation();
                      setDraft(null);
                    }
                  }}
                />{" "}
                %
              </label>
              <button disabled={disabled || draft === null || !valid}>
                应用总亮度
              </button>
              {draft !== null && (
                <button
                  type="button"
                  onClick={() => {
                    setDraft(null);
                    precise.current?.focus();
                  }}
                >
                  取消总亮度输入
                </button>
              )}
            </form>
          </div>
          <div className="execution-buttons">
            <button
              disabled={disabled}
              aria-pressed={output!.blackout}
              onClick={() =>
                onAction({ kind: "blackout", enabled: !output!.blackout })
              }
            >
              {output!.blackout ? "解除后台熄灯" : "后台熄灯"}
            </button>
            {own && (
              <button
                disabled={live.view.phase === "cancelled"}
                onClick={() => cancel(outputMasterKey)}
              >
                停止总亮度调整
              </button>
            )}
          </div>
          {count > 0 && (
            <p role="status" className="wb-preview-warning">
              {count} 台灯具未识别连续亮度，不能保证受总控或熄灯影响。
            </p>
          )}
          <p className="wb-dim">
            熄灯不是停止节目、释放手动层或机械急停。归还控制权不重置总控；这里只报告软件输出。
          </p>
        </>
      )}
    </section>
  );
}
