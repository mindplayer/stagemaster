import { validWheelColor, type WheelAppearance } from "../../wheel-appearance";
import { WheelSwatch } from "./WheelSwatch";
export function ProfileWheelAppearance({
  value,
  field,
  label,
  onChange,
}: {
  value?: WheelAppearance;
  field: string;
  label: string;
  onChange(value: WheelAppearance | undefined): void;
}) {
  const kind = !value
    ? "unknown"
    : value.kind === "open"
      ? "open"
      : value.colors.length === 2
        ? "split"
        : "color";
  const colors = value?.kind === "color" ? value.colors : [];
  function changeColor(index: number, color: string) {
    onChange({
      kind: "color",
      colors: colors.map((c, i) => (i === index ? color : c)),
    });
  }
  return (
    <div className="profile-wheel-appearance">
      <WheelSwatch value={value} />
      <label>
        档位外观
        <select
          name={`${field}-appearance`}
          aria-label={`${label}外观`}
          value={kind}
          onChange={(e) =>
            onChange(
              e.target.value === "unknown"
                ? undefined
                : e.target.value === "open"
                  ? { kind: "open" }
                  : {
                      kind: "color",
                      colors:
                        e.target.value === "split"
                          ? [colors[0] ?? "#FFFFFF", colors[1] ?? "#FFFFFF"]
                          : [colors[0] ?? "#FFFFFF"],
                    },
            )
          }
        >
          <option value="unknown">未标记</option>
          <option value="open">通光</option>
          <option value="color">单色</option>
          <option value="split">半色</option>
        </select>
      </label>
      {colors.map((c, i) => (
        <div className="wheel-color-input" key={i}>
          <input
            type="color"
            aria-label={`${label}颜色 ${i + 1} 选色`}
            value={validWheelColor(c) ? c : "#000000"}
            onChange={(e) => changeColor(i, e.target.value)}
          />
          <label>
            颜色 {i + 1}
            <input
              type="text"
              name={`${field}-appearance-${i}`}
              aria-label={`${label}颜色 ${i + 1} 色值`}
              value={c}
              maxLength={7}
              placeholder="#RRGGBB"
              onChange={(e) => changeColor(i, e.target.value)}
            />
          </label>
        </div>
      ))}
      {value && (
        <button
          type="button"
          onClick={() => onChange(undefined)}
          aria-label={`清除${label}外观`}
        >
          清除标记
        </button>
      )}
    </div>
  );
}
