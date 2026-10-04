import { useEffect, useRef, useState } from "react";
import type { ExecutionView } from "../../execution-types";
import type { LiveLevels } from "../../execution-level-gesture";
import {
  manualBrightnessKey,
  manualBrightnessReading,
  manualBrightnessTarget,
} from "../../manual-brightness-target";
import { levelPercent } from "../../execution-level-context";
import { useSourceLevelInput } from "./useSourceLevelInput";
import "./source-level.css";

export function ManualBrightnessControls({
  runtime,
  source,
  selected,
  live,
  disabled,
  active,
  observed,
  pending,
}: {
  runtime: ExecutionView;
  source: string;
  selected: string[];
  live: LiveLevels;
  disabled: boolean;
  active: boolean;
  observed: boolean;
  pending: boolean;
}) {
  const key = manualBrightnessKey(source, selected);
  const own = live.view.key === key;
  const enabled = active && observed && !pending && (!disabled || own);
  const latest = useRef({ key, enabled });
  latest.current = { key, enabled };
  const mounted = useRef(true);
  const [problem, setProblem] = useState("");
  const reading = manualBrightnessReading(runtime, source, selected);
  let target: ReturnType<typeof manualBrightnessTarget> | undefined;
  let unsupported = "";
  try {
    target = manualBrightnessTarget(
      runtime,
      source,
      selected,
      () =>
        mounted.current && latest.current.key === key && latest.current.enabled,
    );
  } catch (error) {
    unsupported = error instanceof Error ? error.message : String(error);
  }
  const value = own
    ? live.view.target
    : ((reading?.unheld === 0 ? reading.raw : undefined) ?? 32768);
  const actual = !reading
    ? "数值不可用"
    : reading.unheld === selected.length
      ? "未持有"
      : `${reading.label}${reading.unheld ? ` · ${reading.unheld} 台未持有` : ""}`;
  const input = useSourceLevelInput(key, enabled && !!target, {
    ...live,
    begin: () => {
      if (!target || !enabled) return false;
      const accepted = live.beginTarget(target);
      if (!accepted) setProblem("当前不能开始连续调整，请核对控制状态");
      else setProblem("");
      return accepted;
    },
  });
  const cancel = live.cancel;
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  useEffect(() => {
    if (!enabled) cancel(key, "手动目标或操作状态已变化，请核对实际亮度");
    return () => cancel(key, "共同亮度控件已关闭，请核对实际设定值");
  }, [key, enabled, cancel]);
  return (
    <div className="source-level" aria-label="所选灯组共同亮度">
      <div className="source-level-reading">
        <span>共同亮度 · {selected.length} 台灯</span>
        <output aria-label="所选灯组实际亮度">
          {observed ? "实际设定" : "最后已知设定"} · {actual}
        </output>
      </div>
      <input
        {...input}
        type="range"
        min="0"
        max="100"
        step="0.1"
        aria-label="所选灯组共同亮度推子"
        aria-valuetext={own ? `目标 ${levelPercent(value)}%` : actual}
        value={levelPercent(value)}
        disabled={
          !enabled || !target || (own && live.view.phase === "cancelled")
        }
      />
      <div className="source-level-feedback" role="status">
        {unsupported ||
          problem ||
          (own
            ? `目标 ${levelPercent(value)}% · ${live.view.phase === "dragging" ? "整组连续调整中" : live.view.phase === "cancelled" ? "等待在途操作返回" : "正在确认整组最终值"}`
            : pending
              ? "先应用或取消精确输入，再拖动推子"
              : reading?.unheld || reading?.raw === undefined
                ? "整组尚无一致设定；移动后才应用同一亮度"
                : "拖动设置整组亮度；零值仍持有，释放另行操作")}
      </div>
      {own && (
        <button
          disabled={live.view.phase === "cancelled"}
          onClick={() => cancel(key)}
        >
          停止连续调整
        </button>
      )}
    </div>
  );
}
