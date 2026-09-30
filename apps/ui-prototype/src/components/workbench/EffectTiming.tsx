import type { SceneEffect } from "../../effect-types";
export interface EffectTimingDraft {
  spread: string;
  phase: string;
  duty: string;
}
export function EffectTiming({
  waveform,
  timing,
  onChange,
}: {
  waveform: SceneEffect["waveform"];
  timing: EffectTimingDraft;
  onChange(value: EffectTimingDraft): void;
}) {
  return (
    <>
      <div className="effect-fields">
        <label>
          灯间展开 · 度
          <input
            type="number"
            required
            min={0}
            max={360}
            step={1}
            aria-label="灯间展开"
            value={timing.spread}
            onChange={(e) => onChange({ ...timing, spread: e.target.value })}
          />
        </label>
        {waveform === "pulse" ? (
          <label>
            亮段比例 · %
            <input
              type="number"
              required
              min={1}
              max={99}
              step={1}
              aria-label="亮段比例"
              value={timing.duty}
              onChange={(e) => onChange({ ...timing, duty: e.target.value })}
            />
          </label>
        ) : (
          <label>
            起始相位 · 度
            <input
              type="number"
              required
              min={0}
              max={359}
              step={1}
              aria-label="起始相位"
              value={timing.phase}
              onChange={(e) => onChange({ ...timing, phase: e.target.value })}
            />
          </label>
        )}
      </div>
      {waveform === "pulse" && (
        <label>
          起始相位 · 度
          <input
            type="number"
            required
            min={0}
            max={359}
            step={1}
            aria-label="起始相位"
            value={timing.phase}
            onChange={(e) => onChange({ ...timing, phase: e.target.value })}
          />
        </label>
      )}
    </>
  );
}
