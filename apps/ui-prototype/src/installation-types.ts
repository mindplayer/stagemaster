export type InstallationPhase =
  | "querying"
  | "transferring"
  | "verifying"
  | "committing"
  | "cancelling"
  | "reconnect"
  | "failed"
  | "installed"
  | "cancelled"
  | "notStarted";
export interface InstallationTask {
  id: string;
  package: {
    projectName: string;
    projectId: string;
    revisionId: string;
    digest: string;
    bytes: number;
    programs: number;
    loaderBytes: number;
    executionSemantics?: number;
  };
  deviceId: string;
  deviceName: string;
  connectionEpoch: number;
  phase: InstallationPhase;
  running: boolean;
  cancelRequested: boolean;
  confirmedBytes: number;
  receipt: { generation: string; digest: string; bytes: number } | null;
  problem: string | null;
}
export interface InstallationView {
  installation: { revision: number; task: InstallationTask | null };
  destination: {
    revision: number;
    epoch: number;
    name: string | null;
    deviceId: string | null;
    allowed: boolean;
    executionSemantics?: number;
    reason: string | null;
  };
}
export type InstallationRequest =
  | { kind: "status" }
  | { kind: "cancel" | "forget"; id: string }
  | { kind: "resume"; id: string; epoch: number };
