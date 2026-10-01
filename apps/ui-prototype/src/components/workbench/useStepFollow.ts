import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { stepScrollTop } from "./step-navigation";

export function useStepFollow({
  scope,
  enabled,
  current,
  next,
  query,
  clearQuery,
}: {
  scope: string;
  enabled: boolean;
  current: string | null;
  next: string | null;
  query: string;
  clearQuery(): void;
}) {
  const viewport = useRef<HTMLDivElement>(null);
  const [armed, setArmed] = useState<string | null>(null);
  const [request, setRequest] = useState<{ scope: string; id: string } | null>(
    null,
  );
  const expectedScroll = useRef<number | null>(null);
  const following = enabled && armed === scope && !query;
  const stop = () => {
    setArmed(null);
    setRequest(null);
  };
  useEffect(() => {
    if (!enabled || query || (armed !== null && armed !== scope)) stop();
  }, [enabled, scope, query, armed]);
  useLayoutEffect(() => {
    const host = viewport.current;
    if (!enabled || query || !host) return;
    const requested = request?.scope === scope ? request.id : null;
    const target = requested ?? (following ? (current ?? next) : null);
    if (!target) return;
    const bounds = (id: string | null) => {
      const row = id ? document.getElementById(`step-${id}`) : null;
      if (!row || !host.contains(row)) return undefined;
      const rect = row.getBoundingClientRect(),
        parent = host.getBoundingClientRect();
      return {
        top: rect.top - parent.top - host.clientTop + host.scrollTop,
        bottom: rect.bottom - parent.top - host.clientTop + host.scrollTop,
      };
    };
    const first = bounds(target);
    if (first) {
      const value = stepScrollTop(
        host.scrollTop,
        host.clientHeight,
        host.scrollHeight,
        first,
        requested ? undefined : bounds(next),
        !!requested,
      );
      host.scrollTop = value;
      expectedScroll.current = host.scrollTop;
    }
    if (request) setRequest(null);
  }, [enabled, following, current, next, query, scope, request]);
  return {
    viewport,
    following,
    stop,
    toggle() {
      if (following) stop();
      else if (enabled && (current || next)) {
        clearQuery();
        setArmed(scope);
      }
    },
    locate(id: string | null) {
      if (!enabled || !id) return;
      setArmed(null);
      clearQuery();
      setRequest({ scope, id });
    },
    onScroll() {
      const actual = viewport.current?.scrollTop;
      if (
        following &&
        actual !== undefined &&
        (expectedScroll.current === null ||
          Math.abs(actual - expectedScroll.current) > 1)
      )
        stop();
    },
  };
}
