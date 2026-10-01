import type {
  StageEditLock,
  StageObject,
  StageSelection,
  StageView,
} from "./stage-types.ts";
export function stageTarget(object: StageObject): StageSelection {
  return {
    kind: object.kind,
    id: object.kind === "placement" ? object.value.fixtureId : object.value.id,
  };
}
export function lockTargets(targets: StageSelection[]): StageEditLock[] {
  return targets.map(({ kind, id }) => ({ kind, targetId: id }));
}
export function isStageLocked(
  stage: StageView,
  target: StageSelection | null,
): boolean {
  return (
    !!target &&
    !!stage.editLocks?.some(
      (lock) => lock.kind === target.kind && lock.targetId === target.id,
    )
  );
}
/** Local gesture feedback; Rust remains authoritative for every direct/indirect edit. */
export function movementBlocker(
  stage: StageView,
  targets: StageSelection[],
): StageSelection | null {
  for (const target of targets) {
    if (isStageLocked(stage, target)) return target;
    const related: StageSelection[] =
      target.kind === "space"
        ? stage.constructions
            .filter(
              (c) =>
                c.shape.kind === "enclosure" && c.shape.spaceId === target.id,
            )
            .map((c) => ({ kind: "construction", id: c.id }))
        : target.kind === "construction"
          ? stage.attachments
              .filter((a) => a.constructionId === target.id)
              .map((a) => ({ kind: "placement", id: a.fixtureId }))
          : [];
    const blocked = related.find((t) => isStageLocked(stage, t));
    if (blocked) return blocked;
  }
  return null;
}
export function placementTargets(ids: string[]): StageSelection[] {
  return ids.map((id) => ({ kind: "placement", id }));
}
