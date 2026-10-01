export interface StepScript {
  section: string;
  trigger: string;
  notes: string;
}
export interface StepView {
  script?: StepScript;
  id: string;
  name: string;
  number: string;
  sceneId: string;
  delayMs: number;
  fadeMs: number;
  waitMs: number | null;
}
export interface SequenceView {
  id: string;
  name: string;
  tracking: "inherited" | "isolated";
  repeat: "once" | "loop";
  steps: StepView[];
}
export type StepGroupOperation =
  { kind: "copy" | "move"; beforeId: string | null } | { kind: "remove" };
export type SequenceEdit =
  | {
      kind: "editSteps";
      id: string;
      stepIds: string[];
      operation: StepGroupOperation;
    }
  | { kind: "add"; name: string; sceneId: string }
  | {
      kind: "update";
      id: string;
      name: string;
      tracking: SequenceView["tracking"];
      repeat: SequenceView["repeat"];
    }
  | { kind: "duplicate"; id: string; name: string }
  | { kind: "remove"; id: string }
  | { kind: "insertStep"; id: string; sceneId: string; afterId: string | null }
  | ({ kind: "updateStep"; id: string; stepId: string } & Omit<
      StepView,
      "id" | "script"
    >)
  | {
      kind: "updateStepScript";
      id: string;
      stepId: string;
      script: StepScript | null;
    }
  | { kind: "moveStep"; id: string; stepId: string; index: number }
  | { kind: "duplicateStep" | "removeStep"; id: string; stepId: string };
export interface FixtureOutput {
  id: string;
  name: string;
  address: number;
  attributes: {
    key: string;
    value: number;
    function?: {
      key: string;
      name: string;
      dmxValue: number;
      position: number | null;
    };
  }[];
}
export interface PreviewSnapshot {
  epoch: number;
  controlSerial: number;
  loaded: null | {
    sequenceId: string;
    sceneId: string | null;
    name: string;
    sourceRevision: string;
    status: "idle" | "running" | "paused" | "finished";
    stepId: string | null;
    elapsedMs: number;
    delayMs: number;
    fadeMs: number;
    waitMs: number | null;
    stale: boolean;
    canNext: boolean;
    bufferBytes: number;
    effectBufferBytes: number;
    steps: { id: string; name: string; number: string; script?: StepScript }[];
    output: { universe: number; slots: number[]; fixtures: FixtureOutput[] };
  };
}
export type PreviewCommand =
  | { kind: "execute"; stepId: string }
  | { kind: "next" | "pause" | "resume" | "stop" };
export type PreviewRequest =
  | { kind: "snapshot" }
  | { kind: "loadScene"; generation: number; sceneId: string }
  | { kind: "load"; generation: number; sequenceId: string }
  | { kind: "control"; epoch: number; serial: number; command: PreviewCommand };
