import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import type { StageSelection } from "../../stage-types";
import { rangeScrollTop } from "../layout/scroll-range";
import { stageTargetKey } from "./stage-outline-navigation";

function reveal(host: HTMLElement, row: HTMLElement, center = false) {
  const rect = row.getBoundingClientRect(),
    parent = host.getBoundingClientRect();
  host.scrollTop = rangeScrollTop(
    host.scrollTop,
    host.clientHeight,
    host.scrollHeight,
    {
      top: rect.top - parent.top - host.clientTop + host.scrollTop,
      bottom: rect.bottom - parent.top - host.clientTop + host.scrollTop,
    },
    undefined,
    center,
  );
}
export function useOutlineNavigation(
  selection: StageSelection | null,
  ancestors: string[],
  busy: boolean,
  clearQuery: () => void,
) {
  const root = useRef<HTMLDivElement>(null);
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const [request, setRequest] = useState(0);
  const previousRequest = useRef(0);
  const key = selection ? stageTargetKey(selection) : "";
  const ancestorKey = ancestors.join("\0");
  useEffect(() => {
    const explicit = request !== previousRequest.current;
    previousRequest.current = request;
    if (!key) return;
    setExpanded((old) => ({
      ...old,
      ...Object.fromEntries(
        ancestorKey
          .split("\0")
          .filter(Boolean)
          .map((k) => [k, true]),
      ),
    }));
    const frame = requestAnimationFrame(() => {
      const host = root.current;
      if (!host?.isConnected) return;
      const row = [
        ...host.querySelectorAll<HTMLButtonElement>("[data-selection]"),
      ].find((el) => el.dataset.selection === key);
      if (!row) return;
      reveal(host, row, explicit);
      if (explicit && !row.disabled) row.focus({ preventScroll: true });
    });
    return () => cancelAnimationFrame(frame);
  }, [key, ancestorKey, request]);
  return {
    root,
    expanded,
    toggle: (key: string, current: boolean) =>
      setExpanded((old) => ({ ...old, [key]: !current })),
    locate() {
      if (busy || !key) return;
      clearQuery();
      setRequest((n) => n + 1);
    },
    onKeyDown(e: KeyboardEvent<HTMLDivElement>) {
      if (
        busy ||
        e.altKey ||
        e.ctrlKey ||
        e.metaKey ||
        e.shiftKey ||
        !["ArrowUp", "ArrowDown", "Home", "End"].includes(e.key)
      )
        return;
      const host = root.current;
      const row = (e.target as HTMLElement).closest<HTMLButtonElement>(
        "[data-selection]",
      );
      if (!host || !row || !host.contains(row)) return;
      const rows = [
        ...host.querySelectorAll<HTMLButtonElement>(
          "[data-selection]:not(:disabled)",
        ),
      ];
      const index = rows.indexOf(row);
      const next =
        e.key === "Home"
          ? 0
          : e.key === "End"
            ? rows.length - 1
            : Math.max(
                0,
                Math.min(
                  rows.length - 1,
                  index + (e.key === "ArrowDown" ? 1 : -1),
                ),
              );
      e.preventDefault();
      e.stopPropagation();
      rows[next]?.focus({ preventScroll: true });
      if (rows[next]) reveal(host, rows[next]);
    },
  };
}
