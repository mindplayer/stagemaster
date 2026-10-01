export type PrevisSource =
  | { kind: "defaults" }
  | { kind: "scene"; sceneId: string }
  | { kind: "playback" };
export type PrevisRequest =
  | { kind: "status" | "enable" | "disable" }
  | { kind: "source"; generation: number; source: PrevisSource };
export interface PrevisStatus {
  enabled: boolean;
  connected: boolean;
  port: number | null;
  viewerUrl: string | null;
  source: PrevisSource;
  problem: string | null;
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
