export type PrevisSource =
  | { kind: "defaults" }
  | { kind: "scene"; sceneId: string }
  | { kind: "playback" }
  | { kind: "background"; hostId: string };
export type PrevisRequest =
  | { kind: "status" | "enable" | "disable" }
  | { kind: "background"; generation: number }
  | { kind: "source"; generation: number; source: PrevisSource };
export interface PrevisStatus {
  enabled: boolean;
  connected: boolean;
  port: number | null;
  viewerUrl: string | null;
  source: PrevisSource;
  problem: string | null;
  background?: { projectName: string; unmodeledFixtures: string[] } | null;
}
import type { FixturePlacement, SpatialVector3 } from "./stage-types";
export interface PrevisPlacement {
  generation: number;
  version: string;
  placement: FixturePlacement;
}
export interface PrevisInteractions {
  selectedIds: string[];
  onSelect(ids: string[], isActive: () => boolean): Promise<boolean>;
  onPrepareMove(): Promise<boolean>;
  onTransform(
    proposal: PrevisTransform,
    isActive: () => boolean,
  ): Promise<boolean>;
  onTranslation(
    proposal: PrevisTranslation,
    isActive: () => boolean,
  ): Promise<boolean>;
}
export interface PrevisTranslation {
  generation: number;
  version: string;
  fixtureIds: string[];
  deltaMeters: SpatialVector3;
}

export interface PrevisTransform {
  generation: number;
  version: string;
  fixtureIds: string[];
  yawDegrees: string;
  spacingScale: string;
}
export type PrevisTool = "horizontal" | "vertical" | "rotate" | "scale";
