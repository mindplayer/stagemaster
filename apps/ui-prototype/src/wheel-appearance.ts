import { FixtureFieldError } from "./fixture-field-error.ts";
import { attributeBase } from "./fixture-emitter-keys.ts";
export type WheelAppearance =
  { kind: "open" } | { kind: "color"; colors: string[] };
export const validWheelColor = (color: string) =>
  /^#[\da-fA-F]{6}$/.test(color);
export function checkedAppearance(
  value: WheelAppearance | undefined,
  attribute: string,
  mode: string,
  field: string,
): WheelAppearance | undefined {
  if (!value) return undefined;
  if (attributeBase(attribute) !== "color-wheel" || mode !== "slot")
    throw new FixtureFieldError(
      `${field}-mode`,
      "外观只能用于色盘固定档位，请先清除标记",
    );
  if (value.kind === "open") return { kind: "open" };
  if (!value.colors.length || value.colors.length > 2)
    throw new FixtureFieldError(`${field}-appearance`, "请选择单色或半色外观");
  const colors = value.colors.map((c, index) => {
    const color = c.trim();
    if (!validWheelColor(color))
      throw new FixtureFieldError(
        `${field}-appearance-${index}`,
        "颜色应为 #RRGGBB，例如 #FF0000",
      );
    return color;
  });
  return { kind: "color", colors };
}
export function sameAppearance(a?: WheelAppearance, b?: WheelAppearance) {
  if (!a || !b) return a === b;
  if (a.kind !== b.kind) return false;
  if (a.kind === "open" || b.kind === "open") return true;
  return (
    a.colors.length === b.colors.length &&
    a.colors.every((c, i) => c.toLowerCase() === b.colors[i].toLowerCase())
  );
}
export function appearanceLabel(value?: WheelAppearance) {
  return !value
    ? "未标记"
    : value.kind === "open"
      ? "通光"
      : value.colors.length === 2
        ? "半色"
        : "单色";
}
