import type { EditCommand } from "./application-host";
import type { FixturePlacement, SpatialVector3 } from "./stage-types";
import { canonical, decimal } from "./stage-tools.ts";
export type ArrangementMode = "line" | "grid" | "circle" | "move" | "align";
export interface ArrangementDraft {
  mode: ArrangementMode;
  x: string;
  y: string;
  z: string;
  spacing: string;
  rowSpacing: string;
  columns: string;
  angle: string;
  radius: string;
  arc: string;
  dx: string;
  dy: string;
  dz: string;
  turn: string;
  axis: "x" | "y" | "z";
  alignment: "min" | "center" | "max" | "distribute";
  spaceId: string | null;
  setRotation: boolean;
  rx: string;
  ry: string;
  rz: string;
}
export function arrangementDraft(
  x = 0,
  y = 0,
  z = 4,
  spaceId: string | null = null,
): ArrangementDraft {
  return {
    mode: "line",
    x: decimal(x),
    y: decimal(y),
    z: decimal(z),
    spacing: "1",
    rowSpacing: "1",
    columns: "4",
    angle: "0",
    radius: "3",
    arc: "360",
    dx: "0",
    dy: "0",
    dz: "0",
    turn: "0",
    axis: "x",
    alignment: "center",
    spaceId,
    setRotation: false,
    rx: "0",
    ry: "0",
    rz: "0",
  };
}
export class PlacementInputError extends Error {
  field: keyof ArrangementDraft;
  constructor(field: keyof ArrangementDraft, message: string) {
    super(message);
    this.field = field;
  }
}
export function arrangePlacements(
  ids: string[],
  existing: FixturePlacement[],
  draft: ArrangementDraft,
): FixturePlacement[] {
  if (!ids.length || ids.length > 256) throw new Error("请选择 1–256 台灯具");
  if (new Set(ids).size !== ids.length) throw new Error("灯具选择不能重复");
  const number = (
    field: keyof ArrangementDraft,
    label: string,
    min = -100000,
    max = 100000,
  ) => {
    let value: number;
    try {
      value = Number(canonical(String(draft[field])));
    } catch {
      throw new PlacementInputError(field, `${label}请输入有效数字`);
    }
    if (value < min || value > max)
      throw new PlacementInputError(field, `${label}须在 ${min}–${max} 之间`);
    return value;
  };
  const originals = ids.map((id) => existing.find((p) => p.fixtureId === id));
  const transform = draft.mode === "move" || draft.mode === "align";
  if (transform && originals.some((p) => !p))
    throw new Error("平移、旋转和对齐需要灯具已有安装位置，请先布灯");
  const points = originals.map((p) =>
    p
      ? {
          x: Number(p.positionMeters.x),
          y: Number(p.positionMeters.y),
          z: Number(p.positionMeters.z),
        }
      : { x: 0, y: 0, z: 0 },
  );
  let positions: typeof points;
  if (draft.mode === "move") {
    const dx = number("dx", "X 平移"),
      dy = number("dy", "Y 平移"),
      dz = number("dz", "Z 平移");
    const angle = (number("turn", "排列旋转", -360, 360) * Math.PI) / 180;
    const cx =
      (Math.min(...points.map((p) => p.x)) +
        Math.max(...points.map((p) => p.x))) /
      2;
    const cy =
      (Math.min(...points.map((p) => p.y)) +
        Math.max(...points.map((p) => p.y))) /
      2;
    positions = points.map((p) => ({
      x: cx + (p.x - cx) * Math.cos(angle) - (p.y - cy) * Math.sin(angle) + dx,
      y: cy + (p.x - cx) * Math.sin(angle) + (p.y - cy) * Math.cos(angle) + dy,
      z: p.z + dz,
    }));
  } else if (draft.mode === "align") {
    const axis = draft.axis,
      values = points.map((p) => p[axis]);
    const min = Math.min(...values),
      max = Math.max(...values);
    const ordered = points
      .map((p, i) => ({ value: p[axis], i }))
      .sort((a, b) => a.value - b.value || a.i - b.i);
    positions = points.map((p) => ({ ...p }));
    ordered.forEach((p, rank) => {
      positions[p.i][axis] =
        draft.alignment === "min"
          ? min
          : draft.alignment === "max"
            ? max
            : draft.alignment === "center"
              ? (min + max) / 2
              : points.length === 1
                ? min
                : min + ((max - min) * rank) / (points.length - 1);
    });
  } else {
    const x = number("x", "中心 X"),
      y = number("y", "中心 Y"),
      z = number("z", "安装高度");
    const angle = (number("angle", "排列角度", -360, 360) * Math.PI) / 180;
    let local: { x: number; y: number }[];
    if (draft.mode === "circle") {
      const radius = number("radius", "半径", 0.001, 100000),
        arc = number("arc", "圆弧范围", 0.01, 360);
      local = ids.map((_, i) => {
        const a =
          angle +
          ((ids.length === 1
            ? 0
            : i / (arc === 360 ? ids.length : ids.length - 1)) *
            arc *
            Math.PI) /
            180;
        return { x: radius * Math.cos(a), y: radius * Math.sin(a) };
      });
    } else {
      const spacing = number("spacing", "灯间距", 0.001, 100000);
      const columns =
        draft.mode === "line"
          ? ids.length
          : number("columns", "每排灯数", 1, 256);
      if (!Number.isInteger(columns))
        throw new PlacementInputError("columns", "每排灯数须为整数");
      const rowSpacing =
        draft.mode === "grid"
          ? number("rowSpacing", "排间距", 0.001, 100000)
          : 0;
      const usedColumns = Math.min(columns, ids.length),
        rows = Math.ceil(ids.length / columns);
      local = ids.map((_, i) => {
        const a = ((i % columns) - (usedColumns - 1) / 2) * spacing,
          b = (Math.floor(i / columns) - (rows - 1) / 2) * rowSpacing;
        return {
          x: a * Math.cos(angle) - b * Math.sin(angle),
          y: a * Math.sin(angle) + b * Math.cos(angle),
        };
      });
    }
    positions = local.map((p) => ({ x: x + p.x, y: y + p.y, z }));
  }
  const rotation: SpatialVector3 | null = draft.setRotation
    ? {
        x: decimal(number("rx", "底座 X 旋转", -3600, 3600)),
        y: decimal(number("ry", "底座 Y 旋转", -3600, 3600)),
        z: decimal(number("rz", "底座 Z 旋转", -3600, 3600)),
      }
    : null;
  return positions.map((p, i) => {
    if (
      Object.values(p).some((v) => !Number.isFinite(v) || Math.abs(v) > 100000)
    )
      throw new Error(
        `第 ${i + 1} 台灯具超出场地坐标范围，请缩小间距或调整中心`,
      );
    return {
      fixtureId: ids[i],
      spaceId: transform ? originals[i]!.spaceId : draft.spaceId,
      positionMeters: { x: decimal(p.x), y: decimal(p.y), z: decimal(p.z) },
      rotationDegreesXYZ: {
        ...(rotation ??
          originals[i]?.rotationDegreesXYZ ?? { x: "0", y: "0", z: "0" }),
      },
    };
  });
}
export function placementBatch(placements: FixturePlacement[]): EditCommand {
  if (!placements.length || placements.length > 256)
    throw new Error("一次布置需要 1–256 台灯具");
  return {
    op: "batch",
    commands: placements.map((placement) => ({
      op: "stage",
      command: { op: "putPlacement", placement },
    })),
  };
}
export function togglePlacement(
  ids: string[],
  id: string,
  additive: boolean,
): string[] {
  return additive
    ? ids.includes(id)
      ? ids.filter((v) => v !== id)
      : [...ids, id]
    : [id];
}
export function selectInBox(
  placements: FixturePlacement[],
  from: [number, number],
  to: [number, number],
): string[] {
  return placements
    .filter((p) => {
      const x = Number(p.positionMeters.x),
        y = Number(p.positionMeters.y);
      return (
        x >= Math.min(from[0], to[0]) &&
        x <= Math.max(from[0], to[0]) &&
        y >= Math.min(from[1], to[1]) &&
        y <= Math.max(from[1], to[1])
      );
    })
    .map((p) => p.fixtureId);
}
