import type { FixtureView } from "./application-host";

export type FixtureOrder = "reverse" | "oddFirst" | "name" | "patch";
const collator = new Intl.Collator("zh-CN", {
  numeric: true,
  sensitivity: "base",
});

/** Change only explicit authoring order; retain every member and stable ties. */
export function arrangeFixtureIds(
  ids: string[],
  fixtures: FixtureView[],
  order: FixtureOrder,
): string[] {
  if (order === "reverse") return [...ids].reverse();
  if (order === "oddFirst")
    return [
      ...ids.filter((_, i) => i % 2 === 0),
      ...ids.filter((_, i) => i % 2 === 1),
    ];
  const byId = new Map(fixtures.map((f) => [f.id, f]));
  return [...ids].sort((a, b) => {
    const left = byId.get(a),
      right = byId.get(b);
    if (!left || !right) return Number(!left) - Number(!right);
    if (order === "name") return collator.compare(left.name, right.name);
    const unpatched = (f: FixtureView) =>
      f.universe === null || f.address === null;
    const missing = Number(unpatched(left)) - Number(unpatched(right));
    if (missing) return missing;
    if (unpatched(left)) return 0;
    return (
      collator.compare(left.domainName, right.domainName) ||
      collator.compare(left.domainId, right.domainId) ||
      left.universe! - right.universe! ||
      left.address! - right.address!
    );
  });
}
