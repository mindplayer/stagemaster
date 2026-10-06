import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type {
  ExecutionPort,
  ExecutionRequest,
  ExecutionStatus,
} from "../../execution-types";
import {
  ExecutionRequestScope,
  requestExecution,
  executionRequestLane,
  type ExecutionRequestLane,
} from "../../execution-request-scope";

interface Observation {
  scope: ExecutionRequestScope;
  epoch: number;
  status: ExecutionStatus | null;
  actionError: string;
  observationError: string;
  fresh: boolean;
  working: boolean;
}
const empty = (scope: ExecutionRequestScope): Observation => ({
  scope,
  epoch: scope.epoch,
  status: null,
  actionError: "",
  observationError: "",
  fresh: false,
  working: false,
});
function currentView(
  value: Observation,
  scope: ExecutionRequestScope,
): Observation {
  if (value.scope !== scope) return empty(scope);
  if (value.epoch !== scope.epoch)
    return { ...empty(scope), status: value.status };
  return value;
}

export function useExecution(port: ExecutionPort, visible: boolean) {
  const lanes = useRef(new WeakMap<ExecutionPort, ExecutionRequestLane>());
  const scope = useMemo(() => {
    let lane = lanes.current.get(port);
    if (!lane) {
      lane = executionRequestLane();
      lanes.current.set(port, lane);
    }
    return new ExecutionRequestScope(port, lane);
  }, [port]);
  scope.display(visible);
  const selected = useRef(scope);
  selected.current = scope;
  const [observation, setObservation] = useState(() => empty(scope));
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      scope.invalidate();
    };
  }, [scope]);
  const request = useCallback(
    async (command: ExecutionRequest, background = false) => {
      const update = (change: Partial<Observation>) =>
        setObservation((value) => ({
          ...currentView(value, scope),
          ...change,
        }));
      return requestExecution(scope, command, background, {
        current: () => mounted.current && selected.current === scope,
        received: (status, polling) =>
          update({
            status,
            fresh: true,
            observationError: "",
            ...(!polling ? { actionError: "" } : {}),
          }),
        failed: (message, polling) =>
          update({
            fresh: false,
            ...(polling
              ? { observationError: message }
              : { actionError: message }),
          }),
        working: (value) => update({ working: value }),
      });
    },
    [scope],
  );
  useEffect(() => {
    if (!visible) {
      return;
    }
    void request({ kind: "snapshot" }, true);
    const timer = window.setInterval(
      () => void request({ kind: "snapshot" }, true),
      750,
    );
    return () => window.clearInterval(timer);
  }, [visible, request]);
  const view = currentView(observation, scope);
  return {
    status: view.status,
    error: view.actionError || view.observationError,
    working: view.working || scope.changing,
    fresh: view.fresh && visible,
    request,
  };
}
