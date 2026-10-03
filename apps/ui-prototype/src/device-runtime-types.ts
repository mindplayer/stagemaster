export interface DeviceProgramKey {
  kind: "scene" | "sequence";
  id: string;
}
export interface DeviceProgram {
  key: DeviceProgramKey;
  name: string;
  loaderBytes: number;
}
export interface DeviceStep {
  id: string;
  name: string;
  number: string;
}
export interface DeviceRunState {
  mode: "operation" | "quiescing" | "maintenance";
  package: string | null;
  selected: DeviceProgramKey | null;
  loaded: DeviceProgramKey | null;
  status: "idle" | "running" | "paused" | "finished" | null;
  instance: string | null;
  step: string | null;
  elapsedMs: string;
  owner: { lease: string; expiresMs: string } | null;
}
export interface DeviceRunReply {
  id: string;
  boot: string;
  revision: string;
  observedMs: string;
  programCount: number;
  stepCount: number;
  body:
    | { kind: "state"; state: DeviceRunState; error: string | null }
    | { kind: "program"; index: number; program: DeviceProgram | null }
    | { kind: "step"; index: number; step: DeviceStep | null };
}
export interface DeviceRunView {
  epoch: number;
  connectionEpoch: number | null;
  peer: {
    device: string;
    boot: string;
    session: string;
    control: boolean;
    installation: boolean;
  } | null;
  pending: boolean;
  lastResponse: DeviceRunReply | null;
  reply: DeviceRunReply | null;
}
export type DeviceRunAction =
  | { kind: "acquire"; takeover: boolean }
  | { kind: "select"; program: DeviceProgramKey }
  | { kind: "start"; step: string }
  | {
      kind:
        | "renew"
        | "release"
        | "load"
        | "pause"
        | "resume"
        | "next"
        | "stop"
        | "beginMaintenance"
        | "cancelMaintenance"
        | "finishMaintenance";
    };
export type DeviceRunRequest =
  | { kind: "connect"; epoch: number; id: string }
  | { kind: "snapshot" | "refresh"; epoch: number }
  | { kind: "catalog" | "step"; epoch: number; revision: string; index: number }
  | { kind: "apply"; epoch: number; revision: string; action: DeviceRunAction };
export type DeviceRunPort = (
  request: DeviceRunRequest,
) => Promise<DeviceRunView>;
