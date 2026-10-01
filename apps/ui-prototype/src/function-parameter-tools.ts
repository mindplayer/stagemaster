import type { FixtureView, SceneView } from "./application-host";
import type { FunctionSelection } from "./fixture-function-types";
export interface FunctionParameterDraft {
  function: FunctionSelection;
  percent?: string;
}
export function functionState(
  scene: SceneView,
  fixtures: FixtureView[],
  key: string,
) {
  const states = fixtures.map((f) => {
    const entry = scene.values.find(
      (v) => v.fixtureId === f.id && v.attribute === key,
    );
    return {
      selection:
        entry?.functionValue ??
        f.attributes.find((a) => a.key === key)!.function!.default,
      mode: entry?.mode ?? "absent",
      presetId: entry?.presetId ?? null,
      presetName: entry?.presetName ?? null,
    };
  });
  const first = states[0];
  return {
    ...first,
    mixed: states.some(
      (s) =>
        s.selection.functionKey !== first.selection.functionKey ||
        s.selection.position !== first.selection.position ||
        s.mode !== first.mode ||
        s.presetId !== first.presetId,
    ),
  };
}
export function functionDraftValue(
  attribute: FixtureView["attributes"][number],
  draft: FunctionParameterDraft,
) {
  const f = attribute.function?.functions.find(
    (f) => f.key === draft.function.functionKey,
  );
  if (!f) throw new Error(`${attribute.label}的功能已不存在，请重新选择`);
  let position = draft.function.position;
  if (draft.percent !== undefined) {
    const percent = Number(draft.percent);
    if (
      !draft.percent.trim() ||
      !Number.isFinite(percent) ||
      percent < 0 ||
      percent > 100
    )
      throw new Error(`${attribute.label}区间位置需要填写 0–100 之间的百分比`);
    position = Math.round((percent * 65535) / 100);
  }
  if (
    !Number.isInteger(position) ||
    position < 0 ||
    position > 65535 ||
    (f.mode === "slot" && position !== 0)
  )
    throw new Error(`${attribute.label}功能位置无效，请重新选择`);
  return { functionKey: f.key, position };
}
