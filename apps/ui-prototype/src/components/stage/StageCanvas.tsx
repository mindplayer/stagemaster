import { usePlanGesture } from "./usePlanGesture";
import {
  translationProblem,
  translationPreview as objectPreview,
  zeroTranslation,
} from "./object-translation";
import type { ObjectTranslationDraft } from "./useObjectTranslation";
import { arrangementStage } from "./arrangement-session";
import { StageCanvasFooter } from "./StageCanvasFooter";
import type { PlanLabelMode } from "../../fixture-plan-display";
import { nudgedPlacements } from "./plan-nudge";
import {
  movementBlocker,
  placementTargets,
  stageTarget,
} from "../../stage-locks";
import { planPreview } from "./plan-preview";
import { planPoints, selectionPoints } from "./plan-focus";
import { StagePlanObjects } from "./StagePlanObjects";
import { StagePlanToolbar } from "./StagePlanToolbar";
import {
  ALL_VISIBLE,
  visibleStage,
  type PlanVisibility,
} from "./stage-display";
import { StageMeasureOverlay } from "./StageMeasureOverlay";
import { useEffect, useRef, useState } from "react";
import { StageSelectionOverlay } from "./StageSelectionOverlay";
import type { ProjectView } from "../../application-host";
import type {
  FixturePlacement,
  StageObject,
  StageSelection,
  SpatialVector3,
} from "../../stage-types";
import { decimal, selectedStage } from "../../stage-tools";
import { usePlanCamera, fittedCamera } from "./usePlanCamera";
export function StageCanvas({
  project,
  visibility = ALL_VISIBLE,
  selection,
  selectedIds,
  selectedTargets,
  translationPreview,
  onTranslate,
  preview,
  placementPreview = null,
  draftConstructionId,
  placementEditing = false,
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
  selectedTargets?: StageSelection[];
  translationPreview?: ObjectTranslationDraft | null;
  onTranslate?(targets: StageSelection[], delta: SpatialVector3): void;
  preview: StageObject | null;
  placementPreview?: FixturePlacement[] | null;
  draftConstructionId?: string;
  placementEditing?: boolean;
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
  const [hiddenLabels, setHiddenLabels] = useState(0);
  const svg = useRef<SVGSVGElement>(null);
  const [blocked, setBlocked] = useState("");
  const [snap, setSnap] = useState(true);
  const [tool, setTool] = useState<"select" | "move" | "pan" | "measure">(
    "select",
  );
  const [measurement, setMeasurement] = useState<{
    from: [number, number];
    to: [number, number];
  } | null>(null);
  const [labels, setLabels] = useState<PlanLabelMode>(
    project.fixtures.length <= 30 ? "name" : "none",
  );
  const shown = arrangementStage(
    visibleStage(project.stage, visibility),
    placementPreview,
  );
  const allPoints = planPoints(shown);
  const visibleIds = selectedIds.filter((id) =>
    shown.placements.some((p) => p.fixtureId === id),
  );
  const targets =
    selectedTargets?.filter((t) => selectedStage(shown, t)) ??
    (selection?.kind === "placement"
      ? placementTargets(visibleIds)
      : selection
        ? [selection]
        : []);
  const fixturesOnly = targets.every((t) => t.kind === "placement");
  const { camera, setCamera, ratio } = usePlanCamera(
    svg,
    allPoints,
    project.id,
  );
  const height = camera.width / ratio;
  const { gesture, active, world, start, move, end } = usePlanGesture({
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
  });
  useEffect(
    () => end(false),
    [visibility, tool, project.id, JSON.stringify(project.stage)],
  );

  const visibleObject = selectedStage(shown, selection);
  const currentObject = visibleObject ? (preview ?? visibleObject) : null;
  function fit(selected = false) {
    const points = selected
      ? targets.length > 1
        ? targets.flatMap(
            (t) => selectionPoints(shown, selectedStage(shown, t), []) ?? [],
          )
        : (selectionPoints(shown, currentObject, visibleIds) ?? allPoints)
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

  const baseDrawn = planPreview(project.stage, preview, gesture);
  const groupDraft =
    gesture?.mode === "object" &&
    !gesture.handle &&
    gesture.targets.every((t) => t.kind !== "space")
      ? {
          targets: gesture.targets,
          delta: { x: decimal(gesture.dx), y: decimal(gesture.dy), z: "0" },
        }
      : translationPreview;
  const drawn = groupDraft
    ? objectPreview(project.stage, groupDraft.targets, groupDraft.delta)
    : baseDrawn;
  const isSelected = (kind: StageSelection["kind"], id: string) =>
    targets.some((t) => t.kind === kind && t.id === id) ||
    (kind === "construction" && draftConstructionId === id);
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
        selectedCount={fixturesOnly ? visibleIds.length : 0}
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
            targets.length &&
            ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(e.key)
          ) {
            e.preventDefault();
            e.stopPropagation();
            const problem = onTranslate
              ? translationProblem(project.stage, targets)
              : "";
            if (problem || movementBlocker(project.stage, targets)) {
              setBlocked(problem || "所选对象包含锁定对象，整组保持原位");
              return;
            }
            setBlocked("");
            if (onTranslate) {
              const step = e.shiftKey ? 1 : 0.1;
              const delta = zeroTranslation();
              if (e.key === "ArrowLeft" || e.key === "ArrowRight")
                delta.x = String(e.key === "ArrowLeft" ? -step : step);
              else delta.y = String(e.key === "ArrowDown" ? -step : step);
              onTranslate(targets, delta);
            } else
              onMovePlacements(
                nudgedPlacements(
                  shown.placements.filter((p) =>
                    visibleIds.includes(p.fixtureId),
                  ),
                  e.key,
                  e.shiftKey,
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
          onHiddenLabels={setHiddenLabels}
          viewport={{
            x: camera.x - camera.width / 2,
            y: -camera.y - height / 2,
            width: camera.width,
            height,
          }}
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
        {tool === "move" && currentObject && targets.length < 2 && (
          <StageSelectionOverlay
            object={drawn(currentObject)}
            unit={unit}
            disabled={
              busy ||
              pending ||
              !!movementBlocker(project.stage, [stageTarget(currentObject)])
            }
            onResize={onMove}
          />
        )}
      </svg>
      <StageCanvasFooter
        hiddenLabels={hiddenLabels}
        fixtures={project.fixtures}
        ids={shown.placements.map((p) => p.fixtureId)}
        blocked={blocked}
        tool={tool}
        measurement={measurement}
        step={step}
        placementEditing={placementEditing}
      />
    </section>
  );
}
