import type {
  StageConstruction,
  StageSelection,
  StageView,
} from "../../stage-types.ts";

export type PlanLayer = "spaces" | "constructions" | "rigs" | "fixtures";
export interface PlanVisibility {
  hiddenSpaces: string[];
  hiddenLayers: PlanLayer[];
}
export const ALL_VISIBLE: PlanVisibility = {
  hiddenSpaces: [],
  hiddenLayers: [],
};
export const planLayers: [PlanLayer, string][] = [
  ["spaces", "空间轮廓"],
  ["constructions", "构件"],
  ["rigs", "桁架"],
  ["fixtures", "灯具"],
];
export function displayMeters(value: string | number) {
  const number = Number(value);
  return Number.isFinite(number)
    ? number.toLocaleString("zh-CN", {
        maximumFractionDigits: 3,
        useGrouping: false,
      })
    : "—";
}
export function constructionLayer(value: StageConstruction): PlanLayer {
  return value.shape.kind === "rig" ? "rigs" : "constructions";
}
export function visibleStage(
  stage: StageView,
  visibility: PlanVisibility,
): StageView {
  const spaceVisible = (id: string | null) =>
    id === null || !visibility.hiddenSpaces.includes(id);
  return {
    spaces: visibility.hiddenLayers.includes("spaces")
      ? []
      : stage.spaces.filter((s) => spaceVisible(s.id)),
    constructions: stage.constructions.filter(
      (c) =>
        spaceVisible(c.shape.spaceId) &&
        !visibility.hiddenLayers.includes(constructionLayer(c)),
    ),
    placements: visibility.hiddenLayers.includes("fixtures")
      ? []
      : stage.placements.filter((p) => spaceVisible(p.spaceId)),
    // Attachments remain authoritative; hidden support does not detach a fixture.
    attachments: stage.attachments,
    editLocks: stage.editLocks,
  };
}
export function revealStageTarget(
  stage: StageView,
  visibility: PlanVisibility,
  target: StageSelection,
): PlanVisibility {
  const construction =
    target.kind === "construction"
      ? stage.constructions.find((c) => c.id === target.id)
      : undefined;
  const space =
    target.kind === "space"
      ? target.id
      : target.kind === "placement"
        ? stage.placements.find((p) => p.fixtureId === target.id)?.spaceId
        : construction?.shape.spaceId;
  const layer: PlanLayer =
    target.kind === "space"
      ? "spaces"
      : target.kind === "placement"
        ? "fixtures"
        : construction
          ? constructionLayer(construction)
          : "constructions";
  const hiddenSpaces = visibility.hiddenSpaces.filter((id) => id !== space);
  const hiddenLayers = visibility.hiddenLayers.filter((key) => key !== layer);
  return hiddenSpaces.length === visibility.hiddenSpaces.length &&
    hiddenLayers.length === visibility.hiddenLayers.length
    ? visibility
    : { hiddenSpaces, hiddenLayers };
}
