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

export { arrangeFixtureIds as arrangeEffectFixtures } from "./fixture-order.ts";
export type { FixtureOrder as EffectOrder } from "./fixture-order.ts";
