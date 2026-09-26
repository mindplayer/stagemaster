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
