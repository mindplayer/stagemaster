import { useState } from "react";
import type { ProjectView } from "../../application-host";
import type {
  RigShape,
  SeatingShape,
  StageEdit,
  StageObject,
  StageSelection,
  StageSpace,
} from "../../stage-types";
import type { Creation } from "./StageCreateDialog";
import { bounds, decimal, selectedStage } from "../../stage-tools";
import { useCommittedStageAction } from "./useCommittedStageAction";
import { uniqueName } from "../../editor-tools";
export function useStageObjects({
  project,
  busy,
  visible,
  onError,
  object,
  selectedSpace,
  fixtureId,
  beforeChange,
  edit,
  onResult,
}: {
  project: ProjectView;
  busy: boolean;
  visible: boolean;
  onError(message: string): void;
  object: StageObject | null;
  selectedSpace: StageSpace | undefined;
  fixtureId: string | undefined;
  beforeChange(): Promise<boolean>;
  edit(command: StageEdit): Promise<ProjectView | null>;
  onResult(
    target: StageSelection | null,
    next: ProjectView,
    focus?: boolean,
  ): void;
}) {
  const [rigCreation, setRigCreation] = useState<{
    shape: RigShape;
    name: string;
  } | null>(null);
  const [seatingCreation, setSeatingCreation] = useState<{
    shape: SeatingShape;
    name: string;
  } | null>(null);
  function createSeating(selectedSpace: StageSpace | undefined) {
    const b = selectedSpace
      ? bounds(
          selectedSpace.outlineMeters.map((p) => [Number(p[0]), Number(p[1])]),
        )
      : null;
    setSeatingCreation({
      name: uniqueName(
        "观众座区",
        project.stage.constructions.map((c) => c.name),
      ),
      shape: {
        kind: "seating",
        spaceId: selectedSpace?.id ?? null,
        positionMeters: {
          x: decimal(b ? (b.minX + b.maxX) / 2 : 0),
          y: decimal(b ? (b.minY + b.maxY) / 2 : 0),
          z: selectedSpace?.floorElevationMeters ?? "0",
        },
        yawDegrees: "0",
        rows: 5,
        columns: 6,
        seatWidthMeters: "0.46",
        seatDepthMeters: "0.48",
        columnSpacingMeters: "0.56",
        rowSpacingMeters: "0.9",
        aisle: null,
      },
    });
  }
  const [creation, setCreation] = useState<Creation | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<StageObject | null>(null);
  function createRig(selectedSpace: StageSpace | undefined) {
    const b = selectedSpace
      ? bounds(
          selectedSpace.outlineMeters.map((p) => [Number(p[0]), Number(p[1])]),
        )
      : null;
    setRigCreation({
      name: uniqueName(
        "桁架",
        project.stage.constructions.map((c) => c.name),
      ),
      shape: {
        kind: "rig",
        rigKind: "truss",
        spaceId: selectedSpace?.id ?? null,
        positionMeters: {
          x: decimal(b ? (b.minX + b.maxX) / 2 : 0),
          y: decimal(b ? (b.minY + b.maxY) / 2 : 0),
          z: decimal(
            Number(selectedSpace?.floorElevationMeters ?? 0) +
              Number(selectedSpace?.clearHeightMeters ?? 5) -
              0.3,
          ),
        },
        yawDegrees: "0",
        lengthMeters: "6",
        widthMeters: "0.3",
        heightMeters: "0.3",
      },
    });
  }
  function create(
    kind: "space" | "platform",
    selectedSpace: StageSpace | undefined,
  ) {
    const max = project.stage.spaces.flatMap((s) =>
      s.outlineMeters.map((p) => Number(p[0])),
    );
    const b = selectedSpace
      ? bounds(
          selectedSpace.outlineMeters.map((p) => [Number(p[0]), Number(p[1])]),
        )
      : null;
    setCreation({
      kind,
      name: uniqueName(
        kind === "space" ? "空间" : "舞台",
        (kind === "space"
          ? project.stage.spaces
          : project.stage.constructions
        ).map((v) => v.name),
      ),
      x:
        kind === "space"
          ? max.length
            ? Math.max(...max) + 2
            : 0
          : (b?.minX ?? 0),
      y: kind === "space" ? 0 : (b?.minY ?? 0),
      elevation:
        kind === "space" ? "0" : (selectedSpace?.floorElevationMeters ?? "0"),
      spaceId: selectedSpace?.id ?? null,
    });
  }
  async function createObject(command: StageEdit) {
    const next = await edit(command);
    if (!next) return;
    const target: StageSelection =
      command.op === "putSpace"
        ? { kind: "space", id: next.stage.spaces.at(-1)!.id }
        : { kind: "construction", id: next.stage.constructions.at(-1)!.id };
    setCreation(null);
    onResult(target, next, true);
    return true;
  }
  async function placeFixture(fixtureId: string, room: StageSpace | undefined) {
    if (!project.fixtures.some((f) => f.id === fixtureId))
      throw new Error("所选灯具已不存在，请重新选择");
    if (project.stage.placements.some((p) => p.fixtureId === fixtureId))
      throw new Error("该灯具已经完成布置，请选择其他待布置灯具");
    const b = room
      ? bounds(room.outlineMeters.map((p) => [Number(p[0]), Number(p[1])]))
      : null;
    const next = await edit({
      op: "putPlacement",
      placement: {
        fixtureId: fixtureId,
        spaceId: room?.id ?? null,
        positionMeters: {
          x: decimal(b ? (b.minX + b.maxX) / 2 : 0),
          y: decimal(b ? (b.minY + b.maxY) / 2 : 0),
          z: decimal(
            Number(room?.floorElevationMeters ?? 0) +
              Number(room?.clearHeightMeters ?? 4) -
              0.5,
          ),
        },
        rotationDegreesXYZ: { x: "0", y: "0", z: "0" },
      },
    });
    if (next) {
      onResult({ kind: "placement", id: fixtureId }, next);
    }
  }
  async function enclose(object: StageObject) {
    if (object.kind !== "space") return;
    const next = await edit({
      op: "putConstruction",
      id: null,
      name: `${object.value.name}围护`,
      shape: {
        kind: "enclosure",
        spaceId: object.value.id,
        wallThicknessMeters: "0.2",
        floorThicknessMeters: "0.1",
        ceilingThicknessMeters: null,
      },
    });
    if (next) {
      const target: StageSelection = {
        kind: "construction",
        id: next.stage.constructions.at(-1)!.id,
      };
      onResult(target, next);
    }
  }
  async function duplicate(object: StageObject) {
    if (object.kind === "placement") return;
    const next = await edit({
      op: object.kind === "space" ? "duplicateSpace" : "duplicateConstruction",
      id: object.value.id,
      name: uniqueName(
        `${object.value.name} 副本`,
        [...project.stage.spaces, ...project.stage.constructions].map(
          (s) => s.name,
        ),
      ),
    });
    if (next) {
      const target: StageSelection = {
        kind: object.kind,
        id:
          object.kind === "space"
            ? next.stage.spaces.at(-1)!.id
            : next.stage.constructions.at(-1)!.id,
      };
      onResult(target, next);
    }
  }
  async function remove() {
    const target = deleteTarget;
    if (!target) return;
    const next = await edit(
      target.kind === "space"
        ? { op: "removeSpace", id: target.value.id, detachMembers: true }
        : target.kind === "construction"
          ? {
              op: "removeConstruction",
              id: target.value.id,
              detachFixtures: target.value.shape.kind === "rig",
            }
          : { op: "removePlacement", fixtureId: target.value.fixtureId },
    );
    if (next) {
      setDeleteTarget(null);
      onResult(null, next);
    }
  }
  type Intent =
    | { kind: "space" | "platform" | "rig" | "seating"; spaceId?: string }
    | { kind: "place"; spaceId?: string; fixtureId: string }
    | { kind: "duplicate" | "enclose" | "delete"; target: StageSelection };
  const run = useCommittedStageAction<Intent>({
    scopeId: project.id,
    active: visible,
    busy,
    beforeChange,
    onError,
    async execute(intent) {
      if ("target" in intent) {
        const target = selectedStage(project.stage, intent.target);
        if (!target) throw new Error("对象已不存在，请重新选择");
        if (intent.kind === "duplicate") await duplicate(target);
        else if (intent.kind === "enclose") await enclose(target);
        else setDeleteTarget(target);
        return;
      }
      const space = project.stage.spaces.find((s) => s.id === intent.spaceId);
      if (intent.spaceId && !space)
        throw new Error("所属空间已不存在，请重新选择");
      if (intent.kind === "place") await placeFixture(intent.fixtureId, space);
      else if (intent.kind === "rig") createRig(space);
      else if (intent.kind === "seating") createSeating(space);
      else create(intent.kind, space);
    },
  });
  function targetAction(kind: "duplicate" | "enclose" | "delete") {
    if (!object) return;
    return run({
      kind,
      target: {
        kind: object.kind,
        id:
          object.kind === "placement"
            ? object.value.fixtureId
            : object.value.id,
      },
    });
  }
  return {
    rigCreation,
    setRigCreation,
    seatingCreation,
    setSeatingCreation,
    createSeating: () => run({ kind: "seating", spaceId: selectedSpace?.id }),
    creation,
    setCreation,
    deleteTarget,
    setDeleteTarget,
    createRig: () => run({ kind: "rig", spaceId: selectedSpace?.id }),
    create: (kind: "space" | "platform") =>
      run({ kind, spaceId: selectedSpace?.id }),
    createObject,
    placeFixture: () =>
      fixtureId
        ? run({ kind: "place", fixtureId, spaceId: selectedSpace?.id })
        : undefined,
    enclose: () => targetAction("enclose"),
    duplicate: () => targetAction("duplicate"),
    remove,
    requestDelete: () => targetAction("delete"),
  };
}
