import {
  useEffect,
  useRef,
  useState,
  type RefObject,
  type PointerEvent,
} from "react";
import type { StageView } from "../../stage-types";
import type { Camera } from "../stage/usePlanCamera";
import {
  nearbyFixtures,
  selectionAfterBox,
  type Point,
} from "./selection-model";
interface Gesture {
  pointer: number;
  client: Point;
  from: Point;
  to: Point;
  camera: Camera;
  mode: "select" | "pan";
  additive: boolean;
  moved: boolean;
}
export function useFixturePlanGesture({
  svg,
  stage,
  selected,
  camera,
  ratio,
  setCamera,
  busy,
  onSelect,
  onPick,
  scope,
}: {
  svg: RefObject<SVGSVGElement | null>;
  stage: StageView;
  selected: string[];
  camera: Camera;
  ratio: number;
  setCamera(value: Camera): void;
  busy: boolean;
  onSelect(ids: string[]): void;
  onPick(ids: string[], additive: boolean): void;
  scope: string;
}) {
  const active = useRef<Gesture | null>(null);
  const [gesture, setGesture] = useState<Gesture | null>(null);
  const [tool, setTool] = useState<"select" | "pan">("select");
  useEffect(() => finish(false), [scope, tool]);
  function world(e: { clientX: number; clientY: number }): Point {
    const r = svg.current!.getBoundingClientRect();
    return [
      camera.x + ((e.clientX - r.left - r.width / 2) / r.width) * camera.width,
      camera.y -
        (((e.clientY - r.top - r.height / 2) / r.height) * camera.width) /
          ratio,
    ];
  }
  function start(e: PointerEvent<SVGSVGElement>) {
    if (busy || active.current || e.button !== 0) return;
    svg.current!.focus({ preventScroll: true });
    const p = world(e);
    const value: Gesture = {
      pointer: e.pointerId,
      client: [e.clientX, e.clientY],
      from: p,
      to: p,
      camera,
      mode: tool,
      additive: e.shiftKey || e.metaKey || e.ctrlKey,
      moved: false,
    };
    active.current = value;
    setGesture(value);
    svg.current!.setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent<SVGSVGElement>) {
    const g = active.current;
    if (!g || g.pointer !== e.pointerId) return;
    const dx = e.clientX - g.client[0],
      dy = e.clientY - g.client[1];
    const moved = g.moved || Math.hypot(dx, dy) >= 4;
    const next = { ...g, to: world(e), moved };
    active.current = next;
    setGesture(next);
    if (g.mode === "pan" && moved) {
      const width = svg.current!.getBoundingClientRect().width;
      setCamera({
        ...g.camera,
        x: g.camera.x - (dx / width) * g.camera.width,
        y: g.camera.y + (dy / width) * g.camera.width,
      });
    }
  }
  function finish(commit: boolean) {
    const g = active.current;
    if (!g) return;
    active.current = null;
    setGesture(null);
    if (svg.current?.hasPointerCapture(g.pointer))
      svg.current.releasePointerCapture(g.pointer);
    if (g.mode === "pan") {
      if (!commit) setCamera(g.camera);
      return;
    }
    if (!commit || busy) return;
    if (g.moved)
      onSelect(selectionAfterBox(stage, selected, g.from, g.to, g.additive));
    else {
      const ids = nearbyFixtures(
        stage.placements,
        g.to,
        (camera.width / 100) * 1.05,
      );
      if (ids.length) onPick(ids, g.additive);
      else if (!g.additive) onSelect([]);
    }
  }
  return {
    tool,
    setTool,
    gesture,
    cancel: () => finish(false),
    handlers: {
      onPointerDown: start,
      onPointerMove: move,
      onPointerUp: (e: PointerEvent<SVGSVGElement>) => {
        move(e);
        finish(true);
      },
      onPointerCancel: () => finish(false),
      onLostPointerCapture: () => finish(false),
    },
  };
}
