import { useCallback, useEffect, useRef, useState } from "react";
import type { ApplicationHost } from "../../application-host";
import type { RecentProject, RecentRequest } from "../../recent-types";

export function useRecentProjects(host: ApplicationHost) {
  const [entries, setEntries] = useState<RecentProject[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const mounted = useRef(false);
  const active = useRef(false);
  const request = useCallback(
    async (request: RecentRequest) => {
      if (active.current) return;
      active.current = true;
      setLoading(true);
      setError("");
      try {
        const next = await host.recent(request);
        if (mounted.current) setEntries(next);
      } catch (reason) {
        if (mounted.current) setError(String(reason));
      } finally {
        active.current = false;
        if (mounted.current) setLoading(false);
      }
    },
    [host],
  );
  useEffect(() => {
    mounted.current = true;
    void request({ kind: "list" });
    return () => {
      mounted.current = false;
    };
  }, [request]);
  return { entries, loading, error, request };
}
