import { useEffect, useRef, useState } from "react";
import type { AudioLightingClip } from "../../audio-types";
import { AudioClipEntryFade } from "./AudioClipEntryFade";
import {
  clipSplitLimits,
  splitClipCommand,
  type ClipSplitActions,
} from "./clip-split-tools";
export function AudioClipSplit({
  clip,
  disabled,
  ready,
  actions,
}: {
  clip: AudioLightingClip;
  disabled: boolean;
  ready: boolean;
  actions: ClipSplitActions;
}) {
  const input = useRef<HTMLInputElement>(null);
  const focusPending = useRef(false);
  const failureFocus = useRef<HTMLElement | null>(null);
  const limits = clipSplitLimits(clip);
  const initial = (Math.floor((limits.min + limits.max) / 2) / 1000).toFixed(3);
  const [value, setValue] = useState(initial),
    [problem, setProblem] = useState(""),
    [working, setWorking] = useState(false);
  const blocked = disabled || working || clip.locked;
  useEffect(() => {
    if (!working && focusPending.current) {
      focusPending.current = false;
      (failureFocus.current?.isConnected
        ? failureFocus.current
        : input.current
      )?.focus();
    }
  }, [working, problem, value]);
  async function run(kind: "split" | "position" | "reset" | "resetEntry") {
    if (blocked) return;
    setProblem("");
    failureFocus.current =
      kind === "resetEntry" && document.activeElement instanceof HTMLElement
        ? document.activeElement
        : input.current;
    setWorking(true);
    try {
      if (kind === "position") {
        setValue(((await actions.readPosition()) / 1000).toFixed(3));
        focusPending.current = true;
      } else if (kind === "resetEntry") {
        if (!(await actions.resetEntryFade()))
          throw new Error("未能重新计算渐变，请检查工程提示");
      } else if (kind === "reset") {
        if (!(await actions.resetOffset()))
          throw new Error("未能重置效果起点，请检查工程提示");
      } else {
        const command = splitClipCommand(clip, value);
        if (
          command.kind === "splitLightingClip" &&
          !(await actions.split(command.timeMs))
        )
          throw new Error("未能分割，请检查工程提示后调整位置");
      }
    } catch (error) {
      setProblem(error instanceof Error ? error.message : String(error));
      focusPending.current = true;
    } finally {
      setWorking(false);
    }
  }
  return (
    <section className="audio-clip-split" aria-label="片段分割与效果起点">
      {clip.entryFade && (
        <AudioClipEntryFade
          fade={clip.entryFade}
          disabled={blocked}
          onReset={() => void run("resetEntry")}
        />
      )}
      <p>效果起点：{((clip.effectOffsetMs ?? 0) / 1000).toFixed(3)} 秒</p>
      {!!clip.effectOffsetMs && (
        <button
          type="button"
          disabled={blocked}
          onClick={() => void run("reset")}
        >
          重置效果起点
        </button>
      )}
      {problem && <p role="alert">{problem}</p>}
      <details>
        <summary>分割片段</summary>
        <label>
          分割位置（秒）
          <input
            ref={input}
            aria-label="片段分割位置（秒）"
            inputMode="decimal"
            value={value}
            disabled={blocked || limits.min > limits.max}
            onChange={(e) => {
              setValue(e.target.value);
              setProblem("");
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                e.stopPropagation();
                void run("split");
              }
              if (e.key === "Escape") {
                e.preventDefault();
                e.stopPropagation();
                setValue(initial);
                setProblem("");
              }
            }}
          />
        </label>
        <small>
          {limits.min > limits.max
            ? "片段不足 2 毫秒，没有可分割的位置。"
            : `可分割范围 ${(limits.min / 1000).toFixed(3)} — ${(limits.max / 1000).toFixed(3)} 秒；保留原渐变与动态效果进度。`}
        </small>
        <div className="wb-actions">
          <button
            type="button"
            disabled={blocked || !ready || limits.min > limits.max}
            onClick={() => void run("position")}
          >
            使用播放头位置
          </button>
          <button
            type="button"
            disabled={blocked || limits.min > limits.max}
            onClick={() => void run("split")}
          >
            确认分割
          </button>
          <button
            type="button"
            disabled={working || (value === initial && !problem)}
            onClick={() => {
              setValue(initial);
              setProblem("");
            }}
          >
            取消分割输入
          </button>
        </div>
      </details>
    </section>
  );
}
