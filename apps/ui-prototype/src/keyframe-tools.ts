import type {
  EffectAttribute,
  EffectChannel,
  EffectKeyframe,
  SceneEffect,
} from "./effect-types";
export const attributeLabels: Record<EffectAttribute, string> = {
  dimmer: "亮度",
  red: "红",
  green: "绿",
  blue: "蓝",
  pan: "水平",
  tilt: "垂直",
};
export interface FrameDraft {
  position: string;
  transition: EffectKeyframe["transition"];
  values: Partial<Record<EffectAttribute, string>>;
}
// Four decimal percentage places round-trip every u16 value without long repeating decimals.
export const valuePercent = (value: number) =>
  String(Number(((value * 100) / 65535).toFixed(4)));
export function toKeyframes(effect: SceneEffect): SceneEffect {
  if (effect.waveform === "position")
    throw new Error("相对位置效果须在运动属性中编辑");
  if (effect.waveform === "keyframes") return structuredClone(effect);
  const pulse = effect.waveform === "pulse";
  if (
    pulse &&
    (!Number.isInteger(effect.dutyPercent) ||
      effect.dutyPercent < 1 ||
      effect.dutyPercent > 99)
  )
    throw new Error("亮段比例须为 1–99 的整数");
  return {
    ...effect,
    waveform: "keyframes",
    channels: effect.channels.map((channel) => {
      if (
        channel.keyframes ||
        channel.low === undefined ||
        channel.high === undefined
      )
        throw new Error("效果数据与变化方式不符");
      return {
        attribute: channel.attribute,
        keyframes: [
          {
            position: 0,
            value: pulse ? channel.high : channel.low,
            transition: pulse
              ? "hold"
              : effect.waveform === "smooth"
                ? "smooth"
                : "linear",
          },
          {
            position: pulse ? effect.dutyPercent * 100 : 5000,
            value: pulse ? channel.low : channel.high,
            transition: pulse
              ? "hold"
              : effect.waveform === "smooth"
                ? "smooth"
                : "linear",
          },
        ],
      };
    }),
  };
}
export function frameDrafts(channels: EffectChannel[]): FrameDraft[] {
  return (channels[0]?.keyframes ?? []).map((frame, i) => ({
    position: String(frame.position / 100),
    transition: frame.transition,
    values: Object.fromEntries(
      channels.map((c) => [c.attribute, valuePercent(c.keyframes![i].value)]),
    ),
  }));
}
export class FrameInputError extends Error {
  readonly index: number;
  readonly field: string;
  constructor(message: string, index: number, field: string) {
    super(message);
    this.index = index;
    this.field = field;
  }
}
export function readFrames(
  frames: FrameDraft[],
  attributes: EffectAttribute[],
): EffectChannel[] {
  if (frames.length < 2 || frames.length > 32)
    throw new Error("效果需要 2–32 个关键帧");
  let previous = -1;
  const positions = frames.map((f, i) => {
    const raw = f.position.trim(),
      value = Number(raw),
      position = Math.round(value * 100);
    if (
      !/^\d+(?:\.\d{1,2})?$/.test(raw) ||
      value < 0 ||
      value >= 100 ||
      (i === 0 && position !== 0) ||
      position <= previous
    )
      throw new FrameInputError(
        `第 ${i + 1} 帧时间须在 0–99.99% 内，首帧为 0%，后续依次递增`,
        i,
        "position",
      );
    previous = position;
    return position;
  });
  return attributes.map((attribute) => ({
    attribute,
    keyframes: frames.map((f, i) => {
      const raw = f.values[attribute] ?? "",
        value = Number(raw);
      if (!raw.trim() || !Number.isFinite(value) || value < 0 || value > 100)
        throw new FrameInputError(
          `第 ${i + 1} 帧${attributeLabels[attribute]}须在 0–100% 之间`,
          i,
          attribute,
        );
      return {
        position: positions[i],
        value: Math.round((value * 65535) / 100),
        transition: f.transition,
      };
    }),
  }));
}
export function reorderFrames(
  frames: FrameDraft[],
  index: number,
  delta: number,
) {
  const target = index + delta;
  if (target < 0 || target >= frames.length) return frames;
  const next = structuredClone(frames),
    positions = frames.map((f) => f.position);
  [next[index], next[target]] = [next[target], next[index]];
  return next.map((f, i) => ({ ...f, position: positions[i] }));
}
export function evenFrames(frames: FrameDraft[]) {
  return frames.map((f, i) => ({
    ...f,
    position: String(Math.floor((i * 10000) / frames.length) / 100),
  }));
}
export function appendFrame(frames: FrameDraft[]) {
  const last = frames.at(-1)!;
  const position = Math.floor((Number(last.position) * 100 + 10000) / 2);
  if (
    !Number.isFinite(position) ||
    position >= 10000 ||
    position <= Number(last.position) * 100
  )
    throw new Error("最后一帧距循环末尾太近，请先均分时间或调整位置");
  return [
    ...frames,
    { ...structuredClone(last), position: String(position / 100) },
  ];
}
export function frameColor(values: FrameDraft["values"]) {
  return (
    "#" +
    ["red", "green", "blue"]
      .map((key) => {
        const n = Number(values[key as EffectAttribute] ?? 0);
        return Math.round(
          (Math.min(100, Math.max(0, Number.isFinite(n) ? n : 0)) * 255) / 100,
        )
          .toString(16)
          .padStart(2, "0");
      })
      .join("")
  );
}
export function colorValues(hex: string): FrameDraft["values"] {
  return Object.fromEntries(
    ["red", "green", "blue"].map((key, i) => [
      key,
      valuePercent(parseInt(hex.slice(1 + i * 2, 3 + i * 2), 16) * 257),
    ]),
  );
}
