export type AudioLoopPlays =
  { kind: "count"; count: number } | { kind: "untilExit" };

export interface AudioLoopRegion {
  id: string;
  name: string;
  startMs: number;
  endMs: number;
  plays: AudioLoopPlays;
  enabled: boolean;
  locked: boolean;
}

/** Native frame consumer owns this state. u64 identities must remain decimal strings. */
export interface AudioPerformancePosition {
  instance: string | null;
  region: number | null;
  pass: string | null;
  exitRequested: boolean;
  pendingExit: { region: number; pass: string; requested: boolean } | null;
  controlProblem: string | null;
  ended: boolean;
  snapshotPending: boolean;
  boundaryMs: number;
  cachedBytes: number;
}
