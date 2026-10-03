export interface SpatialVector3 {
  x: string;
  y: string;
  z: string;
}
export interface StageSpace {
  id: string;
  name: string;
  outlineMeters: [string, string][];
  floorElevationMeters: string;
  clearHeightMeters: string | null;
}
export interface RigShape {
  kind: "rig";
  rigKind: "truss" | "pipe";
  spaceId: string | null;
  positionMeters: SpatialVector3;
  yawDegrees: string;
  lengthMeters: string;
  widthMeters: string;
  heightMeters: string;
}
export interface SeatingShape {
  arc?: { radiusMeters: string } | null;
  kind: "seating";
  spaceId: string | null;
  positionMeters: SpatialVector3;
  yawDegrees: string;
  rows: number;
  columns: number;
  seatWidthMeters: string;
  seatDepthMeters: string;
  columnSpacingMeters: string;
  rowSpacingMeters: string;
  aisle: { afterColumn: number; widthMeters: string } | null;
}
export interface RigAttachment {
  fixtureId: string;
  constructionId: string;
}
export interface RigLayout {
  startMarginMeters: string;
  endMarginMeters: string;
  dropMeters: string;
}
export type ConstructionShape =
  | RigShape
  | SeatingShape
  | {
      kind: "enclosure";
      spaceId: string;
      wallThicknessMeters: string;
      floorThicknessMeters: string;
      ceilingThicknessMeters: string | null;
    }
  | {
      kind: "platform";
      spaceId: string | null;
      outlineMeters: [string, string][];
      baseElevationMeters: string;
      heightMeters: string;
    };
export interface StageConstruction {
  id: string;
  name: string;
  shape: ConstructionShape;
}
export interface FixturePlacement {
  fixtureId: string;
  spaceId: string | null;
  positionMeters: SpatialVector3;
  rotationDegreesXYZ: SpatialVector3;
}
export interface StageEditLock {
  kind: StageObject["kind"];
  targetId: string;
}
export interface StageView {
  editLocks?: StageEditLock[];
  attachments: RigAttachment[];
  spaces: StageSpace[];
  constructions: StageConstruction[];
  placements: FixturePlacement[];
}
export type StageEdit =
  | {
      op: "translateObjects";
      targets: StageEditLock[];
      deltaMeters: SpatialVector3;
    }
  | { op: "setEditLocks"; targets: StageEditLock[]; locked: boolean }
  | ({ op: "putSpace" } & Omit<StageSpace, "id"> & { id: string | null })
  | { op: "duplicateSpace"; id: string; name: string }
  | { op: "removeSpace"; id: string; detachMembers: boolean }
  | {
      op: "putConstruction";
      id: string | null;
      name: string;
      shape: ConstructionShape;
    }
  | { op: "duplicateConstruction"; id: string; name: string }
  | { op: "removeConstruction"; id: string; detachFixtures?: boolean }
  | {
      op: "attachFixtures";
      constructionId: string | null;
      fixtureIds: string[];
      layout: RigLayout | null;
    }
  | { op: "putPlacement"; placement: FixturePlacement }
  | {
      op: "translatePlacements";
      fixtureIds: string[];
      deltaMeters: SpatialVector3;
    }
  | {
      op: "transformPlacements";
      fixtureIds: string[];
      yawDegrees: string;
      spacingScale: string;
    }
  | { op: "removePlacement"; fixtureId: string };
export type StageObject =
  | { kind: "space"; value: StageSpace }
  | { kind: "construction"; value: StageConstruction }
  | { kind: "placement"; value: FixturePlacement };
export type StageSelection = { kind: StageObject["kind"]; id: string };
