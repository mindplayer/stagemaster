import { stageProject } from "./stage-organization-fixture";
export function mixedPlanProject() {
  const p = stageProject();
  p.stage.constructions = p.stage.constructions.slice(0, 1);
  p.stage.spaces = p.stage.spaces.slice(0, 1);
  p.stage.constructions.push(
    ...Array.from({ length: 8 }, (_, n) => ({
      id: `seats-${n}`,
      name: `观众座区 ${n + 1} · 很长的中文名称👨‍👩‍👧‍👦`,
      shape: {
        kind: "seating" as const,
        spaceId: "stage",
        positionMeters: {
          x: String(((n % 4) - 1.5) * 2),
          y: String(-2 - Math.floor(n / 4) * 2),
          z: "0",
        },
        yawDegrees: String(n * 35),
        rows: 2,
        columns: 3,
        seatWidthMeters: "0.45",
        seatDepthMeters: "0.45",
        columnSpacingMeters: "0.55",
        rowSpacingMeters: "0.8",
        aisle: null,
      },
    })),
  );
  p.fixtures = Array.from({ length: 24 }, (_, n) => ({
    ...p.fixtures[0],
    id: `f-${n}`,
    name: `光束 ${n + 1}`,
    universe: 1,
    address: n * 8 + 1,
  }));
  p.stage.placements = p.fixtures.map((f, n) => ({
    ...p.stage.placements[0],
    fixtureId: f.id,
    positionMeters: {
      x: String(((n % 8) - 3.5) * 0.7),
      y: String(Math.floor(n / 8) * 1.2),
      z: "6",
    },
  }));
  p.stage.attachments = [];
  return p;
}
