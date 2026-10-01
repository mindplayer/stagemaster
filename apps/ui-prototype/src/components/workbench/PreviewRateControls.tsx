import { useEffect, useId, useState } from "react";
import type { PreviewController } from "./usePreviewController";
import { previewRatePercent } from "../../preview-rate";
import "./preview-rate.css";

/** Controls the one loaded Rust player; draft input is not an engineering edit. */
export function PreviewRateControls({
  controller,
  disabled,
}: {
  controller: PreviewController;
  disabled: boolean;
}) {
  const { snapshot, working, act } = controller;
  const percent = snapshot.loaded?.ratePercent ?? 100;
  const [draft, setDraft] = useState(String(percent));
  const [error, setError] = useState("");
  const errorId = useId();
  useEffect(() => {
    setDraft(String(percent));
    setError("");
  }, [snapshot.epoch, percent]);
  const cancel = () => {
    setDraft(String(percent));
    setError("");
  };
  const blocked = disabled || working;
  const change = (value: string) => {
    if (blocked) return;
    try {
      const next = previewRatePercent(value);
      setDraft(String(next));
      setError("");
      void act({ kind: "setRate", percent: next });
    } catch (reason) {
      setError(String(reason instanceof Error ? reason.message : reason));
    }
  };
  if (!snapshot.loaded) return null;
  return (
    <section className="preview-rate" aria-label="预演速率">
      <header>
        <strong>预演速率</strong>
        <output aria-label="当前预演速率">{percent}%</output>
      </header>
      <div
        className="preview-rate-presets"
        role="group"
        aria-label="预演速率快捷值"
      >
        {[50, 100, 200].map((value) => (
          <button
            type="button"
            key={value}
            aria-pressed={percent === value}
            disabled={blocked}
            onClick={() => change(String(value))}
          >
            {value === 100 ? "正常 100%" : `${value}%`}
          </button>
        ))}
      </div>
      <div className="preview-rate-input">
        <input
          aria-label="预演速率百分比"
          inputMode="numeric"
          maxLength={3}
          value={draft}
          aria-invalid={!!error}
          aria-describedby={error ? errorId : undefined}
          disabled={blocked}
          onChange={(event) => {
            setDraft(event.target.value);
            setError("");
          }}
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              event.preventDefault();
              event.stopPropagation();
              cancel();
            }
            if (event.key === "Enter") {
              event.preventDefault();
              event.stopPropagation();
              change(draft);
            }
          }}
        />
        <span>%</span>
        <button
          type="button"
          disabled={blocked || draft === String(percent)}
          onClick={() => change(draft)}
        >
          应用速率
        </button>
        {draft !== String(percent) && (
          <button type="button" disabled={working} onClick={cancel}>
            取消输入
          </button>
        )}
      </div>
      {error && (
        <p id={errorId} role="alert">
          {error}
        </p>
      )}
      <small>按此倍率推进延时、渐变、等待和效果。音乐按原速。</small>
    </section>
  );
}
