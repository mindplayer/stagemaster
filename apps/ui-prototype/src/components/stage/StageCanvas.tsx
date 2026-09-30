import { planPreview } from "./plan-preview";
import { planPoints, selectionPoints } from "./plan-focus";
import { StagePlanObjects } from "./StagePlanObjects";
import { StagePlanToolbar } from "./StagePlanToolbar";
import {
  ALL_VISIBLE,
  visibleStage,
  type PlanVisibility,
} from "./stage-display";
import { planeDistance } from "../../rigging-tools";
import { StageMeasureOverlay } from "./StageMeasureOverlay";
import { useEffect, useRef, useState } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";
import { StageSelectionOverlay } from "./StageSelectionOverlay";
import { resizedByHandle } from "../../stage-geometry";
import type { ProjectView } from "../../application-host";
import type {
  FixturePlacement,
  StageObject,
  StageSelection,
} from "../../stage-types";
import { selectedStage, translated } from "../../stage-tools";
import { selectInBox } from "../../placement-tools";
import { usePlanCamera, fittedCamera, type Camera } from "./usePlanCamera";
type Gesture = {
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
};
export function StageCanvas({
  project,
  visibility = ALL_VISIBLE,
  selection,
  selectedIds,
  preview,
  focusRequest,
  busy,
  pending,
  onSelect,
  onSelectPlacements,
  onMovePlacements,
  onArrange,
  onMove,
  onGesture,
}: {
  project: ProjectView;
  visibility?: PlanVisibility;
  selection: StageSelection | null;
  selectedIds: string[];
  preview: StageObject | null;
  focusRequest: number;
  busy: boolean;
  pending: boolean;
  onSelect(
    target: StageSelection,
    additive?: boolean,
    preserve?: boolean,
  ): void;
  onSelectPlacements(ids: string[], additive?: boolean): void;
  onMovePlacements(placements: FixturePlacement[]): void;
  onArrange(): void;
  onMove(object: StageObject): void;
  onGesture(value: boolean): void;
}) {
  const svg = useRef<SVGSVGElement>(null);
  const [gesture, setGesture] = useState<Gesture | null>(null),
    active = useRef<Gesture | null>(null);
  const [snap, setSnap] = useState(true);
  const [tool, setTool] = useState<"select" | "move" | "pan" | "measure">(
    "select",
  );
  const [measurement, setMeasurement] = useState<{
    from: [number, number];
    to: [number, number];
  } | null>(null);
  const [labels, setLabels] = useState(project.fixtures.length <= 30);
  const shown = visibleStage(project.stage, visibility);
  useEffect(() => end(false), [visibility]);
  const allPoints = planPoints(shown);
  const visibleIds = selectedIds.filter((id) =>
    shown.placements.some((p) => p.fixtureId === id),
  );
  const { camera, setCamera, ratio } = usePlanCamera(
    svg,
    allPoints,
    project.id,
  );
  const height = camera.width / ratio;
  const visibleObject = selectedStage(shown, selection);
  const currentObject = visibleObject ? (preview ?? visibleObject) : null;
  function fit(selected = false) {
    const points = selected
      ? (selectionPoints(shown, currentObject, visibleIds) ?? allPoints)
      : allPoints;
    setCamera(fittedCamera(points, ratio));
  }
  const lastFocus = useRef(0);
  useEffect(() => {
    if (focusRequest !== lastFocus.current) {
      lastFocus.current = focusRequest;
      fit(true);
    }
  }, [focusRequest]);
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
    if (busy || pending || active.current || e.button !== 0) return;
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
            : additive && target?.kind === "placement"
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
    if (!commit) {
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
      else if (g.target) onSelect(g.target);
      else onSelectPlacements([]);
    }
    if (g.object && (g.dx !== 0 || g.dy !== 0)) {
      if (g.fixtures.length)
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

  const drawn = planPreview(project.stage, preview, gesture);
  const isSelected = (kind: StageSelection["kind"], id: string) =>
    kind === "placement"
      ? visibleIds.includes(id)
      : selection?.kind === kind && selection.id === id;
  const unit = camera.width / 100;
  const step =
    camera.width > 200 ? 10 : camera.width > 70 ? 5 : camera.width > 30 ? 2 : 1;
  return (
    <section className="stage-canvas-panel">
      <StagePlanToolbar
        tool={tool}
        onTool={setTool}
        snap={snap}
        onSnap={setSnap}
        labels={labels}
        onLabels={setLabels}
        disabled={busy || pending}
        visibleCount={shown.placements.length}
        selectedCount={visibleIds.length}
        onSelectAll={() =>
          onSelectPlacements(shown.placements.map((p) => p.fixtureId))
        }
        onArrange={onArrange}
        measured={!!measurement}
        onClearMeasure={() => setMeasurement(null)}
        canFocus={!!currentObject}
        onFocus={() => fit(true)}
        onFit={() => fit()}
        onZoom={(factor) =>
          setCamera((c) => ({
            ...c,
            width: Math.max(1, Math.min(200000, c.width * factor)),
          }))
        }
      />
      <svg
        ref={svg}
        className="stage-canvas"
        role="application"
        aria-label="场地平面画布"
        tabIndex={0}
        viewBox={`${camera.x - camera.width / 2} ${-camera.y - height / 2} ${camera.width} ${height}`}
        onPointerDown={start}
        onPointerMove={move}
        onPointerUp={(e) => {
          move(e);
          end(true);
        }}
        onPointerCancel={() => end(false)}
        onLostPointerCapture={() => end(false)}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            e.preventDefault();
            end(false);
            if (tool === "measure") setMeasurement(null);
          }
          if (e.key.toLowerCase() === "f") fit(!e.shiftKey);
          if (
            !busy &&
            !pending &&
            !active.current &&
            (e.metaKey || e.ctrlKey) &&
            e.key.toLowerCase() === "a"
          ) {
            e.preventDefault();
            e.stopPropagation();
            onSelectPlacements(shown.placements.map((p) => p.fixtureId));
          }
          if (
            !busy &&
            !pending &&
            !active.current &&
            tool !== "measure" &&
            visibleIds.length &&
            ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(e.key)
          ) {
            e.preventDefault();
            e.stopPropagation();
            const step = e.shiftKey ? 1 : 0.1,
              dx =
                e.key === "ArrowLeft"
                  ? -step
                  : e.key === "ArrowRight"
                    ? step
                    : 0,
              dy =
                e.key === "ArrowDown" ? -step : e.key === "ArrowUp" ? step : 0;
            onMovePlacements(
              shown.placements
                .filter((p) => visibleIds.includes(p.fixtureId))
                .map(
                  (p) =>
                    (
                      translated({ kind: "placement", value: p }, dx, dy) as {
                        kind: "placement";
                        value: FixturePlacement;
                      }
                    ).value,
                ),
            );
          }
        }}
        onWheel={(e) => {
          if (active.current) return;
          const factor = Math.exp(
            Math.max(-100, Math.min(100, e.deltaY)) * 0.003,
          );
          const at = world(e);
          setCamera((c) => {
            const width = Math.max(1, Math.min(200000, c.width * factor));
            return {
              x: at[0] + ((c.x - at[0]) * width) / c.width,
              y: at[1] + ((c.y - at[1]) * width) / c.width,
              width,
            };
          });
        }}
      >
        <defs>
          <pattern
            id="stage-grid"
            width={step}
            height={step}
            patternUnits="userSpaceOnUse"
          >
            <path
              d={`M ${step} 0 H 0 V ${step}`}
              fill="none"
              stroke="#263440"
              strokeWidth={unit * 0.06}
            />
          </pattern>
        </defs>
        <rect
          x={camera.x - camera.width / 2}
          y={-camera.y - height / 2}
          width={camera.width}
          height={height}
          fill="url(#stage-grid)"
        />
        <StagePlanObjects
          project={project}
          stage={shown}
          drawn={drawn}
          isSelected={isSelected}
          selectedIds={visibleIds}
          unit={unit}
          labels={labels}
        />
        {gesture?.mode === "box" && (gesture.dx !== 0 || gesture.dy !== 0) && (
          <rect
            className="stage-marquee"
            pointerEvents="none"
            x={Math.min(gesture.origin[0], gesture.origin[0] + gesture.dx)}
            y={-Math.max(gesture.origin[1], gesture.origin[1] + gesture.dy)}
            width={Math.abs(gesture.dx)}
            height={Math.abs(gesture.dy)}
            strokeWidth={unit * 0.1}
          />
        )}
        <StageMeasureOverlay
          measurement={
            gesture?.mode === "measure"
              ? {
                  from: gesture.origin,
                  to: [
                    gesture.origin[0] + gesture.dx,
                    gesture.origin[1] + gesture.dy,
                  ],
                }
              : measurement
          }
          unit={unit}
        />
        {tool === "move" && currentObject && visibleIds.length < 2 && (
          <StageSelectionOverlay
            object={drawn(currentObject)}
            unit={unit}
            disabled={busy || pending}
            onResize={onMove}
          />
        )}
      </svg>
      <footer>
        <span>
          {tool === "select"
            ? "拖框选择灯具 · ⇧ 点击增减选择"
            : tool === "move"
              ? "拖动所选灯具整组移动 · ⇧ 锁定方向 · Esc 取消"
              : tool === "measure"
                ? "拖动两点测量平面距离 · ⇧ 锁定方向 · Esc 清除"
                : "拖动平移视图"}{" "}
          {tool !== "measure" && " · 方向键微调"}
        </span>
        <span>
          {measurement
            ? `平面距离 ${planeDistance(measurement.from, measurement.to).distance.toFixed(3)} 米 · `
            : ""}
          网格 {step} 米
        </span>
      </footer>
    </section>
  );
}
