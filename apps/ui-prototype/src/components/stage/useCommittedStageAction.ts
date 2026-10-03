import { useEffect, useLayoutEffect, useRef, useState } from "react";

/** Preserve click identity, but execute with props rendered after the draft transaction. */
export function useCommittedStageAction<T>({
  scopeId,
  active,
  busy,
  beforeChange,
  execute,
  onError,
}: {
  scopeId: string;
  active: boolean;
  busy: boolean;
  beforeChange(): Promise<boolean>;
  execute(intent: T): void | Promise<void>;
  onError(message: string): void;
}) {
  type Ticket = { scope: object; intent: T; started: boolean };
  const scope = useRef<object | null>(null);
  const current = useRef<Ticket | null>(null);
  const [ready, setReady] = useState<Ticket | null>(null);
  useLayoutEffect(() => {
    scope.current = active ? {} : null;
    return () => {
      scope.current = null;
      current.current = null;
    };
  }, [scopeId, active]);
  function valid(ticket: Ticket) {
    return scope.current === ticket.scope && current.current === ticket;
  }
  useEffect(() => {
    if (!ready || busy || ready.started) return;
    setReady(null);
    if (!valid(ready)) return;
    ready.started = true;
    void (async () => {
      try {
        await execute(ready.intent);
      } catch (e) {
        if (valid(ready)) onError(e instanceof Error ? e.message : String(e));
      } finally {
        if (current.current === ready) current.current = null;
      }
    })();
  }, [ready, busy, execute, onError]);
  return async (intent: T) => {
    if (!scope.current || busy || current.current) return;
    const ticket: Ticket = { scope: scope.current, intent, started: false };
    current.current = ticket;
    try {
      if ((await beforeChange()) && valid(ticket)) setReady(ticket);
      else if (current.current === ticket) current.current = null;
    } catch (e) {
      if (valid(ticket)) onError(e instanceof Error ? e.message : String(e));
      if (current.current === ticket) current.current = null;
    }
  };
}
