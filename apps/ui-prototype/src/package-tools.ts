import type {
  PackageCandidate,
  PackageResult,
  PackageSelection,
} from "./package-types.ts";
export const packageKey = (item: PackageSelection) => `${item.kind}:${item.id}`;
export const selectionKey = (items: PackageSelection[]) =>
  items.map(packageKey).sort().join("|");
export function filterPrograms(
  items: PackageCandidate[],
  query: string,
  kind: "all" | PackageSelection["kind"],
) {
  const term = query.trim().toLocaleLowerCase();
  return items.filter(
    (p) =>
      (kind === "all" || p.kind === kind) &&
      p.name.toLocaleLowerCase().includes(term),
  );
}
export function packageCurrent(
  result: PackageResult | null,
  projectId: string,
  generation: number,
  hasDrafts: boolean,
  builtSelection: string,
  selected: PackageSelection[],
) {
  return (
    !!result &&
    result.generation === generation &&
    !hasDrafts &&
    (!result.report || result.report.projectId === projectId) &&
    builtSelection === selectionKey(selected)
  );
}
export function selectFiltered(
  selected: PackageSelection[],
  filtered: PackageCandidate[],
): PackageSelection[] | null {
  const values = new Map(selected.map((p) => [packageKey(p), p]));
  for (const { kind, id } of filtered)
    values.set(packageKey({ kind, id }), { kind, id });
  return values.size > 64 ? null : [...values.values()];
}
