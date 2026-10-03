import type {
  StageEdit,
  StageObject,
  StageSelection,
  StageView,
} from "./stage-types.ts";
export const decimal = (value: number) => String(Number(value.toFixed(6)));
export function canonical(value: string): string {
  if (
    !/^-?(?:\d+\.?\d*|\.\d+)$/.test(value.trim()) ||
    !Number.isFinite(Number(value))
  )
    throw new Error("请输入有效数字");
  return decimal(Number(value));
}
export function selectedStage(
  stage: StageView,
  selection: StageSelection | null,
): StageObject | null {
  if (!selection) return null;
  if (selection.kind === "space") {
    const value = stage.spaces.find((s) => s.id === selection.id);
    return value ? { kind: "space", value } : null;
  }
  if (selection.kind === "construction") {
    const value = stage.constructions.find((s) => s.id === selection.id);
    return value ? { kind: "construction", value } : null;
  }
  const value = stage.placements.find((p) => p.fixtureId === selection.id);
  return value ? { kind: "placement", value } : null;
}
export function objectOutline(object: StageObject): [string, string][] | null {
  return object.kind === "space"
    ? object.value.outlineMeters
    : object.kind === "construction" && object.value.shape.kind === "platform"
      ? object.value.shape.outlineMeters
      : null;
}
/** Resolve an object's editing space without substituting an unrelated room. */
export function objectSpace(stage: StageView, object: StageObject | null) {
  if (object?.kind === "space") return object.value;
  const id =
    object?.kind === "placement"
      ? object.value.spaceId
      : object?.kind === "construction"
        ? object.value.shape.spaceId
        : null;
  return stage.spaces.find((space) => space.id === id);
}
export function stageCommand(object: StageObject): StageEdit {
  const copy = structuredClone(object);
  const outline = objectOutline(copy);
  if (outline)
    for (const p of outline) {
      p[0] = canonical(p[0]);
      p[1] = canonical(p[1]);
    }
  if (copy.kind === "space") {
    copy.value.floorElevationMeters = canonical(
      copy.value.floorElevationMeters,
    );
    if (copy.value.clearHeightMeters !== null)
      copy.value.clearHeightMeters = canonical(copy.value.clearHeightMeters);
    return { op: "putSpace", ...copy.value, name: copy.value.name.trim() };
  }
  if (copy.kind === "construction") {
    const s = copy.value.shape;
    if (s.kind === "platform") {
      s.baseElevationMeters = canonical(s.baseElevationMeters);
      s.heightMeters = canonical(s.heightMeters);
    } else if (s.kind === "seating") {
      for (const axis of ["x", "y", "z"] as const)
        s.positionMeters[axis] = canonical(s.positionMeters[axis]);
      for (const key of [
        "yawDegrees",
        "seatWidthMeters",
        "seatDepthMeters",
        "columnSpacingMeters",
        "rowSpacingMeters",
      ] as const)
        s[key] = canonical(s[key]);
      if (s.aisle) s.aisle.widthMeters = canonical(s.aisle.widthMeters);
      if (s.arc) s.arc.radiusMeters = canonical(s.arc.radiusMeters);
    } else if (s.kind === "rig") {
      for (const axis of ["x", "y", "z"] as const)
        s.positionMeters[axis] = canonical(s.positionMeters[axis]);
      for (const key of [
        "yawDegrees",
        "lengthMeters",
        "widthMeters",
        "heightMeters",
      ] as const)
        s[key] = canonical(s[key]);
    } else {
      s.wallThicknessMeters = canonical(s.wallThicknessMeters);
      s.floorThicknessMeters = canonical(s.floorThicknessMeters);
      if (s.ceilingThicknessMeters !== null)
        s.ceilingThicknessMeters = canonical(s.ceilingThicknessMeters);
    }
    return {
      op: "putConstruction",
      ...copy.value,
      name: copy.value.name.trim(),
    };
  }
  for (const v of [copy.value.positionMeters, copy.value.rotationDegreesXYZ])
    for (const axis of ["x", "y", "z"] as const) v[axis] = canonical(v[axis]);
  return { op: "putPlacement", placement: copy.value };
}
export function translated(
  object: StageObject,
  dx: number,
  dy: number,
): StageObject {
  const copy = structuredClone(object),
    outline = objectOutline(copy);
  if (outline)
    for (const p of outline) {
      p[0] = decimal(Number(p[0]) + dx);
      p[1] = decimal(Number(p[1]) + dy);
    }
  else if (
    copy.kind === "construction" &&
    (copy.value.shape.kind === "rig" || copy.value.shape.kind === "seating")
  ) {
    copy.value.shape.positionMeters.x = decimal(
      Number(copy.value.shape.positionMeters.x) + dx,
    );
    copy.value.shape.positionMeters.y = decimal(
      Number(copy.value.shape.positionMeters.y) + dy,
    );
  } else if (copy.kind === "placement") {
    copy.value.positionMeters.x = decimal(
      Number(copy.value.positionMeters.x) + dx,
    );
    copy.value.positionMeters.y = decimal(
      Number(copy.value.positionMeters.y) + dy,
    );
  }
  return copy;
}
export function bounds(points: [number, number][]) {
  const xs = points.map((p) => p[0]),
    ys = points.map((p) => p[1]);
  return {
    minX: Math.min(...xs),
    minY: Math.min(...ys),
    maxX: Math.max(...xs),
    maxY: Math.max(...ys),
  };
}
export function rectangle(
  x: number,
  y: number,
  w: number,
  h: number,
): [string, string][] {
  return [
    [x, y],
    [x + w, y],
    [x + w, y + h],
    [x, y + h],
  ].map(([x, y]) => [decimal(x!), decimal(y!)]);
}
