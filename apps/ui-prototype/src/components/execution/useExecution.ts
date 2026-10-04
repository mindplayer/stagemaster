import { useCallback, useEffect, useRef, useState } from "react";
import type {
  ExecutionPort,
  ExecutionRequest,
  ExecutionStatus,
} from "../../execution-types";

export function useExecution(port: ExecutionPort, visible: boolean) {
  const [status, setStatus] = useState<ExecutionStatus | null>(null);
  const [actionError, setActionError] = useState("");
  const [observationError, setObservationError] = useState("");
  const [working, setWorking] = useState(false);
  const [fresh, setFresh] = useState(false);
  const visibleNow = useRef(visible);
  visibleNow.current = visible;
  const inflight = useRef<Promise<void> | null>(null);
  const changing = useRef(false);
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  const request = useCallback(
    async (command: ExecutionRequest, background = false) => {
      const polling = background && command.kind === "snapshot";
      if (changing.current || (polling && inflight.current)) return;
      if (!polling) {
        changing.current = true;
        setWorking(true);
        // One explicit action waits for the read already in flight; polling cannot swallow clicks.
        await inflight.current;
      }
      let result: ExecutionStatus | undefined;
      const operation = (async () => {
        try {
          const value = await port(command);
          result = value;
          if (mounted.current) {
            setStatus(value);
            setFresh(visibleNow.current);
            setObservationError("");
            if (!polling) setActionError("");
          }
        } catch (e) {
          if (mounted.current) {
            const message = e instanceof Error ? e.message : String(e);
            setFresh(false);
            if (polling) setObservationError(message);
            else setActionError(message);
          }
        }
      })();
      inflight.current = operation;
      try {
        await operation;
      } finally {
        inflight.current = null;
        if (!polling) {
          changing.current = false;
          if (mounted.current) setWorking(false);
        }
      }
      return result;
    },
    [port],
  );
  useEffect(() => {
    if (!visible) {
      setFresh(false);
      return;
    }
    void request({ kind: "snapshot" }, true);
    const timer = window.setInterval(
      () => void request({ kind: "snapshot" }, true),
      750,
    );
    return () => window.clearInterval(timer);
  }, [visible, request]);
  return {
    status,
    error: actionError || observationError,
    working,
    fresh: fresh && visible,
    request,
  };
}
