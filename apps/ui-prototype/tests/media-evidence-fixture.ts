import type { ExecutionView } from "../src/execution-types.ts";
import type { MediaHttpEvidence } from "../src/media-operation-evidence.ts";

export function http(status: number | null = 200): MediaHttpEvidence {
  return { status, bodyComplete: status !== null, code: null, problem: null };
}

/** Explicitly synthetic data; no backend, audio, renderer or lighting output. */
export function evidenceView(): ExecutionView {
  return {
    hostId: "fixture",
    sessionId: "controller",
    controlling: true,
    pending: false,
    catalog: {
      projectId: "fixture",
      layout: "layout",
      sources: [],
      physicalOutput: false,
      audio: {
        output: "software",
        durationMs: 5000,
        group: "music",
        seekIncludesEnd: true,
        performanceLoops: true,
      },
    },
    record: {
      serial: "9007199254740993",
      status: "complete",
      outcome: {
        kind: "rejected",
        message: "循环播放目标已变化，请确认当前区段和遍次后重新操作",
        state: null,
      },
    },
    mediaOperation: {
      target: {
        hostId: "fixture",
        revision: "20",
        group: "music",
        generation: "5",
        action: {
          kind: "exitLoop",
          instance: "2",
          region: 0,
          pass: "2",
          requested: true,
        },
      },
      serial: "9007199254740993",
      attempted: true,
      submission: http(),
      receiptRead: http(),
      receipt: {
        serial: "9007199254740993",
        complete: true,
        outcome: "rejected",
        code: "loopTargetChanged",
        mediaRequest: null,
        generation: null,
      },
      notSubmittedReason: null,
    },
    observation: {
      phase: "running",
      fault: null,
      snapshot: {
        cycles: "1",
        missedPeriods: "0",
        state: {
          revision: "20",
          sources: [],
          owner: { sessionId: "controller", expiresMs: "60000" },
          fault: false,
          media: [
            {
              id: "music",
              generation: "5",
              status: "Paused",
              positionMs: 2500,
              control: { request: "7", status: "applied", problem: null },
            },
          ],
          audio: {
            output: "software",
            status: "paused",
            positionMs: 2500,
            durationMs: 5000,
            instance: "2",
            problem: null,
            loopState: {
              region: 0,
              name: "一秒候场",
              pass: "2",
              exitRequested: false,
              pendingExit: null,
            },
          },
        },
      },
    },
  };
}
