import type { StageObject, StageView } from "../../stage-types.ts";
import { objectOutline } from "../../stage-tools.ts";
import { rigOutline } from "../../rigging-tools.ts";
type Point = [number, number];
export function planPoints(stage: StageView): Point[] {
  return [
    ...stage.spaces.flatMap((s) =>
      s.outlineMeters.map((p) => [Number(p[0]), Number(p[1])] as Point),
    ),
    ...stage.constructions.flatMap((c) =>
      c.shape.kind === "platform"
        ? c.shape.outlineMeters.map(
            (p) => [Number(p[0]), Number(p[1])] as Point,
          )
        : c.shape.kind === "rig"
          ? rigOutline(c.shape)
          : [],
    ),
    ...stage.placements.map(
      (p) => [Number(p.positionMeters.x), Number(p.positionMeters.y)] as Point,
    ),
  ];
}
export function selectionPoints(
  stage: StageView,
  object: StageObject | null,
  ids: string[],
): Point[] | null {
  const group = stage.placements
    .filter((p) => ids.includes(p.fixtureId))
    .map(
      (p) => [Number(p.positionMeters.x), Number(p.positionMeters.y)] as Point,
    );
  if (group.length) return group;
  if (!object) return null;
  const outline = objectOutline(object);
  if (outline) return outline.map((p) => [Number(p[0]), Number(p[1])]);
  if (object.kind === "placement")
    return [
      [
        Number(object.value.positionMeters.x),
        Number(object.value.positionMeters.y),
      ],
    ];
  if (object.kind === "construction" && object.value.shape.kind === "rig")
    return rigOutline(object.value.shape);
  return [];
}
