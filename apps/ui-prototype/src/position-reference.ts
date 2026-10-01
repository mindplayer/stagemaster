import type { SpatialVector3 } from "./stage-types";
import type {
  EditOperation,
  FixtureView,
  ProjectView,
} from "./application-host";
import { FixtureFieldError } from "./fixture-field-error.ts";
import { positionDecimal } from "./position-tools.ts";
export interface ReferencePoint {
  id: string;
  name: string;
  targetMeters: SpatialVector3;
  panValue: number;
  tiltValue: number;
  source: "sceneSetpoint";
}
export type ReferenceCheck =
  | { status: "unavailable"; reason: string }
  | {
      status: "checked";
      distanceAlongMeters: number;
      missMeters: number;
      angleDegrees: number;
      closestPointMeters: [number, number, number];
    };
export interface PositionReferenceView {
  profileId: string;
  profileRevision: string;
  compatible: boolean;
  points: { point: ReferencePoint; check: ReferenceCheck }[];
}
export type ReferenceDraft = { name: string; x: string; y: string; z: string };
export function referenceCapture(
  project: ProjectView,
  fixture: FixtureView,
  sceneId: string,
  draft: ReferenceDraft,
): EditOperation {
  if (!fixture.positioning) throw new Error("请先定义灯具的两轴物理模型");
  if (!project.stage.placements.some((p) => p.fixtureId === fixture.id))
    throw new Error("请先布置灯具的安装位置");
  if (fixture.positionReference?.compatible === false)
    throw new Error("档案已改变，请清除旧参考记录后重新记录");
  const name = draft.name.trim();
  if (!name || [...name].length > 256)
    throw new FixtureFieldError("name", "参考点名称需要 1–256 个字符");
  const points = fixture.positionReference?.points ?? [];
  if (points.some((p) => p.point.name.trim() === name))
    throw new FixtureFieldError("name", "这台灯具已有同名参考点");
  if (points.length >= 16) throw new Error("这台灯具已有 16 个参考点");
  return {
    op: "position",
    command: {
      op: "captureReference",
      fixtureId: fixture.id,
      sceneId,
      name,
      targetMeters: {
        x: positionDecimal(draft.x, "x", "目标 X", 100000),
        y: positionDecimal(draft.y, "y", "目标 Y", 100000),
        z: positionDecimal(draft.z, "z", "目标高度", 100000),
      },
    },
  };
}
