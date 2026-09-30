import { planLayers, ALL_VISIBLE, type PlanVisibility } from "./stage-display";
import "./stage-organization.css";
export function StagePlanLayers({
  value,
  disabled,
  onChange,
}: {
  value: PlanVisibility;
  disabled: boolean;
  onChange(value: PlanVisibility): void;
}) {
  const hidden = value.hiddenLayers.length + value.hiddenSpaces.length;
  return (
    <section className="stage-plan-layers" aria-label="平面显示">
      <header>
        <strong>平面显示</strong>
        <button
          disabled={disabled || !hidden}
          onClick={() => onChange(ALL_VISIBLE)}
        >
          全部显示
        </button>
      </header>
      <div>
        {planLayers.map(([key, label]) => (
          <button
            key={key}
            disabled={disabled}
            aria-pressed={!value.hiddenLayers.includes(key)}
            title={`在平面图中显示或隐藏${label}`}
            onClick={() =>
              onChange({
                ...value,
                hiddenLayers: value.hiddenLayers.includes(key)
                  ? value.hiddenLayers.filter((k) => k !== key)
                  : [...value.hiddenLayers, key],
              })
            }
          >
            {label}
          </button>
        ))}
      </div>
      {!!hidden && <small>{hidden} 项隐藏 · 节目和三维保持不变</small>}
    </section>
  );
}
