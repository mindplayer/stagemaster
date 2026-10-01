import { fixtureAddress } from "./fixture-plan-display.ts";
import { sameFunctions } from "./fixture-function-types.ts";
import {
  functionDraftValue,
  type FunctionParameterDraft,
} from "./function-parameter-tools.ts";
import type {
  EditOperation,
  FixtureView,
  ProjectView,
  SceneView,
} from "./application-host";

export function uniqueName(base: string, names: string[]): string {
  if (!names.includes(base)) return base;
  let n = 2;
  while (names.includes(`${base} ${n}`)) n++;
  return `${base} ${n}`;
}
export function availableAddress(
  project: ProjectView,
  domainId: string,
  universe: number,
  footprint: number,
  count = 1,
): number | null {
  if (
    ![universe, footprint, count].every((v) => Number.isInteger(v) && v > 0) ||
    universe > 65535
  )
    return null;
  const occupied = project.fixtures.filter(
    (f) =>
      f.domainId === domainId && f.universe === universe && f.address !== null,
  );
  const width = footprint * count;
  for (let start = 1; start + width - 1 <= 512; start++) {
    if (
      !occupied.some(
        (f) =>
          start <= f.address! + f.footprint - 1 && start + width > f.address!,
      )
    )
      return start;
  }
  return null;
}
export function commonAttributes(fixtures: FixtureView[]) {
  return (
    fixtures[0]?.attributes.filter((a) =>
      fixtures.every((f) =>
        f.attributes.some(
          (b) =>
            b.key === a.key &&
            sameFunctions(a.function?.functions, b.function?.functions),
        ),
      ),
    ) ?? []
  );
}
export function attributeState(
  scene: SceneView,
  fixtures: FixtureView[],
  key: string,
) {
  const values = fixtures.map((f) => {
    const entry = scene.values.find(
      (v) => v.fixtureId === f.id && v.attribute === key,
    );
    return {
      mode: entry?.mode ?? "absent",
      value:
        entry?.value ?? f.attributes.find((a) => a.key === key)!.defaultValue,
      presetName: entry?.presetName ?? null,
      presetId: entry?.presetId ?? null,
    };
  });
  const first = values[0];
  const mixed = values.some(
    (v) =>
      v.value !== first.value ||
      v.mode !== first.mode ||
      v.presetId !== first.presetId,
  );
  return { ...first, mixed };
}
export function selectRange(
  ids: string[],
  selected: string[],
  id: string,
  anchor: string,
  range: boolean,
  toggle: boolean,
) {
  if (range && ids.includes(anchor) && ids.includes(id)) {
    const [a, b] = [ids.indexOf(anchor), ids.indexOf(id)].sort((x, y) => x - y);
    const span = ids.slice(a, b + 1);
    return toggle ? [...new Set([...selected, ...span])] : span;
  }
  return toggle
    ? selected.includes(id)
      ? selected.filter((v) => v !== id)
      : [...selected, id]
    : [id];
}
export type ParameterDraft =
  number | string | { mode: "release" | "remove" } | FunctionParameterDraft;
export function parameterCommands(
  sceneId: string,
  fixtures: FixtureView[],
  drafts: Record<string, ParameterDraft>,
): EditOperation[] {
  const attributes = commonAttributes(fixtures);
  const commands: EditOperation[] = [];
  for (const [attribute, draft] of Object.entries(drafts)) {
    const spec = attributes.find((a) => a.key === attribute);
    if (!spec) throw new Error("选择范围已变化，请重新编辑");
    if (typeof draft === "object" && "function" in draft) {
      const selection = functionDraftValue(spec, draft);
      for (const fixture of fixtures)
        commands.push({
          op: "setSceneFunctionValue",
          sceneId,
          fixtureId: fixture.id,
          attribute,
          selection,
        });
      continue;
    }
    if (spec.function && typeof draft !== "object")
      throw new Error(`${spec.label}需要选择一个功能`);
    let value = 0;
    let mode: "literal" | "release" | "remove" = "literal";
    if (typeof draft === "object") mode = draft.mode;
    else if (typeof draft === "number") value = draft;
    else {
      const percent = Number(draft);
      if (
        !draft.trim() ||
        !Number.isFinite(percent) ||
        percent < 0 ||
        percent > 100
      )
        throw new Error(`${spec.label}需要填写 0–100 之间的百分比`);
      value = Math.round((percent * 65535) / 100);
    }
    for (const fixture of fixtures)
      commands.push({
        op: "setSceneValue",
        sceneId,
        fixtureId: fixture.id,
        attribute,
        mode,
        value,
      });
  }
  if (commands.length > 256)
    throw new Error("一次最多修改 256 项属性，请减少选中的灯具数量");
  return commands;
}
export function fixtureMatches(fixture: FixtureView, query: string) {
  return `${fixture.name} ${fixture.profileName} ${fixture.domainName} ${fixtureAddress(fixture)} ${fixture.universe ?? ""}.${fixture.address ?? ""}`
    .toLocaleLowerCase()
    .includes(query.trim().toLocaleLowerCase());
}
