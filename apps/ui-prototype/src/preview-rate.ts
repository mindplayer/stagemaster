/** Input conversion only. Rust owns bounds, transport time and command ordering. */
export function previewRatePercent(value: string): number {
  const text = value.trim();
  const percent = Number(text);
  if (!/^\d{1,3}$/.test(text) || percent < 25 || percent > 400)
    throw new Error("请输入 25—400 之间的整数百分比");
  return percent;
}
