import type { SpatialVector3 } from "./stage-types";
export interface PositionAxis {
  minDegrees: string;
  maxDegrees: string;
  reversed: boolean;
}
export interface PositionModel {
  kind: "intersectingOrthogonal";
  pan: PositionAxis;
  tilt: PositionAxis;
}
export interface FixtureZero {
  panDegrees: string;
  tiltDegrees: string;
}
export type PositionEdit =
  | {
      op: "axes" | "offsetAxes";
      sceneId: string;
      fixtureIds: string[];
      panDegrees: string | null;
      tiltDegrees: string | null;
    }
  | {
      op: "aim";
      sceneId: string;
      fixtureIds: string[];
      targetMeters: SpatialVector3;
      branch: "front" | "back" | null;
    }
  | { op: "flip" | "home"; sceneId: string; fixtureIds: string[] }
  | { op: "calibrate"; fixtureId: string; correction: FixtureZero | null };
