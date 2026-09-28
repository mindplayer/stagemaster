export type DevicePhase =
  | "idle"
  | "preparing"
  | "scanning"
  | "connecting"
  | "connected"
  | "stopping"
  | "fault"
  | "blocked";
export interface DeviceCandidate {
  id: string;
  name: string;
  rssi: number | null;
}
export interface DeviceDiagnostics {
  selfTest: boolean;
  outputDisabled: boolean;
  uptimeMs: number;
  ticks: number;
  heapUsed: number;
  heapFree: number;
}
export interface DeviceDescription {
  deviceId: string;
  bootId: string;
  model: number;
  modelName: string;
  firmware: string;
  declaredFunctions: string[];
  unknownCapabilities: number;
  authenticationMethod: number;
  limits: {
    packageVersion: number;
    transferVersion: number;
    packageBytes: number;
    programs: number;
    universes: number;
    messageBytes: number;
    chunkBytes: number;
    slotBytes: number;
    loaderBytes: number;
    frameMs: number;
  };
}
export interface DeviceProblem {
  code: string;
  message: string;
  detail: string | null;
}
export interface DeviceSnapshot {
  revision: number;
  epoch: number;
  phase: DevicePhase;
  candidates: DeviceCandidate[];
  truncated: boolean;
  selected: DeviceCandidate | null;
  diagnostics: DeviceDiagnostics | null;
  description: DeviceDescription | null;
  scanPerformed: boolean;
  heartbeatCount: number;
  roundTripMs: number | null;
  lastReplyAgeMs: number | null;
  problem: DeviceProblem | null;
}
export type DeviceRequest =
  | { kind: "status" }
  | { kind: "scan" | "cancel"; epoch: number }
  | { kind: "connect"; epoch: number; id: string };
