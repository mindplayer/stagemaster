export type ExecutionSelection = { kind: "scene" | "sequence"; id: string };
export interface ExecutionSource {
  id: string;
  name: string;
  priority: number;
  selection: ExecutionSelection | { kind: "manual" };
  steps: { id: string; name: string; number: string }[];
}
export type ExecutionAction =
  | { kind: "start"; step: string }
  | { kind: "pause" | "resume" | "next" | "stop" }
  | { kind: "level"; value: number };
export interface ExecutionView {
  hostId: string;
  catalog: {
    projectId: string;
    layout: string;
    sources: ExecutionSource[];
    physicalOutput: false;
  };
  observation: {
    phase: string;
    fault: string | null;
    snapshot: null | {
      cycles: string;
      missedPeriods: string;
      state: {
        revision: string;
        sources: {
          id: string;
          level: number;
          status: string | null;
          step: string | null;
        }[];
        owner: { sessionId: string; expiresMs: string } | null;
        fault: boolean;
      };
    };
  };
  sessionId: string | null;
  controlling: boolean;
  pending: boolean;
  record: null | {
    serial: string;
    status: string;
    outcome: null | { kind: string; message: string | null };
  };
}
export interface ExecutionStatus {
  phase: "empty" | "connected" | "unavailable" | "closing";
  problem: string | null;
  runtime: ExecutionView | null;
}
export type ExecutionRequest =
  | { kind: "snapshot" | "reconnect" | "release" }
  | { kind: "prepare"; generation: number; selection: ExecutionSelection[] }
  | { kind: "acquire"; takeover: boolean }
  | {
      kind: "apply";
      hostId: string;
      revision: string;
      source: string;
      action: ExecutionAction;
    }
  | { kind: "shutdown"; hostId: string };
export type ExecutionPort = (
  request: ExecutionRequest,
) => Promise<ExecutionStatus>;
