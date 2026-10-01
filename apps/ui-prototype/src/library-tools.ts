import { functionLabels } from "./fixture-function-types.ts";
import type { FixtureView, SceneView } from "./application-host";
import type { PresetView } from "./library-types";
export type RecallMode = "replace" | "add" | "subtract";
export function recallGroup(
  selected: string[],
  members: string[],
  mode: RecallMode,
): string[] {
  if (mode === "replace") return [...members];
  if (mode === "subtract")
    return selected.filter((id) => !members.includes(id));
  return [...new Set([...selected, ...members])];
}
export function moveMember(
  ids: string[],
  index: number,
  delta: number,
): string[] {
  if (
    index < 0 ||
    index >= ids.length ||
    index + delta < 0 ||
    index + delta >= ids.length
  )
    return ids;
  const next = [...ids];
  const [id] = next.splice(index, 1);
  next.splice(index + delta, 0, id);
  return next;
}
export function scopeAttributes(fixtures: FixtureView[]) {
  return [
    ...new Map(
      fixtures.flatMap((f) => f.attributes).map((a) => [a.key, a]),
    ).values(),
  ];
}
export function matchingValues(
  values: SceneView["values"],
  ids: string[],
  attributes: string[],
) {
  return values.filter(
    (v) =>
      ids.includes(v.fixtureId) &&
      attributes.includes(v.attribute) &&
      v.value !== null &&
      v.mode !== "release",
  );
}
export function presetCoverage(
  preset: PresetView,
  ids: string[],
  attributes: string[],
) {
  const values = matchingValues(preset.values, ids, attributes);
  return {
    fixtures: new Set(values.map((v) => v.fixtureId)).size,
    attributes: values.length,
  };
}
export const attributeName = (key: string) =>
  ({
    ...functionLabels,
    dimmer: "亮度",
    red: "红色",
    green: "绿色",
    blue: "蓝色",
    pan: "水平",
    tilt: "垂直",
  })[key] ?? key;

export function sceneValueLabel(
  value: SceneView["values"][number],
  fixtures: FixtureView[],
): string {
  if (value.mode === "release") return "释放";
  if (value.functionValue) {
    const selection = value.functionValue;
    const definition = fixtures
      .find((fixture) => fixture.id === value.fixtureId)
      ?.attributes.find((attribute) => attribute.key === value.attribute)
      ?.function?.functions.find(
        (entry) => entry.key === selection.functionKey,
      );
    if (!definition) return "功能定义不可用";
    return definition.mode === "slot"
      ? definition.name
      : `${definition.name} · ${((selection.position / 65535) * 100).toFixed(2)}%`;
  }
  return value.value === null
    ? "未记录"
    : `${((value.value / 65535) * 100).toFixed(2)}%`;
}
