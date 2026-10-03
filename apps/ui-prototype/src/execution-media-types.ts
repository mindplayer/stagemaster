export type ExecutionAudioOutput = "systemDefault" | "software";
export type ExecutionMediaAction =
  | { kind: "play" | "pause" | "stop" }
  | { kind: "seek"; positionMs: number; playing: boolean };
export interface ExecutionAudioCatalog {
  output: ExecutionAudioOutput;
  durationMs: number;
  group: string;
  seekIncludesEnd: boolean;
}
export interface ExecutionMediaState {
  id: string;
  generation: string;
  status: "Ready" | "Following" | "Paused" | "Lost" | "Stopped";
  positionMs: number;
  control: null | {
    request: string;
    status: "pending" | "applied" | "failed" | "timedOut";
    problem: string | null;
  };
}
export interface ExecutionAudioState {
  output: ExecutionAudioOutput;
  status:
    | "ready"
    | "preparing"
    | "playing"
    | "paused"
    | "stopped"
    | "ended"
    | "failed";
  positionMs: number;
  durationMs: number;
  problem: string | null;
}
