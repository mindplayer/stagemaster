import type { FixtureView } from "./application-host";

type Attribute = FixtureView["attributes"][number];
export const parameterCategories = [
  { id: "intensity", label: "亮度", keys: ["dimmer", "shutter"] },
  { id: "color", label: "颜色", keys: ["red", "green", "blue", "color-wheel"] },
  { id: "beam", label: "图案", keys: ["gobo-wheel", "prism"] },
  { id: "optics", label: "镜头", keys: ["zoom", "focus", "iris"] },
  { id: "other", label: "其他", keys: [] },
] as const;
export type ParameterCategory = (typeof parameterCategories)[number]["id"];
export function parameterCategoryKeys(
  id: ParameterCategory,
): readonly string[] {
  return parameterCategories.find((c) => c.id === id)!.keys;
}
export function parameterCategory(key: string): ParameterCategory {
  return (
    parameterCategories.find((c) => (c.keys as readonly string[]).includes(key))
      ?.id ?? "other"
  );
}
export function availableParameterCategories(attributes: Attribute[]) {
  return parameterCategories
    .map((c) => ({
      ...c,
      count: attributes.filter((a) => parameterCategory(a.key) === c.id).length,
    }))
    .filter((c) => c.count > 0);
}
/** Canonical presentation order is independent of physical channel order. */
export function visibleParameterAttributes(
  attributes: Attribute[],
  category: string,
) {
  const keys: readonly string[] = parameterCategories.flatMap((c) => [
    ...c.keys,
  ]);
  const rank = (key: string) =>
    keys.includes(key) ? keys.indexOf(key) : keys.length;
  return attributes
    .filter((a) => category === "all" || parameterCategory(a.key) === category)
    .sort((a, b) => rank(a.key) - rank(b.key));
}
