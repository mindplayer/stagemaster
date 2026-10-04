import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import {
  LevelGestureController,
  idleLevelGesture,
} from "../../execution-level-gesture";
import type { ExecutionRequest, ExecutionStatus } from "../../execution-types";

export function useLiveLevels(
  status: ExecutionStatus | null,
  available: boolean,
  busy: boolean,
  request: (command: ExecutionRequest) => Promise<ExecutionStatus | undefined>,
) {
  const latest = useRef({ status, available, busy, request });
  latest.current = { status, available, busy, request };
  const [view, setView] = useState(idleLevelGesture);
  const mounted = useRef(true);
  const [controller] = useState(
    () =>
      new LevelGestureController({
        context: () => latest.current,
        request: (command) => latest.current.request(command),
        publish: (value) => {
          if (mounted.current) setView(value);
        },
        now: () => performance.now(),
        wait: (ms) => new Promise((resolve) => window.setTimeout(resolve, ms)),
      }),
  );
  useLayoutEffect(() => controller.validate(), [controller, status, available]);
  useEffect(() => {
    mounted.current = true;
    const cancel = () =>
      controller.cancel(undefined, "窗口操作已中断，请核对实际电平");
    const visibility = () => {
      if (document.hidden) cancel();
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape" && controller.busy) {
        event.preventDefault();
        event.stopPropagation();
        controller.cancel();
      }
    };
    window.addEventListener("blur", cancel);
    window.addEventListener("keydown", escape, true);
    document.addEventListener("visibilitychange", visibility);
    return () => {
      mounted.current = false;
      controller.cancel();
      window.removeEventListener("blur", cancel);
      window.removeEventListener("keydown", escape, true);
      document.removeEventListener("visibilitychange", visibility);
    };
  }, [controller]);
  const actions = useMemo(
    () => ({
      isBusy: () => controller.busy,
      begin: (source: string) => controller.begin(source),
      change: (source: string, value: number) =>
        controller.change(source, value),
      finish: (source: string) => controller.finish(source),
      cancel: (source?: string, reason?: string) =>
        controller.cancel(source, reason),
    }),
    [controller],
  );
  return { ...actions, view };
}
