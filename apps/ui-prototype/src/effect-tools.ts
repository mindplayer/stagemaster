import type { FixtureView, EditOperation } from "./application-host";
import type { SceneEffect } from "./effect-types";
import { effectTargetIssues } from "./effect-targets.ts";
import {
  createPositionEffect,
  isPositionTemplate,
  positionTemplates,
} from "./position-effect-tools.ts";
import type { PositionEffectTemplate } from "./position-effect-tools.ts";
export type EffectTemplate =
  "breathe" | "chase" | "color" | "multicolor" | PositionEffectTemplate;
export const effectTemplates: {
  key: EffectTemplate;
  name: string;
  detail: string;
}[] = [
  { key: "breathe", name: "亮度呼吸", detail: "平滑明暗 · 同步起伏" },
  { key: "chase", name: "亮度追逐", detail: "依次点亮 · 按选灯顺序" },
  { key: "color", name: "双色循环", detail: "两色渐变 · 可展开灯序" },
  { key: "multicolor", name: "多色关键帧", detail: "逐帧调色 · 自由过渡" },
  ...positionTemplates,
];
export function supportsEffect(fixtures: FixtureView[], kind: EffectTemplate) {
  if (isPositionTemplate(kind))
    return (
      fixtures.length > 0 &&
      fixtures.every(
        (f) =>
          f.positioning &&
          ["pan", "tilt"].every((key) =>
            f.attributes.some((a) => a.key === key),
          ),
      )
    );
  const keys =
    kind === "color" || kind === "multicolor"
      ? ["red", "green", "blue"]
      : ["dimmer"];
  return (
    fixtures.length > 0 &&
    fixtures.every((f) =>
      keys.every((key) => f.attributes.some((a) => a.key === key)),
    )
  );
}
/** Authoring defaults only. Runtime curves are exclusively evaluated by Rust. */
export function createEffect(
  kind: EffectTemplate,
  id: string,
  fixtureIds: string[],
): SceneEffect {
  if (isPositionTemplate(kind))
    return createPositionEffect(kind, id, fixtureIds);
  if (kind === "multicolor") {
    const base = createEffect("color", id, fixtureIds);
    return {
      ...base,
      name: "多色关键帧",
      waveform: "keyframes",
      periodMs: 6000,
      channels: (["red", "green", "blue"] as const).map((attribute, i) => ({
        attribute,
        keyframes: [
          { position: 0, value: [0, 34952, 65535][i], transition: "smooth" },
          { position: 3333, value: [50000, 0, 65535][i], transition: "smooth" },
          { position: 6667, value: 65535, transition: "smooth" },
        ],
      })),
    };
  }
  return {
    id,
    name: effectTemplates.find((t) => t.key === kind)!.name,
    enabled: true,
    fixtureIds: [...fixtureIds],
    periodMs: kind === "color" ? 4000 : 2000,
    spreadDegrees: kind === "chase" ? 360 : 0,
    phaseDegrees: 0,
    reverse: false,
    waveform: kind === "chase" ? "pulse" : "smooth",
    dutyPercent: 25,
    channels:
      kind === "color"
        ? [
            { attribute: "red", low: 0, high: 65535 },
            { attribute: "green", low: 34952, high: 16448 },
            { attribute: "blue", low: 65535, high: 32896 },
          ]
        : [{ attribute: "dimmer", low: 0, high: 65535 }],
  };
}
export function reorderEffect(ids: string[], index: number, delta: number) {
  const next = [...ids],
    target = index + delta;
  if (index >= 0 && index < ids.length && target >= 0 && target < ids.length)
    [next[index], next[target]] = [next[target], next[index]];
  return next;
}
export function effectCommands(
  sceneId: string,
  effect: SceneEffect,
  fixtures: FixtureView[],
  illuminate: boolean,
): EditOperation[] {
  if (!effect.name.trim()) throw new Error("请填写效果名称");
  const issue = effectTargetIssues(
    effect.fixtureIds,
    fixtures,
    effect.channels,
    effect.waveform === "position",
  )[0];
  if (issue) throw new Error(`灯具“${issue.name}”：${issue.reason}`);
  const commands: EditOperation[] = [
    {
      op: "effect",
      command: {
        kind: "put",
        sceneId,
        effect: { ...effect, name: effect.name.trim() },
      },
    },
  ];
  if (illuminate)
    for (const id of effect.fixtureIds) {
      if (
        fixtures
          .find((f) => f.id === id)
          ?.attributes.some((a) => a.key === "dimmer")
      )
        commands.push({
          op: "setSceneValue",
          sceneId,
          fixtureId: id,
          attribute: "dimmer",
          mode: "literal",
          value: 65535,
        });
    }
  if (commands.length > 256)
    throw new Error("本次修改超过容量，请减少灯具数量");
  return commands;
}

/** Independent authoring copy: new identity, explicit fixture scope, no live reference. */
export function reuseEffect(
  effect: SceneEffect,
  id: string,
  fixtureIds?: string[],
): SceneEffect {
  return {
    ...structuredClone(effect),
    id,
    enabled: false,
    fixtureIds: [...(fixtureIds ?? effect.fixtureIds)],
  };
}
