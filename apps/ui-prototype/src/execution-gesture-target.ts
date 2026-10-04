import type { ExecutionRequest, ExecutionView } from "./execution-types";

/** Frozen command target; the shared controller still owns scheduling and receipts. */
export interface ExecutionGestureTarget {
  key: string;
  source?: string;
  label: string;
  initial?: number;
  normalize?(value: number): number;
  valid(runtime: ExecutionView): boolean;
  value(runtime: ExecutionView): number | undefined;
  request(hostId: string, revision: string, value: number): ExecutionRequest;
}

export function sourceLevelTarget(source: string): ExecutionGestureTarget {
  return {
    key: source,
    source,
    label: "电平",
    valid: () => true,
    value: (runtime) =>
      runtime.observation.snapshot?.state.sources.find((s) => s.id === source)
        ?.level,
    request: (hostId, revision, value) => ({
      kind: "apply",
      hostId,
      revision,
      source,
      action: { kind: "level", value },
    }),
  };
}
