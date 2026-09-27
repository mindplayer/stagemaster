import { type ReactNode, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

/**
 * Keep component/DOM state while parking inactive workspaces outside the document.
 * WKWebView can omit an entire re-shown display:none subtree from its native AX tree.
 * The portal target never changes: only this owned container is attached/detached.
 * Navigation must flush drafts before changing visible, as in the workbench queue.
 */
export function WorkspaceSurface({
  visible,
  className,
  label,
  children,
}: {
  visible: boolean;
  className: string;
  label: string;
  children: ReactNode;
}) {
  const slot = useRef<HTMLDivElement>(null);
  const [surface] = useState(() => document.createElement("div"));
  useLayoutEffect(() => {
    surface.className = className;
    surface.setAttribute("role", "region");
    surface.setAttribute("aria-label", label);
  }, [surface, className, label]);
  useLayoutEffect(() => {
    if (visible) slot.current?.appendChild(surface);
    return () => {
      surface.remove();
    };
  }, [surface, visible]);
  return (
    <>
      <div ref={slot} className="wb-surface-slot" hidden={!visible} />
      {createPortal(children, surface)}
    </>
  );
}
