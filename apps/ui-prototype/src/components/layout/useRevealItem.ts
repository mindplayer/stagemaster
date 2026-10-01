import { useEffect, useRef } from "react";
import { rangeScrollTop } from "./scroll-range";
export interface RevealItem {
  id: string;
  serial: number;
}
/** Explicit editing navigation only. Scroll within the owning panel, never the window. */
export function useRevealItem(
  request: RevealItem | null | undefined,
  visible: boolean,
  busy: boolean,
) {
  const root = useRef<HTMLDivElement>(null);
  const done = useRef<RevealItem | null>(null);
  useEffect(() => {
    if (!request || !visible || busy || done.current === request) return;
    const frame = requestAnimationFrame(() => {
      const host = root.current;
      const row =
        host &&
        [...host.querySelectorAll<HTMLButtonElement>("[data-reveal-id]")].find(
          (item) => item.dataset.revealId === request.id,
        );
      if (!host || !row || row.disabled) return;
      for (
        let parent = row.parentElement;
        parent && host.contains(parent);
        parent = parent.parentElement
      ) {
        const rect = row.getBoundingClientRect(),
          bounds = parent.getBoundingClientRect();
        parent.scrollTop = rangeScrollTop(
          parent.scrollTop,
          parent.clientHeight,
          parent.scrollHeight,
          {
            top: rect.top - bounds.top - parent.clientTop + parent.scrollTop,
            bottom:
              rect.bottom - bounds.top - parent.clientTop + parent.scrollTop,
          },
        );
        if (parent === host) break;
      }
      row.focus({ preventScroll: true });
      done.current = request;
    });
    return () => cancelAnimationFrame(frame);
  }, [request, visible, busy]);
  return root;
}
