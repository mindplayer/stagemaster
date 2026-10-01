import {
  createContext,
  useContext,
  useLayoutEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { createPortal } from "react-dom";

export type DockRegion =
  "library" | "viewport" | "inspector" | "editor" | "full";
export type DockTargets = Record<DockRegion, HTMLElement>;
export const DockContext = createContext<DockTargets | null>(null);
export const ViewportRevealContext = createContext<() => void>(() => {});

/** Fixed portal identity; moving or hiding a panel never recreates its editor. */
export function DockPane({
  region,
  visible = true,
  passthrough = false,
  keepConnected = false,
  className = "",
  children,
}: {
  region: DockRegion;
  visible?: boolean;
  passthrough?: boolean;
  keepConnected?: boolean;
  className?: string;
  children: ReactNode;
}) {
  const targets = useContext(DockContext);
  const inline = useRef<HTMLDivElement>(null);
  const [container] = useState(() => {
    const node = document.createElement("div");
    node.className = "dock-pane";
    return node;
  });
  const target =
    targets && !passthrough
      ? visible || keepConnected
        ? targets[region]
        : null
      : undefined;
  useLayoutEffect(() => {
    container.className = `dock-pane ${className}`.trim();
  }, [container, className]);
  useLayoutEffect(() => {
    if (keepConnected) {
      container.dataset.parked = String(!visible);
      container.inert = !visible;
    } else {
      delete container.dataset.parked;
      container.inert = false;
    }
  }, [container, keepConnected, visible]);
  useLayoutEffect(() => {
    (target === undefined ? inline.current : target)?.appendChild(container);
    return () => container.remove();
  }, [target, container]);
  return (
    <>
      <div
        className="dock-inline"
        ref={inline}
        hidden={!!targets && !passthrough}
      />
      {createPortal(children, container)}
    </>
  );
}
