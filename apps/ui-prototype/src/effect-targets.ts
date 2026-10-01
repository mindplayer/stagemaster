import type { FixtureView } from "./application-host";
import type { EffectChannel } from "./effect-types";

export interface EffectTargetIssue {
  id: string;
  name: string;
  reason: string;
}

const labels: Record<string, string> = {
  dimmer: "亮度",
  red: "红",
  green: "绿",
  blue: "蓝",
  pan: "水平",
  tilt: "垂直",
};

/** Authoring feedback only; the Rust project compiler remains authoritative. */
export function effectTargetIssues(
  ids: string[],
  fixtures: FixtureView[],
  channels: EffectChannel[],
  requiresPosition = channels.some((c) => c.amplitudeDegrees !== undefined),
): EffectTargetIssue[] {
  if (!ids.length)
    return [{ id: "", name: "灯具", reason: "效果至少需要一台灯具" }];
  const byId = new Map(fixtures.map((f) => [f.id, f]));
  const seen = new Set<string>();
  return ids.flatMap((id) => {
    const fixture = byId.get(id);
    const name = fixture?.name ?? id;
    if (seen.has(id)) return [{ id, name, reason: "效果灯具不能重复" }];
    seen.add(id);
    if (!fixture) return [{ id, name, reason: "灯具已删除" }];
    const missing = channels.filter(
      (c) => !fixture.attributes.some((a) => a.key === c.attribute),
    );
    const reasons = missing.length
      ? [`缺少${missing.map((c) => labels[c.attribute]).join("、")}属性`]
      : [];
    if (requiresPosition && !fixture.positioning)
      reasons.push("未定义两轴运动模型");
    return reasons.length ? [{ id, name, reason: reasons.join("；") }] : [];
  });
}

export type EffectOrder = "reverse" | "oddFirst" | "name" | "patch";
const collator = new Intl.Collator("zh-CN", {
  numeric: true,
  sensitivity: "base",
});

/** Change only explicit authoring order; retain every member and stable ties. */
export function arrangeEffectFixtures(
  ids: string[],
  fixtures: FixtureView[],
  order: EffectOrder,
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
