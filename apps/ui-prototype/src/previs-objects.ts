import type { StageSelection } from "./stage-types.ts";
export type PrevisTarget = { kind: "placement" | "construction"; id: string };
export function readPrevisTargets(
  value: unknown,
  max = 1024,
): PrevisTarget[] | null {
  if (!Array.isArray(value) || value.length > max) return null;
  const seen = new Set<string>();
  for (const v of value) {
    if (
      !v ||
      typeof v !== "object" ||
      Array.isArray(v) ||
      Object.keys(v).length !== 2 ||
      !["placement", "construction"].includes(v.kind) ||
      typeof v.id !== "string" ||
      !v.id ||
      v.id.length > 256
    )
      return null;
    const key = `${v.kind}:${v.id}`;
    if (seen.has(key)) return null;
    seen.add(key);
  }
  return value as PrevisTarget[];
}
export function viewportTargets(targets: StageSelection[]): PrevisTarget[] {
  // Never show only part of a mixed selection containing an unsupported space.
  return readPrevisTargets(targets) ?? [];
}
export function sameTargets(a: PrevisTarget[], b: PrevisTarget[]) {
  return (
    a.length === b.length &&
    a.every((v, i) => v.id === b[i].id && v.kind === b[i].kind)
  );
}
export const fixtureTargets = (ids: string[]): PrevisTarget[] =>
  ids.map((id) => ({ kind: "placement", id }));
