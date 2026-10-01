import { useEffect, useRef, type PointerEvent, type RefObject } from "react";
import type { Camera } from "./usePlanCamera";
import { panPlan, zoomPlan } from "./plan-navigation";

/** Viewing only: never selects, edits a placement or issues a playback command. */
export function usePlanNavigation({
  svg,
  camera,
  ratio,
  setCamera,
  scope,
  fit,
}: {
  svg: RefObject<SVGSVGElement | null>;
  camera: Camera;
  ratio: number;
  setCamera(camera: Camera): void;
  scope: string;
  fit(lightsOnly?: boolean): void;
}) {
  const active = useRef<{
    pointer: number;
    x: number;
    y: number;
    camera: Camera;
  } | null>(null);
  function finish(commit: boolean) {
    const start = active.current;
    if (!start) return;
    active.current = null;
    if (svg.current?.hasPointerCapture(start.pointer))
      svg.current.releasePointerCapture(start.pointer);
    if (!commit) setCamera(start.camera);
  }
  useEffect(() => finish(false), [scope]);
  function move(event: PointerEvent<SVGSVGElement>) {
    const start = active.current;
    if (!start || start.pointer !== event.pointerId) return;
    const width = svg.current!.getBoundingClientRect().width;
    if (width > 0)
      setCamera(
        panPlan(
          start.camera,
          event.clientX - start.x,
          event.clientY - start.y,
          width,
        ),
      );
  }
  return {
    onPointerDown(event: PointerEvent<SVGSVGElement>) {
      if (event.button !== 0 || active.current) return;
      event.preventDefault();
      svg.current!.focus({ preventScroll: true });
      active.current = {
        pointer: event.pointerId,
        x: event.clientX,
        y: event.clientY,
        camera,
      };
      svg.current!.setPointerCapture(event.pointerId);
    },
    onPointerMove: move,
    onPointerUp(event: PointerEvent<SVGSVGElement>) {
      if (active.current?.pointer !== event.pointerId) return;
      move(event);
      finish(true);
    },
    onPointerCancel: () => finish(false),
    onLostPointerCapture: () => finish(false),
    onKeyDown(event: React.KeyboardEvent<SVGSVGElement>) {
      const key = event.key.toLowerCase();
      if (
        ![
          "escape",
          "home",
          "f",
          "+",
          "=",
          "-",
          "arrowup",
          "arrowdown",
          "arrowleft",
          "arrowright",
        ].includes(key)
      )
        return;
      if (event.metaKey || event.ctrlKey || event.altKey) return;
      event.preventDefault();
      event.stopPropagation();
      if (key === "escape") return finish(false);
      if (active.current) return;
      if (key === "home" || key === "f")
        return fit(key === "f" && event.shiftKey);
      if (["+", "=", "-"].includes(key))
        return setCamera(zoomPlan(camera, ratio, key === "-" ? 1.2 : 1 / 1.2));
      setCamera({
        ...camera,
        x:
          camera.x +
          (key === "arrowright" ? 1 : key === "arrowleft" ? -1 : 0) *
            camera.width *
            0.15,
        y:
          camera.y +
          (((key === "arrowup" ? 1 : key === "arrowdown" ? -1 : 0) *
            camera.width) /
            ratio) *
            0.15,
      });
    },
    onWheel(event: React.WheelEvent<SVGSVGElement>) {
      if (active.current) return;
      const rect = svg.current!.getBoundingClientRect();
      if (rect.width <= 0 || rect.height <= 0) return;
      setCamera(
        zoomPlan(
          camera,
          ratio,
          Math.exp(Math.max(-100, Math.min(100, event.deltaY)) * 0.003),
          [
            (event.clientX - rect.left) / rect.width - 0.5,
            0.5 - (event.clientY - rect.top) / rect.height,
          ],
        ),
      );
    },
  };
}
