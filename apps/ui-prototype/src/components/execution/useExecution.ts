import { useCallback, useEffect, useRef, useState } from "react";
import type {
  ExecutionPort,
  ExecutionRequest,
  ExecutionStatus,
} from "../../execution-types";

export function useExecution(port: ExecutionPort, visible: boolean) {
  const [status, setStatus] = useState<ExecutionStatus | null>(null);
  const [error, setError] = useState("");
  const [working, setWorking] = useState(false);
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
    async (command: ExecutionRequest) => {
      const mutation = command.kind !== "snapshot";
      if (changing.current || (!mutation && inflight.current)) return;
      if (mutation) {
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
            setError("");
          }
        } catch (e) {
          if (mounted.current)
            setError(e instanceof Error ? e.message : String(e));
        }
      })();
      inflight.current = operation;
      try {
        await operation;
      } finally {
        inflight.current = null;
        if (mutation) {
          changing.current = false;
          if (mounted.current) setWorking(false);
        }
      }
      return result;
    },
    [port],
  );
  useEffect(() => {
    if (!visible) return;
    void request({ kind: "snapshot" });
    const timer = window.setInterval(
      () => void request({ kind: "snapshot" }),
      750,
    );
    return () => window.clearInterval(timer);
  }, [visible, request]);
  return { status, error, working, request };
}
