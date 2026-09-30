import type { ProjectView } from "../src/application-host.ts";
import { rectangle } from "../src/stage-tools.ts";
export function stageProject(): ProjectView {
  return {
    id: "stage-test",
    name: "场地组织验收",
    description: "",
    audio: null,
    profiles: [],
    domains: [],
    scenes: [],
    groups: [],
    presets: [],
    sequences: [],
    fixtures: ["front", "audience", "loose"].map((id) => ({
      id,
      name: id,
      profileId: "profile",
      profileName: "灯具",
      domainId: "domain",
      domainName: "灯光",
      footprint: 1,
      universe: null,
      address: null,
      attributes: [],
    })),
    stage: {
      spaces: ["stage", "audience"].map((id, i) => ({
        id,
        name: i ? "观众区" : "表演区",
        outlineMeters: rectangle(i * 20, 0, 10, 6),
        floorElevationMeters: "0",
        clearHeightMeters: "7",
      })),
      constructions: [
        {
          id: "rig",
          name: "前桁架",
          shape: {
            kind: "rig",
            rigKind: "truss",
            spaceId: "stage",
            positionMeters: { x: "0", y: "0", z: "6" },
            yawDegrees: "0",
            lengthMeters: "8.123456",
            widthMeters: "0.3",
            heightMeters: "0.3",
          },
        },
        ...Array.from({ length: 24 }, (_, i) => ({
          id: `piece-${i}`,
          name: `座椅构件 ${i + 1}`,
          shape: {
            kind: "platform" as const,
            spaceId: "audience",
            outlineMeters: rectangle(20 + i / 2, 1, 0.4, 0.4),
            baseElevationMeters: "0.4",
            heightMeters: "0.05",
          },
        })),
      ],
      placements: ["front", "audience", "loose"].map((fixtureId, i) => ({
        fixtureId,
        spaceId: i === 0 ? "stage" : i === 1 ? "audience" : null,
        positionMeters: { x: String(i * 20), y: "2", z: "5.876543" },
        rotationDegreesXYZ: { x: "0", y: "0", z: "0" },
      })),
      attachments: [{ fixtureId: "front", constructionId: "rig" }],
    },
  };
}
