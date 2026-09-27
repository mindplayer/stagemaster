import {
  rigOutline,
  previewRigPlacement,
  planeDistance,
} from "../../rigging-tools";
import { StageMeasureOverlay } from "./StageMeasureOverlay";
import { useEffect, useRef, useState } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";
import {
  ArrowsOutIcon,
  MagnifyingGlassMinusIcon,
  MagnifyingGlassPlusIcon,
} from "@phosphor-icons/react";
import { StageSelectionOverlay } from "./StageSelectionOverlay";
import { resizedByHandle } from "../../stage-geometry";
import type { ProjectView } from "../../application-host";
import type {
  FixturePlacement,
  StageObject,
  StageSelection,
} from "../../stage-types";
import {
  bounds,
  objectOutline,
  selectedStage,
  translated,
} from "../../stage-tools";
import { selectInBox } from "../../placement-tools";
type Camera = { x: number; y: number; width: number };
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
  const [camera, setCamera] = useState<Camera>({ x: 4, y: 3, width: 20 });
  const [ratio, setRatio] = useState(1.4);
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
  const [labels, setLabels] = useState(true);
  const height = camera.width / ratio;
  useEffect(() => {
    const el = svg.current;
    if (!el) return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry && entry.contentRect.height > 0)
        setRatio(entry.contentRect.width / entry.contentRect.height);
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  const allPoints: [number, number][] = [
    ...project.stage.spaces.flatMap((s) =>
      s.outlineMeters.map(
        (p) => [Number(p[0]), Number(p[1])] as [number, number],
      ),
    ),
    ...project.stage.constructions.flatMap((c) =>
      c.shape.kind === "platform"
        ? c.shape.outlineMeters.map(
            (p) => [Number(p[0]), Number(p[1])] as [number, number],
          )
        : c.shape.kind === "rig"
          ? rigOutline(c.shape)
          : [],
    ),
    ...project.stage.placements.map(
      (p) =>
        [Number(p.positionMeters.x), Number(p.positionMeters.y)] as [
          number,
          number,
        ],
    ),
  ];
  const currentObject = preview ?? selectedStage(project.stage, selection);
  function fit(selected = false) {
    const groupPoints = project.stage.placements
      .filter((p) => selectedIds.includes(p.fixtureId))
      .map(
        (p) =>
          [Number(p.positionMeters.x), Number(p.positionMeters.y)] as [
            number,
            number,
          ],
      );
    const points =
      selected && groupPoints.length
        ? groupPoints
        : selected && currentObject
          ? (objectOutline(currentObject)?.map(
              (p) => [Number(p[0]), Number(p[1])] as [number, number],
            ) ??
            (currentObject.kind === "placement"
              ? [
                  [
                    Number(currentObject.value.positionMeters.x),
                    Number(currentObject.value.positionMeters.y),
                  ] as [number, number],
                ]
              : currentObject.kind === "construction" &&
                  currentObject.value.shape.kind === "rig"
                ? rigOutline(currentObject.value.shape)
                : []))
          : allPoints;
    if (!points.length) {
      setCamera({ x: 4, y: 3, width: 20 });
      return;
    }
    const b = bounds(points);
    setCamera({
      x: (b.minX + b.maxX) / 2,
      y: (b.minY + b.maxY) / 2,
      width: Math.max(
        8,
        (b.maxX - b.minX) * 1.35,
        (b.maxY - b.minY) * ratio * 1.35,
      ),
    });
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
        ? project.stage.placements.filter((p) =>
            selectedIds.includes(target.id)
              ? selectedIds.includes(p.fixtureId)
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
          selectInBox(project.stage.placements, g.origin, [
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

  const identity = (object: StageObject) =>
    object.kind === "placement" ? object.value.fixtureId : object.value.id;
  const drawn = (object: StageObject): StageObject => {
    if (
      gesture?.fixtures.length &&
      object.kind === "placement" &&
      gesture.fixtures.some((p) => p.fixtureId === object.value.fixtureId)
    )
      return translated(object, gesture.dx, gesture.dy);
    if (
      gesture?.object &&
      gesture.target?.id === identity(object) &&
      gesture.target.kind === object.kind
    )
      return gesture.handle
        ? resizedByHandle(object, gesture.handle, gesture.dx, gesture.dy)
        : translated(object, gesture.dx, gesture.dy);
    if (object.kind === "placement") {
      const attachment = project.stage.attachments.find(
        (a) => a.fixtureId === object.value.fixtureId,
      );
      const rig = project.stage.constructions.find(
        (c) => c.id === attachment?.constructionId,
      );
      const candidate =
        gesture?.object?.kind === "construction" &&
        gesture.object.value.id === rig?.id
          ? translated(gesture.object, gesture.dx, gesture.dy)
          : preview?.kind === "construction" && preview.value.id === rig?.id
            ? preview
            : null;
      if (
        rig?.shape.kind === "rig" &&
        candidate?.kind === "construction" &&
        candidate.value.shape.kind === "rig"
      )
        return {
          kind: "placement",
          value: previewRigPlacement(
            object.value,
            rig.shape,
            candidate.value.shape,
          ),
        };
    }
    return preview &&
      preview.kind === object.kind &&
      identity(preview) === identity(object)
      ? preview
      : object;
  };
  const isSelected = (kind: StageSelection["kind"], id: string) =>
    kind === "placement"
      ? selectedIds.includes(id)
      : selection?.kind === kind && selection.id === id;
  const unit = camera.width / 100;
  const step =
    camera.width > 200 ? 10 : camera.width > 70 ? 5 : camera.width > 30 ? 2 : 1;
  const outline = (object: StageObject) =>
    objectOutline(drawn(object))!
      .map((p) => `${p[0]},${-Number(p[1])}`)
      .join(" ");
  return (
    <section className="stage-canvas-panel">
      <div className="stage-canvas-toolbar">
        <div className="stage-tool-switch" aria-label="布置工具">
          {(
            [
              ["select", "选择"],
              ["move", "移动"],
              ["pan", "平移视图"],
              ["measure", "测距"],
            ] as const
          ).map(([key, label]) => (
            <button
              key={key}
              aria-pressed={tool === key}
              onClick={() => setTool(key)}
            >
              {label}
            </button>
          ))}
        </div>
        <label className="stage-check">
          <input
            type="checkbox"
            checked={snap}
            onChange={(e) => setSnap(e.target.checked)}
          />
          吸附 0.1 米
        </label>
        <label className="stage-check">
          <input
            type="checkbox"
            checked={labels}
            onChange={(e) => setLabels(e.target.checked)}
          />
          名称
        </label>
        <button
          disabled={busy || pending || !project.stage.placements.length}
          onClick={() =>
            onSelectPlacements(project.stage.placements.map((p) => p.fixtureId))
          }
        >
          全选灯位
        </button>
        <button
          disabled={busy || pending || !selectedIds.length}
          onClick={onArrange}
        >
          排列所选 · {selectedIds.length}
        </button>
        {measurement && (
          <button onClick={() => setMeasurement(null)}>清除测距</button>
        )}
        <span />
        <button
          aria-label="缩小场地"
          onClick={() =>
            setCamera((c) => ({
              ...c,
              width: Math.min(200000, c.width * 1.25),
            }))
          }
        >
          <MagnifyingGlassMinusIcon />
        </button>
        <button
          aria-label="放大场地"
          onClick={() =>
            setCamera((c) => ({ ...c, width: Math.max(1, c.width / 1.25) }))
          }
        >
          <MagnifyingGlassPlusIcon />
        </button>
        <button
          aria-label="聚焦所选"
          title="聚焦所选（F）"
          disabled={!currentObject}
          onClick={() => fit(true)}
        >
          聚焦所选
        </button>
        <button aria-label="查看全场" title="查看全场" onClick={() => fit()}>
          <ArrowsOutIcon />
        </button>
      </div>
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
            onSelectPlacements(
              project.stage.placements.map((p) => p.fixtureId),
            );
          }
          if (
            !busy &&
            !pending &&
            !active.current &&
            tool !== "measure" &&
            selectedIds.length &&
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
              project.stage.placements
                .filter((p) => selectedIds.includes(p.fixtureId))
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
        {project.stage.spaces.map((space) => (
          <g
            key={space.id}
            data-kind="space"
            data-id={space.id}
            className={isSelected("space", space.id) ? "is-selected" : ""}
          >
            <polygon
              className="stage-room"
              points={outline({ kind: "space", value: space })}
              strokeWidth={unit * 0.14}
            />
            <text
              className="stage-room-label"
              x={
                Math.min(
                  ...objectOutline(drawn({ kind: "space", value: space }))!.map(
                    (p) => Number(p[0]),
                  ),
                ) + unit
              }
              y={
                -Math.max(
                  ...objectOutline(drawn({ kind: "space", value: space }))!.map(
                    (p) => Number(p[1]),
                  ),
                ) +
                unit * 2
              }
              fontSize={unit * 1.15}
            >
              {space.name}
            </text>
          </g>
        ))}
        {project.stage.constructions
          .filter((c) => c.shape.kind === "platform")
          .map((c) => (
            <g
              key={c.id}
              data-kind="construction"
              data-id={c.id}
              className={isSelected("construction", c.id) ? "is-selected" : ""}
            >
              <polygon
                className="stage-platform"
                points={outline({ kind: "construction", value: c })}
                strokeWidth={unit * 0.15}
              />
              <title>{c.name}</title>
            </g>
          ))}
        {project.stage.constructions
          .filter((c) => c.shape.kind === "rig")
          .map((c) => {
            const item = drawn({ kind: "construction", value: c });
            if (item.kind !== "construction" || item.value.shape.kind !== "rig")
              return null;
            const rig = item.value.shape;
            return (
              <g
                key={c.id}
                data-kind="construction"
                data-id={c.id}
                className={`stage-rig ${isSelected("construction", c.id) ? "is-selected" : ""}`}
              >
                <polygon
                  points={rigOutline(rig)
                    .map((p) => `${p[0]},${-p[1]}`)
                    .join(" ")}
                  strokeWidth={unit * 0.18}
                />
                {labels && (
                  <text
                    x={Number(rig.positionMeters.x)}
                    y={
                      -Number(rig.positionMeters.y) -
                      Number(rig.widthMeters) / 2 -
                      unit * 1.4
                    }
                    fontSize={unit}
                    textAnchor="middle"
                  >
                    {c.name}
                  </text>
                )}
                <title>
                  {c.name} · {rig.lengthMeters} 米 · 标高 {rig.positionMeters.z}{" "}
                  米
                </title>
              </g>
            );
          })}
        {project.stage.placements.map((p) => {
          const moved = drawn({ kind: "placement", value: p });
          if (moved.kind !== "placement") return null;
          return (
            <g
              key={p.fixtureId}
              data-kind="placement"
              data-id={p.fixtureId}
              transform={`translate(${moved.value.positionMeters.x},${-Number(moved.value.positionMeters.y)})`}
              className={`stage-light ${isSelected("placement", p.fixtureId) ? "is-selected" : ""}`}
            >
              <circle r={unit * 0.8} strokeWidth={unit * 0.16} />
              <path
                d={`M ${-unit * 0.4} 0 H ${unit * 0.4} M 0 ${-unit * 0.4} V ${unit * 0.4}`}
                strokeWidth={unit * 0.13}
              />
              {labels && (
                <text y={unit * 2} fontSize={unit * 0.95}>
                  {project.fixtures.find((f) => f.id === p.fixtureId)?.name}
                </text>
              )}
              {isSelected("placement", p.fixtureId) && (
                <text
                  className="stage-selection-number"
                  y={-unit * 1.4}
                  fontSize={unit * 0.9}
                >
                  {selectedIds.indexOf(p.fixtureId) + 1}
                </text>
              )}
              <title>
                {project.fixtures.find((f) => f.id === p.fixtureId)?.name}
              </title>
            </g>
          );
        })}
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
        {tool === "move" && currentObject && selectedIds.length < 2 && (
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
