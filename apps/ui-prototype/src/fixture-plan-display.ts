import type { FixtureView } from "./application-host.ts";
export type PlanLabelMode = "none" | "name" | "address";
export interface FixtureSymbolInfo {
  moving: boolean;
  rgb: boolean;
  wheel: boolean;
  key: string;
  label: string;
}
/** Capabilities, not guessed photometry, names or physical housing. */
export function fixtureSymbol(
  fixture?: Pick<FixtureView, "attributes" | "positioning">,
): FixtureSymbolInfo {
  const keys = new Set(fixture?.attributes.map((a) => a.key) ?? []);
  const moving =
    !!fixture?.positioning || (keys.has("pan") && keys.has("tilt"));
  const rgb = ["red", "green", "blue"].every((k) => keys.has(k));
  const wheel = keys.has("color-wheel");
  return {
    moving,
    rgb,
    wheel,
    key: `${+moving}${+rgb}${+wheel}`,
    label: [
      moving ? "双轴灯具" : "通用灯具",
      ...(rgb ? ["混色"] : []),
      ...(wheel ? ["色盘"] : []),
    ].join(" · "),
  };
}
export function fixtureAddress(
  f?: Pick<FixtureView, "universe" | "address">,
): string {
  return f?.universe != null && f.address != null
    ? `${f.universe}.${String(f.address).padStart(3, "0")}`
    : "未配适";
}
export function fixturePlanLabel(
  f: FixtureView | undefined,
  mode: PlanLabelMode,
): string {
  return mode === "none"
    ? ""
    : mode === "address"
      ? fixtureAddress(f)
      : (f?.name ?? "未知灯具");
}
export function fixtureLegend(fixtures: FixtureView[], ids: string[]) {
  const visible = new Set(ids),
    entries = new Map<string, { symbol: FixtureSymbolInfo; count: number }>();
  for (const f of fixtures) {
    if (!visible.has(f.id)) continue;
    const symbol = fixtureSymbol(f),
      existing = entries.get(symbol.key);
    if (existing) existing.count++;
    else entries.set(symbol.key, { symbol, count: 1 });
  }
  return [...entries.values()].sort((a, b) =>
    b.symbol.key.localeCompare(a.symbol.key),
  );
}
