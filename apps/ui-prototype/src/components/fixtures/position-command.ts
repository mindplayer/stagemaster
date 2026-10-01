import type { EditOperation, FixtureView } from "../../application-host";
import type { PositionEdit } from "../../position-types";
import { FixtureFieldError } from "../../fixture-tools";
import { positionDecimal } from "../../position-tools";
export type PositionMode = "axes" | "offset" | "aim" | "calibrate";
export type PositionAction = "home" | "flip" | "release" | "remove" | null;
export type PositionValues = Record<
  | "pan"
  | "tilt"
  | "panOffset"
  | "tiltOffset"
  | "x"
  | "y"
  | "z"
  | "branch"
  | "panZero"
  | "tiltZero",
  string
>;
export function collectPosition({
  mode,
  values,
  fixtures,
  sceneId,
  action,
}: {
  mode: PositionMode;
  values: PositionValues;
  fixtures: FixtureView[];
  sceneId: string;
  action: PositionAction;
}): EditOperation[] {
  const heads = fixtures.filter((f) => f.positioning);
  if (!heads.length || heads.length !== fixtures.length)
    throw new Error("请选择全部具有两轴模型的灯具");
  const v = values,
    fixtureIds = heads.map((f) => f.id);
  const decimal = (key: keyof typeof v, label: string, limit = 3600) =>
    positionDecimal(v[key], key, label, limit);
  if (action === "home" || action === "flip")
    return [{ op: "position", command: { op: action, sceneId, fixtureIds } }];
  if (action === "release" || action === "remove")
    return fixtureIds.flatMap((fixtureId) =>
      ["pan", "tilt"].map((attribute) => ({
        op: "setSceneValue" as const,
        sceneId,
        fixtureId,
        attribute,
        mode: action as "release" | "remove",
        value: 0,
      })),
    );
  let command: PositionEdit;
  if (mode === "axes") {
    for (const key of ["pan", "tilt"] as const) {
      if (v[key].trim()) {
        const n = Number(decimal(key, key === "pan" ? "水平角" : "垂直角"));
        for (const f of heads) {
          const a = f.positioning![key];
          if (n < Number(a.minDegrees) || n > Number(a.maxDegrees))
            throw new FixtureFieldError(
              key,
              `${f.name}：${key === "pan" ? "水平" : "垂直"}角须在 ${a.minDegrees}–${a.maxDegrees}° 之间`,
            );
        }
      }
    }
    if (!v.pan.trim() && !v.tilt.trim())
      throw new FixtureFieldError("pan", "至少填写一个轴角度");
    command = {
      op: "axes",
      fixtureIds,
      sceneId,
      panDegrees: v.pan.trim() ? decimal("pan", "水平角") : null,
      tiltDegrees: v.tilt.trim() ? decimal("tilt", "垂直角") : null,
    };
  } else if (mode === "offset") {
    const panDegrees = v.panOffset.trim()
      ? decimal("panOffset", "水平增量")
      : null;
    const tiltDegrees = v.tiltOffset.trim()
      ? decimal("tiltOffset", "垂直增量")
      : null;
    if (!Number(panDegrees) && !Number(tiltDegrees))
      throw new FixtureFieldError("panOffset", "至少填写一个非零轴增量");
    command = {
      op: "offsetAxes",
      sceneId,
      fixtureIds,
      panDegrees,
      tiltDegrees,
    };
  } else if (mode === "aim")
    command = {
      op: "aim",
      fixtureIds,
      sceneId,
      targetMeters: {
        x: decimal("x", "目标 X", 100000),
        y: decimal("y", "目标 Y", 100000),
        z: decimal("z", "目标高度", 100000),
      },
      branch: (v.branch as "front" | "back") || null,
    };
  else {
    if (heads.length !== 1) throw new Error("请单独选择一台灯具设置零偏");
    const old = heads[0].zeroCorrection;
    command = {
      op: "calibrate",
      fixtureId: heads[0].id,
      correction: {
        panDegrees: v.panZero.trim()
          ? decimal("panZero", "水平零偏", 360)
          : (old?.panDegrees ?? "0"),
        tiltDegrees: v.tiltZero.trim()
          ? decimal("tiltZero", "垂直零偏", 360)
          : (old?.tiltDegrees ?? "0"),
      },
    };
  }
  return [{ op: "position", command }];
}
