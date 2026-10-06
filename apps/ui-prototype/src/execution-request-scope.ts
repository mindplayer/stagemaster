import type {
  ExecutionPort,
  ExecutionRequest,
  ExecutionStatus,
} from "./execution-types.ts";

export interface ExecutionRequestLane {
  inflight: Promise<void> | null;
  changing: boolean;
}
export const executionRequestLane = (): ExecutionRequestLane => ({
  inflight: null,
  changing: false,
});
/** Each UI lifetime owns replies; the same port retains its single serial lane across A→B→A. */
export class ExecutionRequestScope {
  readonly port: ExecutionPort;
  readonly lane: ExecutionRequestLane;
  visible = false;
  epoch = 0;
  constructor(port: ExecutionPort, lane = executionRequestLane()) {
    this.port = port;
    this.lane = lane;
  }
  get inflight() {
    return this.lane.inflight;
  }
  set inflight(value: Promise<void> | null) {
    this.lane.inflight = value;
  }
  get changing() {
    return this.lane.changing;
  }
  set changing(value: boolean) {
    this.lane.changing = value;
  }
  display(visible: boolean) {
    if (visible !== this.visible) {
      this.visible = visible;
      this.invalidate();
    }
  }
  invalidate() {
    this.epoch += 1;
  }
}
export interface ExecutionRequestObserver {
  current(): boolean;
  received(status: ExecutionStatus, polling: boolean): void;
  failed(message: string, polling: boolean): void;
  working(value: boolean): void;
}
export async function requestExecution(
  scope: ExecutionRequestScope,
  command: ExecutionRequest,
  background: boolean,
  observer: ExecutionRequestObserver,
): Promise<ExecutionStatus | undefined> {
  const epoch = scope.epoch;
  const current = () =>
    observer.current() && scope.visible && scope.epoch === epoch;
  const polling = background && command.kind === "snapshot";
  if (!current() || scope.changing || (polling && scope.inflight)) return;
  if (!polling) {
    scope.changing = true;
    observer.working(true);
  }
  let result: ExecutionStatus | undefined;
  try {
    // Keep the original read/receipt alive, but recheck context BEFORE submitting an action.
    if (!polling) {
      await scope.inflight;
      if (!current()) return;
    }
    const operation = (async () => {
      try {
        const value = await scope.port(command);
        if (current()) {
          result = value;
          observer.received(value, polling);
        }
      } catch (error) {
        if (current())
          observer.failed(
            error instanceof Error ? error.message : String(error),
            polling,
          );
      }
    })();
    scope.inflight = operation;
    try {
      await operation;
    } finally {
      if (scope.inflight === operation) scope.inflight = null;
    }
    return current() ? result : undefined;
  } finally {
    if (!polling) {
      scope.changing = false;
      if (current()) observer.working(false);
    }
  }
}
