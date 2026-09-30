import type { EffectChannel, SceneEffect } from "./effect-types";

export type PositionEffectTemplate = "panSweep" | "tiltSweep" | "circle";
export const positionTemplates: {
  key: PositionEffectTemplate;
  name: string;
  detail: string;
}[] = [
  { key: "panSweep", name: "水平摆动", detail: "围绕当前位置 · 左右往返" },
  { key: "tiltSweep", name: "垂直摆动", detail: "围绕当前位置 · 上下往返" },
  { key: "circle", name: "双轴圆形", detail: "两轴错开相位 · 连续运动" },
];
export function isPositionTemplate(
  value: string,
): value is PositionEffectTemplate {
  return positionTemplates.some((t) => t.key === value);
}
/** Authoring parameters only. Physical mapping and playback sampling belong to Rust. */
export function createPositionEffect(
  kind: PositionEffectTemplate,
  id: string,
  fixtureIds: string[],
): SceneEffect {
  return {
    id,
    name: positionTemplates.find((t) => t.key === kind)!.name,
    enabled: true,
    fixtureIds: [...fixtureIds],
    periodMs: 8000,
    spreadDegrees: 0,
    phaseDegrees: 0,
    reverse: false,
    waveform: "position",
    dutyPercent: 25,
    channels: (kind === "circle"
      ? (["pan", "tilt"] as const)
      : kind === "panSweep"
        ? (["pan"] as const)
        : (["tilt"] as const)
    ).map((attribute) => ({
      attribute,
      amplitudeDegrees:
        kind === "circle" ? "15" : attribute === "pan" ? "30" : "15",
      offsetDegrees: "0",
      phaseDegrees: kind === "circle" && attribute === "tilt" ? 90 : 0,
    })),
  };
}
export interface PositionEffectAxisDraft {
  attribute: "pan" | "tilt";
  amplitude: string;
  offset: string;
  phase: string;
}
export function positionAxisDrafts(
  channels: EffectChannel[],
): PositionEffectAxisDraft[] {
  return channels.map((c) => {
    if (
      (c.attribute !== "pan" && c.attribute !== "tilt") ||
      c.amplitudeDegrees === undefined
    )
      throw new Error("位置效果轴参数无效");
    return {
      attribute: c.attribute,
      amplitude: c.amplitudeDegrees,
      offset: c.offsetDegrees,
      phase: String(c.phaseDegrees),
    };
  });
}
function degrees(raw: string, min: number, label: string): string {
  const value = Number(raw);
  if (
    !/^-?(0|[1-9]\d*)(\.\d{1,6})?$/.test(raw.trim()) ||
    !Number.isFinite(value) ||
    value < min ||
    value > 3600
  )
    throw new Error(`${label}须为 ${min}–3600°，最多六位小数`);
  return value.toFixed(6).replace(/\.?0+$/, "") || "0";
}
export function readPositionAxes(
  axes: PositionEffectAxisDraft[],
): EffectChannel[] {
  if (
    axes.length < 1 ||
    axes.length > 2 ||
    new Set(axes.map((a) => a.attribute)).size !== axes.length
  )
    throw new Error("位置效果需要一至两个独立运动轴");
  return axes.map((a) => {
    const label = a.attribute === "pan" ? "水平" : "垂直";
    if (!/^\d+$/.test(a.phase.trim()) || Number(a.phase) > 359)
      throw new Error(`${label}相位须为 0–359 的整数`);
    return {
      attribute: a.attribute,
      amplitudeDegrees: degrees(a.amplitude, 0, `${label}幅度`),
      offsetDegrees: degrees(a.offset, -3600, `${label}中心偏移`),
      phaseDegrees: Number(a.phase),
    };
  });
}
