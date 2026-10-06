import type {
  ManualFixture,
  ManualEdit,
  ManualLimits,
} from "./execution-manual";
import type {
  ExecutionAudioOutput,
  ExecutionMediaAction,
  ExecutionAudioCatalog,
  ExecutionMediaState,
  ExecutionAudioState,
} from "./execution-media-types";
import type { ExecutionSourceState } from "./execution-source-progress";
import type { MediaOperationEvidence } from "./media-operation-evidence";
import type { SourceOperationEvidence } from "./source-operation-evidence";
export type ExecutionSelection =
  { kind: "scene" | "sequence"; id: string } | { kind: "audioTimeline" };
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
  | { kind: "level"; value: number }
  | { kind: "patch"; changes: ManualEdit[] };
export type ExecutionOutputAction =
  { kind: "level"; percent: number } | { kind: "blackout"; enabled: boolean };
export type ExecutionBatchAction = { kind: "pause" | "resume" | "stop" };
export interface ExecutionBatchRequest {
  kind: "batch";
  hostId: string;
  revision: string;
  sources: string[];
  action: ExecutionBatchAction;
}
export interface ExecutionView {
  hostId: string;
  catalog: {
    projectId: string;
    layout: string;
    sources: ExecutionSource[];
    physicalOutput: false;
    capabilities?: string[];
    fixtures?: ManualFixture[];
    limits?: ManualLimits;
    audio?: ExecutionAudioCatalog;
    output?: { uncontrolledFixtures: number };
  };
  observation: {
    phase: string;
    fault: string | null;
    snapshot: null | {
      cycles: string;
      missedPeriods: string;
      state: {
        revision: string;
        output?: { percent: number; blackout: boolean };
        audio?: ExecutionAudioState;
        media?: ExecutionMediaState[];
        sources: ExecutionSourceState[];
        owner: { sessionId: string; expiresMs: string } | null;
        fault: boolean;
      };
    };
  };
  sessionId: string | null;
  controlling: boolean;
  pending: boolean;
  mediaOperation?: MediaOperationEvidence;
  sourceOperation?: SourceOperationEvidence;
  record: null | {
    serial: string;
    status: string;
    outcome: null | {
      kind: string;
      message: string | null;
      state?: { media?: ExecutionMediaState[] } | null;
    };
  };
}
export interface ExecutionStatus {
  phase: "empty" | "connected" | "unavailable" | "closing";
  problem: string | null;
  runtime: ExecutionView | null;
}
export type ExecutionRequest =
  | ExecutionBatchRequest
  | { kind: "snapshot" | "reconnect" | "release" }
  | {
      kind: "prepare";
      generation: number;
      selection: ExecutionSelection[];
      audioOutput?: ExecutionAudioOutput;
    }
  | { kind: "acquire"; takeover: boolean }
  | {
      kind: "apply";
      hostId: string;
      revision: string;
      source: string;
      action: ExecutionAction;
    }
  | {
      kind: "output";
      hostId: string;
      revision: string;
      action: ExecutionOutputAction;
    }
  | {
      kind: "media";
      hostId: string;
      revision: string;
      group: string;
      generation: string;
      action: ExecutionMediaAction;
    }
  | { kind: "shutdown"; hostId: string };
export type ExecutionPort = (
  request: ExecutionRequest,
) => Promise<ExecutionStatus>;
