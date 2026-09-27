import type { PositionAxis } from "./position-types";
import { FixtureFieldError } from "./fixture-tools.ts";
// Display only. Rust validates and writes all physical position edits.
export function axisReadout(
  axis: PositionAxis,
  value: number,
  fine: boolean,
): number {
  let t = fine ? value / 65535 : Math.floor(value / 256) / 255;
  if (axis.reversed) t = 1 - t;
  return (
    Number(axis.minDegrees) +
    (Number(axis.maxDegrees) - Number(axis.minDegrees)) * t
  );
}
export function positionDecimal(
  raw: string,
  field: string,
  label: string,
  limit: number,
): string {
  const value = raw.trim();
  if (
    !/^-?(0|[1-9]\d*)(\.\d+)?$/.test(value) ||
    !Number.isFinite(Number(value)) ||
    Math.abs(Number(value)) > limit
  )
    throw new FixtureFieldError(
      field,
      `${label}须填写 -${limit} 至 ${limit} 的数值`,
    );
  return value;
}
