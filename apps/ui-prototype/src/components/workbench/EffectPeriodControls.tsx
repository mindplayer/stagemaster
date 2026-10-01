import { useEffect, useState } from "react";
import { scaledEffectPeriod } from "../../effect-tempo";
import { seconds } from "../../sequence-tools";
import { EffectBeatControls } from "./EffectBeatControls";
export function EffectPeriodControls({
  value,
  onChange,
}: {
  value: string;
  onChange(value: string): void;
}) {
  const [error, setError] = useState("");
  useEffect(() => setError(""), [value]);
  function change(next: string) {
    setError("");
    if (next !== value) onChange(next);
  }
  return (
    <section
      className="effect-period-controls"
      aria-label="效果周期"
      data-editor-navigation="true"
    >
      <label>
        循环周期 · 秒
        <input
          type="number"
          required
          min={0.1}
          max={3600}
          step={0.001}
          inputMode="decimal"
          aria-label="循环周期"
          value={value}
          onChange={(e) => change(e.target.value)}
        />
      </label>
      <div className="effect-speed-actions">
        {(
          [
            [2, "半速"],
            [0.5, "倍速"],
          ] as const
        ).map(([factor, label]) => (
          <button
            type="button"
            key={label}
            onClick={() => {
              try {
                change(seconds(scaledEffectPeriod(value, factor)));
              } catch (reason) {
                setError(
                  reason instanceof Error ? reason.message : String(reason),
                );
              }
            }}
          >
            {label}
          </button>
        ))}
      </div>
      {error && (
        <p className="wb-library-error" role="alert">
          {error}
        </p>
      )}
      <EffectBeatControls onPeriod={change} />
    </section>
  );
}
