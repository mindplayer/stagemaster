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
import type { FixturePlacement } from "./stage-types";
export interface PrevisPlacement {
  generation: number;
  version: string;
  placement: FixturePlacement;
}
export interface PrevisInteractions {
  selectedId: string;
  onSelect(id: string): Promise<boolean>;
  onPrepareMove(): Promise<boolean>;
  onPlacement(proposal: PrevisPlacement, isActive: () => boolean): Promise<boolean>;
}
