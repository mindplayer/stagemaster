import type { ExecutionAction, ExecutionView } from "./execution-types";

/** Frozen command target; the shared controller still owns scheduling and receipts. */
export interface ExecutionGestureTarget {
  key: string;
  source: string;
  label: string;
  initial?: number;
  valid(runtime: ExecutionView): boolean;
  value(runtime: ExecutionView): number | undefined;
  action(value: number): ExecutionAction;
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
    action: (value) => ({ kind: "level", value }),
  };
}
