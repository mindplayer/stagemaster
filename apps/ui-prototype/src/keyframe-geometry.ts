import type { EffectAttribute } from "./effect-types";
import { readFrames, type FrameDraft } from "./keyframe-tools.ts";

// Editing geometry only. Runtime sampling, phase and fixture spread remain in Rust.
export function curvePath(frames: FrameDraft[], attribute: EffectAttribute) {
  readFrames(frames, [attribute]);
  const point = (f: FrameDraft) => [
    Number(f.position),
    100 - Number(f.values[attribute]),
  ];
  const start = point(frames[0]);
  let path = `M ${start[0]} ${start[1]}`;
  frames.forEach((frame, i) => {
    const [x, y] = point(frame);
    const [end, value] =
      i + 1 < frames.length ? point(frames[i + 1]) : [100, start[1]];
    if (frame.transition === "hold") path += ` H ${end} V ${value}`;
    else if (frame.transition === "linear") path += ` L ${end} ${value}`;
    // Cubic smoothstep has horizontal tangents and controls at 1/3 and 2/3.
    else
      path += ` C ${x + (end - x) / 3} ${y} ${end - (end - x) / 3} ${value} ${end} ${value}`;
  });
  return path;
}

export function moveCurveFrame(
  frames: FrameDraft[],
  index: number,
  attribute: EffectAttribute,
  position: number,
  value: number,
): FrameDraft[] {
  readFrames(frames, [attribute]);
  if (!frames[index] || !Number.isFinite(position) || !Number.isFinite(value))
    throw new Error("关键帧坐标无效");
  const min = index
    ? Math.round(Number(frames[index - 1].position) * 100) + 1
    : 0;
  const max = index
    ? frames[index + 1]
      ? Math.round(Number(frames[index + 1].position) * 100) - 1
      : 9999
    : 0;
  const time = Math.min(max, Math.max(min, Math.round(position * 100))) / 100;
  const amount = Math.round(Math.min(100, Math.max(0, value)) * 10000) / 10000;
  return frames.map((f, i) =>
    i === index
      ? {
          ...f,
          position: String(time),
          values: { ...f.values, [attribute]: String(amount) },
        }
      : f,
  );
}
