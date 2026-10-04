import { parameterCategoryKeys } from "./parameter-categories.ts";
import { attributeBase } from "./fixture-emitters.ts";

export const presetAttributeScopes = [
  { id: "light", name: "仅亮度", keys: ["dimmer"] },
  { id: "color", name: "仅颜色", keys: parameterCategoryKeys("color") },
  { id: "position", name: "仅位置", keys: ["pan", "tilt"] },
  { id: "axisSpeed", name: "仅两轴速度控制", keys: ["pan-tilt-speed"] },
  { id: "program", name: "仅内置程序", keys: ["fixture-program"] },
  { id: "beam", name: "仅图案与棱镜", keys: parameterCategoryKeys("beam") },
  { id: "shutter", name: "仅快门与频闪", keys: ["shutter"] },
  { id: "optics", name: "仅镜头与光圈", keys: parameterCategoryKeys("optics") },
] as const;
export function presetScopeOptions(available: { key: string }[]) {
  const keys = [...new Set(available.map((a) => a.key))];
  return [
    { id: "all", name: "全部属性", keys },
    ...presetAttributeScopes.map((scope) => ({
      ...scope,
      keys: keys.filter((key) =>
        (scope.keys as readonly string[]).includes(attributeBase(key)),
      ),
    })),
  ];
}
export function presetScopeId(
  mask: string[] | null,
  available: { key: string }[],
): string {
  if (mask === null) return "all";
  const keys = new Set(available.map((a) => a.key));
  const selected = new Set(mask.filter((key) => keys.has(key)));
  if (!selected.size) return "custom";
  return (
    presetScopeOptions(available).find(
      (scope) =>
        scope.keys.length === selected.size &&
        scope.keys.every((key) => selected.has(key)),
    )?.id ?? "custom"
  );
}
