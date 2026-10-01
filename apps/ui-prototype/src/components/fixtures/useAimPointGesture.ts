import {
  useEffect,
  useRef,
  useState,
  type PointerEvent,
  type RefObject,
} from "react";
import type { Camera } from "../stage/usePlanCamera";
import { aimPointAt, type AimPoint, type PlaneRect } from "./aim-point";

/** Only the pointer-up updates a field draft. No project or playback commands. */
export function useAimPointGesture({
  svg,
  camera,
  ratio,
  disabled,
  scope,
  onPick,
}: {
  svg: RefObject<SVGSVGElement | null>;
  camera: Camera;
  ratio: number;
  disabled: boolean;
  scope: string;
  onPick(point: AimPoint): void;
}) {
  const [preview, setPreview] = useState<AimPoint | null>(null);
  const active = useRef<{
    pointer: number;
    rect: PlaneRect;
    camera: Camera;
    ratio: number;
    point: AimPoint;
  } | null>(null);
  function finish(commit: boolean) {
    const gesture = active.current;
    if (!gesture) return;
    active.current = null;
    if (svg.current?.hasPointerCapture(gesture.pointer))
      svg.current.releasePointerCapture(gesture.pointer);
    setPreview(null);
    if (commit) onPick(gesture.point);
  }
  useEffect(() => {
    finish(false);
  }, [scope, disabled, camera.x, camera.y, camera.width, ratio]);
  useEffect(() => {
    const cancel = () => finish(false);
    const resize = new ResizeObserver(() => {
      const start = active.current;
      const rect = svg.current?.getBoundingClientRect();
      if (
        start &&
        rect &&
        (rect.width !== start.rect.width || rect.height !== start.rect.height)
      )
        cancel();
    });
    if (svg.current) resize.observe(svg.current);
    window.addEventListener("blur", cancel);
    document.addEventListener("visibilitychange", cancel);
    return () => {
      resize.disconnect();
      window.removeEventListener("blur", cancel);
      document.removeEventListener("visibilitychange", cancel);
      cancel();
    };
  }, [svg]);
  function move(event: PointerEvent<SVGSVGElement>) {
    const start = active.current;
    if (!start || start.pointer !== event.pointerId) return;
    const rect = event.currentTarget.getBoundingClientRect();
    if (
      ["left", "top", "width", "height"].some(
        (key) =>
          rect[key as keyof PlaneRect] !== start.rect[key as keyof PlaneRect],
      )
    )
      return finish(false);
    const point = aimPointAt(
      start.camera,
      start.ratio,
      start.rect,
      event.clientX,
      event.clientY,
    );
    if (!point) return finish(false);
    {
      start.point = point;
      setPreview(point);
    }
  }
  return {
    preview,
    active: () => active.current !== null,
    cancel: () => finish(false),
    handlers: {
      onPointerDown(event: PointerEvent<SVGSVGElement>) {
        if (disabled || event.button !== 0 || active.current) return;
        const rect = event.currentTarget.getBoundingClientRect();
        const point = aimPointAt(
          camera,
          ratio,
          rect,
          event.clientX,
          event.clientY,
        );
        if (!point) return;
        event.preventDefault();
        event.currentTarget.focus({ preventScroll: true });
        active.current = {
          pointer: event.pointerId,
          rect,
          camera,
          ratio,
          point,
        };
        event.currentTarget.setPointerCapture(event.pointerId);
        setPreview(point);
      },
      onPointerMove: move,
      onPointerUp(event: PointerEvent<SVGSVGElement>) {
        if (active.current?.pointer !== event.pointerId) return;
        move(event);
        finish(!disabled);
      },
      onPointerCancel: () => finish(false),
      onLostPointerCapture: () => finish(false),
    },
  };
}
