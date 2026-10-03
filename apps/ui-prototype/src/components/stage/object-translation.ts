import type {
  StageObject,
  StageSelection,
  StageView,
  SpatialVector3,
} from "../../stage-types.ts";
import { decimal, selectedStage } from "../../stage-tools.ts";
import {
  lockTargets,
  movementBlocker,
  stageTarget,
} from "../../stage-locks.ts";
import { targetKey } from "./stage-selection.ts";
export const zeroTranslation = (): SpatialVector3 => ({
  x: "0",
  y: "0",
  z: "0",
});
export function translationProblem(
  stage: StageView,
  targets: StageSelection[],
) {
  if (
    !targets.length ||
    targets.length > 256 ||
    new Set(targets.map(targetKey)).size !== targets.length
  )
    return "一次移动需要 1–256 个不重复的场地对象";
  for (const target of targets) {
    const object = selectedStage(stage, target);
    if (!object) return "所选对象已不存在，请重新选择";
    if (
      object.kind === "space" ||
      (object.kind === "construction" &&
        object.value.shape.kind === "enclosure")
    )
      return "空间和围护需要单独编辑，请从本组中取消选择";
  }
  if (movementBlocker(stage, targets))
    return "本次移动涉及已锁定的对象或挂接灯具，请先解锁";
  return "";
}
export function affectedTargets(stage: StageView, targets: StageSelection[]) {
  const keys = new Set(targets.map(targetKey));
  const result = [...targets];
  for (const a of stage.attachments) {
    const target: StageSelection = { kind: "placement", id: a.fixtureId };
    if (
      keys.has(`construction:${a.constructionId}`) &&
      !keys.has(targetKey(target))
    ) {
      keys.add(targetKey(target));
      result.push(target);
    }
  }
  return result;
}
export function translationCommand(
  stage: StageView,
  targets: StageSelection[],
  delta: SpatialVector3,
) {
  const problem = translationProblem(stage, targets);
  if (problem) throw new Error(problem);
  for (const [axis, value] of Object.entries(delta)) {
    if (
      !/^-?\d+(?:\.\d{1,6})?$/.test(value) ||
      value.length > 32 ||
      Math.abs(Number(value)) > 200000
    )
      throw new Error(
        `${axis.toUpperCase()} 位移需要 −200000～200000 米、最多六位小数的十进制数`,
      );
  }
  return {
    op: "translateObjects" as const,
    targets: lockTargets(targets),
    deltaMeters: { ...delta },
  };
}
/** Display-only preview. Core validates the complete atomic command at commit. */
export function translatedObject(
  object: StageObject,
  delta: SpatialVector3,
): StageObject {
  const copy = structuredClone(object);
  const shift = (value: string, axis: keyof SpatialVector3) =>
    Number(delta[axis]) === 0
      ? value
      : decimal(Number(value) + Number(delta[axis]));
  if (
    copy.kind === "placement" ||
    (copy.kind === "construction" &&
      (copy.value.shape.kind === "rig" || copy.value.shape.kind === "seating"))
  ) {
    const position =
      copy.kind === "placement"
        ? copy.value.positionMeters
        : (copy.value.shape as { positionMeters: SpatialVector3 })
            .positionMeters;
    for (const axis of ["x", "y", "z"] as const)
      position[axis] = shift(position[axis], axis);
  } else if (
    copy.kind === "construction" &&
    copy.value.shape.kind === "platform"
  ) {
    copy.value.shape.outlineMeters = copy.value.shape.outlineMeters.map(
      ([x, y]) => [shift(x, "x"), shift(y, "y")],
    );
    copy.value.shape.baseElevationMeters = shift(
      copy.value.shape.baseElevationMeters,
      "z",
    );
  }
  return copy;
}
export function translationPreview(
  stage: StageView,
  targets: StageSelection[],
  delta: SpatialVector3,
) {
  const keys = new Set(affectedTargets(stage, targets).map(targetKey));
  return (object: StageObject) =>
    keys.has(targetKey(stageTarget(object)))
      ? translatedObject(object, delta)
      : object;
}
