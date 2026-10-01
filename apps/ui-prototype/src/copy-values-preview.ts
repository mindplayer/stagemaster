import type { FixtureView, SceneView } from "./application-host";
import { attributeName, matchingValues } from "./library-tools.ts";

type SourceValue = SceneView["values"][number];
export interface CopyTargetPreview {
  fixture: FixtureView;
  issues: string[];
}
export function copyValuesPreview(
  scene: SceneView,
  source: FixtureView | undefined,
  candidates: FixtureView[],
  attributes: string[],
) {
  const keys = [...new Set(attributes)];
  const values = source ? matchingValues(scene.values, [source.id], keys) : [];
  const byKey = new Map(values.map((v) => [v.attribute, v]));
  const missingSource = keys.filter(
    (key) => !source?.attributes.some((a) => a.key === key),
  );
  const targets: CopyTargetPreview[] = candidates
    .filter((f) => f.id !== source?.id)
    .map((fixture) => ({
      fixture,
      issues: keys.flatMap((key) => {
        const target = fixture.attributes.find((a) => a.key === key);
        if (!target) return [`缺少${attributeName(key)}`];
        const reason = valueIssue(
          byKey.get(key),
          target,
          source?.attributes.find((a) => a.key === key),
        );
        return reason ? [`${attributeName(key)}：${reason}`] : [];
      }),
    }));
  return {
    keys,
    values,
    targets,
    skipped: keys.filter((key) => !byKey.has(key)),
    sourceIssue: !source
      ? "请选择来源灯具"
      : missingSource.length
        ? `来源缺少${missingSource.map(attributeName).join("、")}`
        : !keys.length
          ? "请选择复制属性"
          : !values.length
            ? "来源没有已记录的数值；释放和未记录项不会复制"
            : "",
  };
}
function valueIssue(
  value: SourceValue | undefined,
  target: FixtureView["attributes"][number],
  source?: FixtureView["attributes"][number],
): string {
  if (!value) return "";
  if (!value.functionValue)
    return target.function ? "连续值不能写入功能属性" : "";
  if (!target.function) return "功能值不能写入连续属性";
  const fn = target.function.functions.find(
    (f) => f.key === value.functionValue!.functionKey,
  );
  if (!fn) {
    const name =
      source?.function?.functions.find(
        (f) => f.key === value.functionValue!.functionKey,
      )?.name ?? "来源所选功能";
    return `没有功能“${name}”`;
  }
  if (fn.mode === "slot" && value.functionValue.position !== 0)
    return `“${fn.name}”是固定档位，不能接收区间位置`;
  return "";
}
export function copySubmissionIssue(
  preview: ReturnType<typeof copyValuesPreview>,
  selected: string[],
) {
  const ids = new Set(selected);
  const targets = preview.targets.filter((t) => ids.has(t.fixture.id));
  if (preview.sourceIssue) return preview.sourceIssue;
  if (!targets.length) return "请至少选择一台写入目标";
  const invalid = targets.filter((t) => t.issues.length);
  if (invalid.length)
    return `${invalid.length} 台已选目标无法写入，请调整目标或属性范围`;
  if (targets.length * preview.values.length > 10000)
    return "本次复制超过 10000 项属性，请缩小目标或属性范围";
  return "";
}
