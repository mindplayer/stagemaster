import type { EditOperation, ProjectView } from "../../application-host";
import type {
  FixturePlacement,
  StageSpace,
  StageView,
} from "../../stage-types";
import {
  arrangementDraft,
  arrangePlacements,
  type ArrangementDraft,
} from "../../placement-tools.ts";
import { bounds } from "../../stage-tools.ts";
import { movementBlocker, placementTargets } from "../../stage-locks.ts";

export interface ArrangementSession {
  source: string;
  ids: string[];
  draft: ArrangementDraft;
  dirty: boolean;
}

/** Drafts are tied to their source geometry, never silently rebased after an external edit. */
function sourceKey(project: ProjectView) {
  return JSON.stringify([
    project.id,
    project.stage,
    project.fixtures.map((f) => f.id),
  ]);
}

export function beginArrangement(
  project: ProjectView,
  ids: string[],
  space: StageSpace | undefined,
  selected: boolean,
): ArrangementSession {
  const items = project.stage.placements.filter((p) =>
    ids.includes(p.fixtureId),
  );
  const points: [number, number][] = items.length
    ? items.map((p) => [Number(p.positionMeters.x), Number(p.positionMeters.y)])
    : (space?.outlineMeters.map((p) => [Number(p[0]), Number(p[1])]) ?? []);
  const b = points.length ? bounds(points) : null;
  const draft = arrangementDraft(
    b ? (b.minX + b.maxX) / 2 : 0,
    b ? (b.minY + b.maxY) / 2 : 0,
    items.length
      ? Number(items[0].positionMeters.z)
      : Number(space?.floorElevationMeters ?? 0) +
          Number(space?.clearHeightMeters ?? 4) -
          0.5,
    space?.id ?? null,
  );
  if (selected) draft.mode = "move";
  return { source: sourceKey(project), ids: [...ids], draft, dirty: false };
}

export function arrangementPlacements(
  project: ProjectView,
  session: ArrangementSession,
) {
  if (session.source !== sourceKey(project))
    throw new Error("场地或灯具已变化，请取消本次排列后重新选择");
  if (session.ids.some((id) => !project.fixtures.some((f) => f.id === id)))
    throw new Error("所选灯具已不存在，请重新选择");
  if (movementBlocker(project.stage, placementTargets(session.ids)))
    throw new Error("所选灯位包含锁定对象，请取消排列并解锁后重试");
  return arrangePlacements(
    session.ids,
    project.stage.placements,
    session.draft,
  );
}

function samePlacement(a: FixturePlacement, b: FixturePlacement) {
  return (
    a.spaceId === b.spaceId &&
    (["positionMeters", "rotationDegreesXYZ"] as const).every((field) =>
      (["x", "y", "z"] as const).every(
        (axis) => Number(a[field][axis]) === Number(b[field][axis]),
      ),
    )
  );
}

export function arrangementOperations(
  project: ProjectView,
  session: ArrangementSession,
): EditOperation[] {
  if (!session.dirty) return [];
  return arrangementPlacements(project, session)
    .filter((p) => {
      const old = project.stage.placements.find(
        (v) => v.fixtureId === p.fixtureId,
      );
      return !old || !samePlacement(old, p);
    })
    .map((placement) => ({
      op: "stage",
      command: { op: "putPlacement", placement },
    }));
}

/** Display-only projection: it never changes the source stage or creates a project revision. */
export function arrangementStage(
  stage: StageView,
  placements: FixturePlacement[] | null,
): StageView {
  if (!placements) return stage;
  const replacements = new Map(placements.map((p) => [p.fixtureId, p]));
  const existing = new Set(stage.placements.map((p) => p.fixtureId));
  return {
    ...stage,
    placements: [
      ...stage.placements.map((p) => replacements.get(p.fixtureId) ?? p),
      ...placements.filter((p) => !existing.has(p.fixtureId)),
    ],
  };
}
