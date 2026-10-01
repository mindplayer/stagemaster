import type { SceneEffect, WorldLinePath } from "./effect-types";
import { canonical } from "./stage-tools.ts";

export class WorldLineInputError extends Error {
  field = "轨迹终点 X";
}

export function createWorldLine(id: string, fixtureIds: string[]): SceneEffect {
  return {
    id,
    name: "空间直线往返",
    enabled: true,
    fixtureIds: [...fixtureIds],
    periodMs: 8000,
    spreadDegrees: 0,
    phaseDegrees: 0,
    reverse: false,
    waveform: "worldLine",
    dutyPercent: 25,
    channels: [{ attribute: "pan" }, { attribute: "tilt" }],
    targetPath: {
      kind: "line",
      fromMeters: { x: "-1", y: "2", z: "0.5" },
      toMeters: { x: "1", y: "2", z: "0.5" },
      branch: "auto",
      maxErrorMeters: "0.1",
    },
  };
}

export function readWorldLine(path: WorldLinePath | undefined): WorldLinePath {
  if (!path) throw new Error("缺少空间轨迹参数");
  const readPoint = (point: WorldLinePath["fromMeters"]) => {
    const result = {
      x: canonical(point.x),
      y: canonical(point.y),
      z: canonical(point.z),
    };
    if (Object.values(result).some((n) => Math.abs(Number(n)) > 100000))
      throw new Error("轨迹坐标须在正负 100000 米以内");
    return result;
  };
  const fromMeters = readPoint(path.fromMeters),
    toMeters = readPoint(path.toMeters);
  if (
    (["x", "y", "z"] as const).every(
      (axis) => fromMeters[axis] === toMeters[axis],
    )
  )
    throw new WorldLineInputError("轨迹起点和终点不能重合");
  const maxErrorMeters = canonical(path.maxErrorMeters);
  if (Number(maxErrorMeters) < 0.001 || Number(maxErrorMeters) > 1)
    throw new Error("允许误差须在 0.001–1 米之间");
  return { ...path, fromMeters, toMeters, maxErrorMeters };
}
