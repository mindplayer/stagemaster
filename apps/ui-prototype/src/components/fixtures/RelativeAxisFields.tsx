import { useState } from "react";
import { positionDecimal } from "../../position-tools";
import "./relative-axis.css";

type Values = { panOffset: string; tiltOffset: string };
export function RelativeAxisFields({
  values,
  onChange,
}: {
  values: Values;
  onChange(patch: Partial<Values>): void;
}) {
  const [step, setStep] = useState("1");
  function valid(raw: string) {
    try {
      positionDecimal(raw || "0", "", "增量", 3600);
      return true;
    } catch {
      return false;
    }
  }
  return (
    <div className="relative-axis-fields">
      <p className="wb-dim">
        在每台灯本场景的基础角上增减，保留灯间差异。留空的轴保持；修改的预设引用转为本场景数值。
      </p>
      <label>
        微调步幅
        <select
          aria-label="轴微调步幅"
          value={step}
          onChange={(e) => setStep(e.target.value)}
        >
          <option value="1">1°</option>
          <option value="0.1">0.1°</option>
          <option value="0.01">0.01°</option>
        </select>
      </label>
      {(
        [
          ["panOffset", "水平"],
          ["tiltOffset", "垂直"],
        ] as const
      ).map(([key, label]) => {
        const raw = values[key];
        const canStep = valid(raw);
        const nudge = (direction: number) =>
          onChange({
            [key]: (Number(raw || "0") + direction * Number(step)).toFixed(3),
          });
        return (
          <div className="relative-axis-row" key={key}>
            <label>
              {label}增量（°）
              <input
                name={key}
                aria-label={`${label}增量（°）`}
                inputMode="decimal"
                value={raw}
                placeholder="留空保持"
                onChange={(e) => onChange({ [key]: e.target.value })}
              />
            </label>
            <div className="relative-axis-buttons">
              <button
                type="button"
                aria-label={`${label}减少 ${step} 度`}
                disabled={!canStep || Number(raw) - Number(step) < -3600}
                onClick={() => nudge(-1)}
              >
                −
              </button>
              <button
                type="button"
                aria-label={`${label}增加 ${step} 度`}
                disabled={!canStep || Number(raw) + Number(step) > 3600}
                onClick={() => nudge(1)}
              >
                ＋
              </button>
            </div>
          </div>
        );
      })}
      <p className="wb-dim">
        按钮只累计修改量，应用后记录。实际精度由灯具通道决定；有运动效果时先停用该效果。
      </p>
    </div>
  );
}
