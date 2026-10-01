import type { PresetView } from "./library-types";

/** An explicit empty scope is different from an unspecified/default scope. */
export function initialPresetAttributes(
  available: { key: string }[],
  mask: string[] | null,
  preset?: PresetView,
): string[] {
  const keys = [...new Set(available.map((a) => a.key))];
  if (mask !== null) return keys.filter((key) => mask.includes(key));
  return preset
    ? keys.filter((key) => preset.values.some((v) => v.attribute === key))
    : keys;
}
