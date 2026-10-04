import {
  useEffect,
  useRef,
  type ChangeEvent,
  type KeyboardEvent,
  type PointerEvent,
} from "react";
import type { LiveLevels } from "../../execution-level-gesture";

const adjustKeys = new Set([
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "ArrowDown",
  "Home",
  "End",
  "PageUp",
  "PageDown",
]);

// Keep native range semantics (including accessibility input and keyboard steps).
// Capturing the pointer on the host steals WebKit's internal thumb dragging.
export function useSourceLevelInput(
  id: string,
  enabled: boolean,
  live?: LiveLevels,
) {
  const range = useRef<HTMLInputElement>(null);
  const input = useRef<"pointer" | "keyboard" | "cancelled" | null>(null);
  const dragging = live?.view.key === id && live.view.phase === "dragging";
  const end = live?.finish;
  useEffect(() => {
    if (!dragging && input.current) input.current = "cancelled";
  }, [dragging]);
  useEffect(() => {
    const up = () => {
      if (input.current !== "pointer" && input.current !== "cancelled") return;
      const finish = input.current === "pointer";
      input.current = null;
      if (finish) {
        end?.(id);
        range.current?.focus({ preventScroll: true });
      }
    };
    // Finish even when the user releases outside the native slider.
    window.addEventListener("pointerup", up, true);
    return () => window.removeEventListener("pointerup", up, true);
  }, [end, id]);
  const begin = () => enabled && live?.begin(id);
  const cancel = (reason?: string) => {
    input.current = "cancelled";
    live?.cancel(id, reason);
  };
  return {
    ref: range,
    onPointerDown(e: PointerEvent<HTMLInputElement>) {
      if (!e.isPrimary || e.button !== 0 || !begin()) {
        e.preventDefault();
        return;
      }
      input.current = "pointer";
    },
    onChange(e: ChangeEvent<HTMLInputElement>) {
      const existing = input.current;
      if (existing === "cancelled" || (!existing && !begin())) return;
      live?.change(id, Math.round(Number(e.target.value) * 655.35));
      if (!existing) live?.finish(id);
    },
    onPointerCancel() {
      cancel("拖动已中断，请核对实际电平");
    },
    onKeyDown(e: KeyboardEvent<HTMLInputElement>) {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        cancel();
      } else if (adjustKeys.has(e.key)) {
        if (begin()) input.current = "keyboard";
        else e.preventDefault();
      }
    },
    onKeyUp(e: KeyboardEvent<HTMLInputElement>) {
      if (adjustKeys.has(e.key) && input.current === "keyboard") {
        input.current = null;
        live?.finish(id);
      }
    },
    onBlur() {
      // WebKit may blur during the range's native mouse-down default.
      // Window lifecycle owns pointer cancellation; blur cancels keyboard input.
      if (input.current === "keyboard")
        cancel("焦点已离开推子，请核对实际电平");
    },
  };
}
