import { useEffect, useRef, useState } from "react";
import type {
  RefObject,
  Dispatch,
  SetStateAction,
  PointerEvent as ReactPointerEvent,
} from "react";
import type { ProjectView } from "../../application-host";
import type {
  FixturePlacement,
  StageObject,
  StageSelection,
  SpatialVector3,
  StageView,
} from "../../stage-types";
import { decimal, selectedStage, translated } from "../../stage-tools";
import { resizedByHandle } from "../../stage-geometry";
import { selectInBox } from "../../placement-tools";
import { movementBlocker } from "../../stage-locks";
import { targetKey } from "./stage-selection";
import { translationProblem } from "./object-translation";
import type { Camera } from "./usePlanCamera";
export type Gesture = {
  pointer: number;
  clientX: number;
  clientY: number;
  origin: [number, number];
  camera: Camera;
  object: StageObject | null;
  target: StageSelection | null;
  dx: number;
  dy: number;
  handle: string | null;
  mode: "pan" | "object" | "box" | "click" | "measure";
  fixtures: FixturePlacement[];
  additive: boolean;
  targets: StageSelection[];
  source: string;
};

interface Options {
  project: ProjectView;
  shown: StageView;
  visibleIds: string[];
  targets: StageSelection[];
  svg: RefObject<SVGSVGElement | null>;
  camera: Camera;
  height: number;
  setCamera: Dispatch<SetStateAction<Camera>>;
  tool: "select" | "move" | "pan" | "measure";
  snap: boolean;
  busy: boolean;
  pending: boolean;
  placementEditing: boolean;
  onSelect(
    target: StageSelection,
    additive?: boolean,
    preserve?: boolean,
  ): void;
  onSelectPlacements(ids: string[], additive?: boolean): void;
  onMovePlacements(placements: FixturePlacement[]): void;
  onMove(object: StageObject): void;
  onTranslate?(targets: StageSelection[], delta: SpatialVector3): void;
  onGesture(value: boolean): void;
  setMeasurement(value: { from: [number, number]; to: [number, number] }): void;
  setBlocked(value: string): void;
}
export function usePlanGesture({
  project,
  shown,
  visibleIds,
  targets,
  svg,
  camera,
  height,
  setCamera,
  tool,
  snap,
  busy,
  pending,
  placementEditing,
  onSelect,
  onSelectPlacements,
  onMovePlacements,
  onMove,
  onTranslate,
  onGesture,
  setMeasurement,
  setBlocked,
}: Options) {
  const [gesture, setGesture] = useState<Gesture | null>(null),
    active = useRef<Gesture | null>(null);
  useEffect(
    () => () => {
      if (active.current) {
        active.current = null;
        onGesture(false);
      }
    },
    [],
  );
  function world(e: { clientX: number; clientY: number }): [number, number] {
    const rect = svg.current!.getBoundingClientRect();
    return [
      camera.x +
        ((e.clientX - rect.left - rect.width / 2) / rect.width) * camera.width,
      camera.y -
        ((e.clientY - rect.top - rect.height / 2) / rect.height) * height,
    ];
  }
  function start(e: ReactPointerEvent<SVGSVGElement>) {
    if (busy || active.current || e.button !== 0) return;
    if (
      pending &&
      !(placementEditing && (tool === "pan" || tool === "measure"))
    )
      return;
    setBlocked("");
    svg.current!.focus({ preventScroll: true });
    const hit = (e.target as Element).closest<SVGElement>(
      "[data-kind][data-id]",
    );
    const target = hit
      ? ({ kind: hit.dataset.kind!, id: hit.dataset.id! } as StageSelection)
      : null;
    const object = target ? selectedStage(project.stage, target) : null;
    const additive = e.shiftKey || e.metaKey || e.ctrlKey;
    const mode =
      tool === "measure"
        ? "measure"
        : tool === "pan"
          ? "pan"
          : tool === "select"
            ? target?.kind === "placement"
              ? "click"
              : "box"
            : additive && target
              ? "click"
              : object &&
                  (object.kind !== "construction" ||
                    object.value.shape.kind !== "enclosure")
                ? "object"
                : "pan";
    if (target && (mode === "click" || mode === "object"))
      onSelect(target, additive, mode === "object" && !additive);
    const fixtures =
      mode === "object" && target?.kind === "placement"
        ? shown.placements.filter((p) =>
            visibleIds.includes(target.id)
              ? visibleIds.includes(p.fixtureId)
              : p.fixtureId === target.id,
          )
        : [];
    const group =
      mode === "object" && target
        ? targets.some((t) => targetKey(t) === targetKey(target))
          ? targets
          : [target]
        : [];
    const mixedProblem =
      group.length > 1 ? translationProblem(project.stage, group) : "";
    if (
      mode === "object" &&
      (mixedProblem || movementBlocker(project.stage, group))
    ) {
      setBlocked(mixedProblem || "本次移动涉及已锁定的对象，请先解锁");
      return;
    }
    const origin = world(e);
    if (mode === "measure" && snap && !e.altKey) {
      origin[0] = Math.round(origin[0] * 10) / 10;
      origin[1] = Math.round(origin[1] * 10) / 10;
    }
    const g: Gesture = {
      pointer: e.pointerId,
      clientX: e.clientX,
      clientY: e.clientY,
      origin,
      camera: { ...camera },
      object: mode === "object" ? object : null,
      target,
      dx: 0,
      dy: 0,
      handle: mode === "object" ? (hit?.dataset.handle ?? null) : null,
      mode,
      fixtures,
      additive,
      targets: group,
      source: JSON.stringify([project.id, project.stage]),
    };
    active.current = g;
    setGesture(g);
    svg.current!.setPointerCapture(e.pointerId);
  }
  function move(e: ReactPointerEvent<SVGSVGElement>) {
    const g = active.current;
    if (!g || g.pointer !== e.pointerId) return;
    const rect = svg.current!.getBoundingClientRect();
    let dx = ((e.clientX - g.clientX) / rect.width) * g.camera.width,
      dy = (-(e.clientY - g.clientY) / rect.width) * g.camera.width;
    if (g.mode === "click") return;
    if (g.mode === "pan") {
      setCamera({ ...g.camera, x: g.camera.x - dx, y: g.camera.y - dy });
      return;
    }
    if (
      Math.hypot(e.clientX - g.clientX, e.clientY - g.clientY) < 3 &&
      g.dx === 0 &&
      g.dy === 0
    )
      return;
    if ((g.mode === "object" || g.mode === "measure") && e.shiftKey) {
      if (Math.abs(dx) > Math.abs(dy)) dy = 0;
      else dx = 0;
    }
    if ((g.mode === "object" || g.mode === "measure") && snap && !e.altKey) {
      dx = Math.round(dx * 10) / 10;
      dy = Math.round(dy * 10) / 10;
    }
    const next = { ...g, dx, dy };
    active.current = next;
    setGesture(next);
    if (g.mode !== "measure") onGesture(true);
  }
  function end(commit: boolean) {
    const g = active.current;
    if (!g) return;
    active.current = null;
    setGesture(null);
    onGesture(false);
    if (svg.current?.hasPointerCapture(g.pointer))
      svg.current.releasePointerCapture(g.pointer);
    if (!commit || g.source !== JSON.stringify([project.id, project.stage])) {
      if (g.mode === "pan") setCamera(g.camera);
      return;
    }
    if (g.mode === "measure")
      setMeasurement({
        from: g.origin,
        to: [g.origin[0] + g.dx, g.origin[1] + g.dy],
      });
    if (g.mode === "box") {
      if (g.dx !== 0 || g.dy !== 0)
        onSelectPlacements(
          selectInBox(shown.placements, g.origin, [
            g.origin[0] + g.dx,
            g.origin[1] + g.dy,
          ]),
          g.additive,
        );
      else if (g.target) onSelect(g.target, g.additive);
      else onSelectPlacements([]);
    }
    if (g.object && (g.dx !== 0 || g.dy !== 0)) {
      if (
        !g.handle &&
        onTranslate &&
        g.targets.length &&
        g.targets.every((t) => t.kind !== "space")
      )
        onTranslate(g.targets, { x: decimal(g.dx), y: decimal(g.dy), z: "0" });
      else if (g.fixtures.length)
        onMovePlacements(
          g.fixtures.map((p) => {
            const moved = translated(
              { kind: "placement", value: p },
              g.dx,
              g.dy,
            );
            return (moved as { kind: "placement"; value: FixturePlacement })
              .value;
          }),
        );
      else
        onMove(
          g.handle
            ? resizedByHandle(g.object, g.handle, g.dx, g.dy)
            : translated(g.object, g.dx, g.dy),
        );
    }
  }

  return { gesture, active, world, start, move, end };
}
