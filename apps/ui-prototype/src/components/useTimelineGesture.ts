import { useEffect, useRef, useState } from "react";
import type { Dispatch, PointerEvent, RefObject } from "react";
import { dragClip, pointerTime } from "../timeline-interaction";
import type { DragMode } from "../timeline-interaction";
import type { Action, Clip, DemoCue } from "../demo-session";

export type TransientEdit = { id: string; patch: Partial<Clip> } | null;
type Gesture = {
  pointerId: number;
  target: HTMLElement;
  x: number;
  originX: number;
  originScroll: number;
  shift: boolean;
  moved: boolean;
  beforeTime: number;
  anchored: boolean;
  clip?: Clip;
  mode: DragMode;
  targets: number[];
  patch?: Clip;
};

/** Keep only the latest input; paint once per animation frame, commit once on release. */
export function useTimelineGesture(options: {
  cue: DemoCue;
  view: string;
  time: number;
  snap: boolean;
  lane: RefObject<HTMLDivElement | null>;
  scroller: RefObject<HTMLDivElement | null>;
  dispatch: Dispatch<Action>;
  setTransient: (edit: TransientEdit) => void;
  onSeek: (time: number) => void;
  onScrubStart: () => void;
}) {
  const latest = useRef(options);
  latest.current = options;
  const gesture = useRef<Gesture | null>(null);
  const frame = useRef<number | null>(null);
  const lastFrame = useRef(0);
  const [feedback, setFeedback] = useState<{
    dragging: string | null;
    snapTarget: number | null;
  }>({ dragging: null, snapTarget: null });

  function apply() {
    const current = gesture.current;
    const config = latest.current;
    const rect = config.lane.current?.getBoundingClientRect();
    if (!current || !rect) return;
    if (!current.clip) {
      const x = current.anchored
        ? rect.left +
          (current.beforeTime / config.cue.duration) * rect.width +
          current.x -
          current.originX +
          (config.scroller.current?.scrollLeft ?? 0) -
          current.originScroll
        : current.x;
      config.onSeek(pointerTime(x, rect.left, rect.width, config.cue.duration));
      return;
    }
    const delta =
      current.x -
      current.originX +
      (config.scroller.current?.scrollLeft ?? 0) -
      current.originScroll;
    if (!current.moved && Math.abs(delta) < 2) return;
    current.moved = true;
    const result = dragClip(
      current.clip,
      current.mode,
      delta,
      rect.width / config.cue.duration,
      config.cue.duration,
      current.targets,
      config.snap && !current.shift,
    );
    current.patch = result.clip;
    config.setTransient({ id: current.clip.id, patch: result.clip });
    setFeedback({ dragging: current.clip.id, snapTarget: result.snapTarget });
  }

  function paint(now: number) {
    frame.current = null;
    const current = gesture.current;
    if (!current) return;
    const scroll = latest.current.scroller.current;
    let continueScroll = false;
    if (scroll && scroll.scrollWidth > scroll.clientWidth) {
      const rect = scroll.getBoundingClientRect();
      const pressure =
        current.x < rect.left + 24
          ? Math.max(-1, (current.x - rect.left - 24) / 24)
          : current.x > rect.right - 24
            ? Math.min(1, (current.x - rect.right + 24) / 24)
            : 0;
      const before = scroll.scrollLeft;
      const elapsed = Math.min(32, Math.max(0, now - lastFrame.current));
      scroll.scrollLeft += pressure * elapsed * 0.6;
      continueScroll = scroll.scrollLeft !== before;
    }
    lastFrame.current = now;
    apply();
    if (continueScroll) queue();
  }
  function queue() {
    if (frame.current === null) frame.current = requestAnimationFrame(paint);
  }
  function cancelFrame() {
    if (frame.current !== null) cancelAnimationFrame(frame.current);
    frame.current = null;
  }
  function finish(cancel = false, event?: PointerEvent<HTMLElement>) {
    const current = gesture.current;
    if (!current || (event && event.pointerId !== current.pointerId)) return;
    cancelFrame();
    if (event) {
      current.x = event.clientX;
      current.shift = event.shiftKey;
    }
    if (!cancel) {
      // pointerup can precede the scheduled frame. Never lose the release position.
      apply();
      if (current.clip && current.patch)
        latest.current.dispatch({
          type: "patchClip",
          id: current.clip.id,
          patch: current.patch,
        });
    } else if (!current.clip) latest.current.onSeek(current.beforeTime);
    gesture.current = null;
    if (current.target.hasPointerCapture(current.pointerId))
      current.target.releasePointerCapture(current.pointerId);
    latest.current.setTransient(null);
    setFeedback({ dragging: null, snapTarget: null });
  }
  function begin(
    event: PointerEvent<HTMLElement>,
    clip?: Clip,
    mode: DragMode = "move",
    anchored = false,
  ) {
    if (event.button !== 0 || gesture.current) return;
    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.focus({ preventScroll: true });
    event.currentTarget.setPointerCapture(event.pointerId);
    const config = latest.current;
    gesture.current = {
      pointerId: event.pointerId,
      target: event.currentTarget,
      x: event.clientX,
      originX: event.clientX,
      originScroll: config.scroller.current?.scrollLeft ?? 0,
      shift: event.shiftKey,
      moved: false,
      beforeTime: config.time,
      anchored,
      clip,
      mode,
      targets: [
        ...new Set([
          0,
          config.cue.duration,
          config.time,
          ...Array.from(
            { length: Math.floor(config.cue.duration / 5) + 1 },
            (_, i) => i * 5,
          ),
          ...config.cue.clips
            .filter((c) => c.id !== clip?.id)
            .flatMap((c) => [c.start, c.start + c.duration]),
        ]),
      ],
    };
    lastFrame.current = performance.now();
    if (clip) config.dispatch({ type: "selectClip", id: clip.id });
    else {
      config.onScrubStart();
      apply();
    }
    setFeedback({ dragging: clip?.id ?? "playhead", snapTarget: null });
  }
  function move(event: PointerEvent<HTMLElement>) {
    const current = gesture.current;
    if (!current || current.pointerId !== event.pointerId) return;
    current.x = event.clientX;
    current.shift = event.shiftKey;
    queue();
  }

  useEffect(() => {
    const abort = () => finish(true);
    const key = (event: KeyboardEvent) => {
      if (event.key === "Escape" && gesture.current) {
        event.preventDefault();
        event.stopPropagation();
        abort();
      }
    };
    const scroll = () => {
      if (gesture.current) queue();
    };
    const element = latest.current.scroller.current;
    window.addEventListener("blur", abort);
    window.addEventListener("keydown", key, true);
    element?.addEventListener("scroll", scroll, { passive: true });
    return () => {
      abort();
      cancelFrame();
      window.removeEventListener("blur", abort);
      window.removeEventListener("keydown", key, true);
      element?.removeEventListener("scroll", scroll);
    };
  }, [options.cue.id, options.view]);

  return {
    ...feedback,
    begin,
    move,
    end: (e: PointerEvent<HTMLElement>) => finish(false, e),
    cancel: () => finish(true),
    lostCapture: () => {
      if (gesture.current) finish(true);
    },
  };
}
