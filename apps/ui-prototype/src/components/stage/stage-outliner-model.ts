import type { ProjectView } from "../../application-host.ts";
import type { StageSelection } from "../../stage-types.ts";
import {
  constructionLayer,
  displayMeters,
  type PlanLayer,
} from "./stage-display.ts";
export interface OutlineMember {
  target: StageSelection;
  name: string;
  space: string | null;
  category: PlanLayer;
  detail: string;
}
export function outlineMembers(project: ProjectView): OutlineMember[] {
  const fixtures = new Map(project.fixtures.map((f) => [f.id, f]));
  const constructions = new Map(
    project.stage.constructions.map((c) => [c.id, c]),
  );
  const attachments = new Map(
    project.stage.attachments.map((a) => [a.fixtureId, a.constructionId]),
  );
  const members: OutlineMember[] = project.stage.constructions.map((c) => ({
    target: { kind: "construction", id: c.id },
    name: c.name,
    space: c.shape.spaceId,
    category: constructionLayer(c),
    detail:
      c.shape.kind === "enclosure"
        ? "墙体与地板"
        : c.shape.kind === "rig"
          ? `${c.shape.rigKind === "truss" ? "桁架" : "灯杆"} · ${displayMeters(c.shape.lengthMeters)} 米 · ${project.stage.attachments.filter((a) => a.constructionId === c.id).length} 台灯`
          : `构件 · 高度 ${displayMeters(c.shape.heightMeters)} 米`,
  }));
  return [
    ...members,
    ...project.stage.placements.map((p) => ({
      target: { kind: "placement" as const, id: p.fixtureId },
      name: fixtures.get(p.fixtureId)?.name ?? "灯具",
      space: p.spaceId,
      category: "fixtures" as const,
      detail: `${constructions.get(attachments.get(p.fixtureId) ?? "")?.name ?? "独立灯位"} · 高度 ${displayMeters(p.positionMeters.z)} 米`,
    })),
  ];
}
export const memberCategories: [PlanLayer, string][] = [
  ["rigs", "桁架与灯杆"],
  ["fixtures", "灯具"],
  ["constructions", "构件"],
];
export const categoryKey = (space: string | null, category: PlanLayer) =>
  `${space ?? "loose"}:${category}`;
