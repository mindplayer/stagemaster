import {
  appearanceLabel,
  validWheelColor,
  type WheelAppearance,
} from "../../wheel-appearance";
import "./wheel-appearance.css";
export function WheelSwatch({ value }: { value?: WheelAppearance }) {
  const colors =
    value?.kind === "color" && value.colors.every(validWheelColor)
      ? value.colors
      : [];
  return (
    <span
      className={`wheel-swatch ${value?.kind === "open" ? "is-open" : ""}`}
      role="img"
      aria-label={appearanceLabel(value)}
      title={appearanceLabel(value)}
    >
      {colors.length ? (
        colors.map((c, i) => <i key={i} style={{ backgroundColor: c }} />)
      ) : (
        <span>{value?.kind === "open" ? "○" : "?"}</span>
      )}
    </span>
  );
}
