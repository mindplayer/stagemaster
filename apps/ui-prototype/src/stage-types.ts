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
export type ConstructionShape =
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
export interface StageView {
  spaces: StageSpace[];
  constructions: StageConstruction[];
  placements: FixturePlacement[];
}
export type StageEdit =
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
  | { op: "removeConstruction"; id: string }
  | { op: "putPlacement"; placement: FixturePlacement }
  | { op: "removePlacement"; fixtureId: string };
export type StageObject =
  | { kind: "space"; value: StageSpace }
  | { kind: "construction"; value: StageConstruction }
  | { kind: "placement"; value: FixturePlacement };
export type StageSelection = { kind: StageObject["kind"]; id: string };
