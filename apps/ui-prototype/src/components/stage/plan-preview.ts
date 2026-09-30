import type {
  FixturePlacement,
  StageObject,
  StageSelection,
  StageView,
} from "../../stage-types.ts";
import { translated } from "../../stage-tools.ts";
import { resizedByHandle } from "../../stage-geometry.ts";
import { previewRigPlacement } from "../../rigging-tools.ts";
interface PreviewGesture {
  object: StageObject | null;
  target: StageSelection | null;
  fixtures: FixturePlacement[];
  dx: number;
  dy: number;
  handle: string | null;
}
export function planPreview(
  stage: StageView,
  preview: StageObject | null,
  gesture: PreviewGesture | null,
) {
  const identity = (object: StageObject) =>
    object.kind === "placement" ? object.value.fixtureId : object.value.id;
  return (object: StageObject): StageObject => {
    if (
      gesture?.fixtures.length &&
      object.kind === "placement" &&
      gesture.fixtures.some((p) => p.fixtureId === object.value.fixtureId)
    )
      return translated(object, gesture.dx, gesture.dy);
    if (
      gesture?.object &&
      gesture.target?.id === identity(object) &&
      gesture.target.kind === object.kind
    )
      return gesture.handle
        ? resizedByHandle(object, gesture.handle, gesture.dx, gesture.dy)
        : translated(object, gesture.dx, gesture.dy);
    if (object.kind === "placement") {
      const attachment = stage.attachments.find(
        (a) => a.fixtureId === object.value.fixtureId,
      );
      const rig = stage.constructions.find(
        (c) => c.id === attachment?.constructionId,
      );
      const candidate =
        gesture?.object?.kind === "construction" &&
        gesture.object.value.id === rig?.id
          ? translated(gesture.object, gesture.dx, gesture.dy)
          : preview?.kind === "construction" && preview.value.id === rig?.id
            ? preview
            : null;
      if (
        rig?.shape.kind === "rig" &&
        candidate?.kind === "construction" &&
        candidate.value.shape.kind === "rig"
      )
        return {
          kind: "placement",
          value: previewRigPlacement(
            object.value,
            rig.shape,
            candidate.value.shape,
          ),
        };
    }
    return preview &&
      preview.kind === object.kind &&
      identity(preview) === identity(object)
      ? preview
      : object;
  };
}
