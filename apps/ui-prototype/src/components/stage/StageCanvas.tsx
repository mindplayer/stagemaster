import { useEffect, useRef, useState } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";
import {
  ArrowsOutIcon,
  MagnifyingGlassMinusIcon,
  MagnifyingGlassPlusIcon,
} from "@phosphor-icons/react";
import type { ProjectView } from "../../application-host";
import type { StageObject, StageSelection } from "../../stage-types";
import {
  bounds,
  objectOutline,
  selectedStage,
  translated,
} from "../../stage-tools";
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
};
export function StageCanvas({
  project,
  selection,
  busy,
  pending,
  onSelect,
  onMove,
  onGesture,
}: {
  project: ProjectView;
  selection: StageSelection | null;
  busy: boolean;
  pending: boolean;
  onSelect(target: StageSelection): void;
  onMove(object: StageObject): void;
  onGesture(value: boolean): void;
}) {
  const svg = useRef<SVGSVGElement>(null);
  const [camera, setCamera] = useState<Camera>({ x: 4, y: 3, width: 20 });
  const [ratio, setRatio] = useState(1.4);
  const [gesture, setGesture] = useState<Gesture | null>(null),
    active = useRef<Gesture | null>(null);
  const [snap, setSnap] = useState(true);
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
  function fit() {
    if (!allPoints.length) {
      setCamera({ x: 4, y: 3, width: 20 });
      return;
    }
    const b = bounds(allPoints);
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
    if (target) onSelect(target);
    const movable =
      object &&
      (object.kind !== "construction" ||
        object.value.shape.kind !== "enclosure");
    const g: Gesture = {
      pointer: e.pointerId,
      clientX: e.clientX,
      clientY: e.clientY,
      origin: world(e),
      camera: { ...camera },
      object: movable ? object : null,
      target: movable ? target : null,
      dx: 0,
      dy: 0,
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
    if (!g.object) {
      setCamera({ ...g.camera, x: g.camera.x - dx, y: g.camera.y - dy });
      return;
    }
    if (
      Math.hypot(e.clientX - g.clientX, e.clientY - g.clientY) < 3 &&
      g.dx === 0 &&
      g.dy === 0
    )
      return;
    if (e.shiftKey) {
      if (Math.abs(dx) > Math.abs(dy)) dy = 0;
      else dx = 0;
    }
    if (snap && !e.altKey) {
      dx = Math.round(dx * 10) / 10;
      dy = Math.round(dy * 10) / 10;
    }
    const next = { ...g, dx, dy };
    active.current = next;
    setGesture(next);
    onGesture(true);
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
      if (!g.object) setCamera(g.camera);
      return;
    }
    if (g.object && (g.dx !== 0 || g.dy !== 0))
      onMove(translated(g.object, g.dx, g.dy));
  }
  const drawn = (object: StageObject) =>
    gesture?.object &&
    gesture.target?.id ===
      (object.kind === "placement"
        ? object.value.fixtureId
        : object.value.id) &&
    gesture.target.kind === object.kind
      ? translated(object, gesture.dx, gesture.dy)
      : object;
  const isSelected = (kind: StageSelection["kind"], id: string) =>
    selection?.kind === kind && selection.id === id;
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
        <strong>平面布置</strong>
        <label className="stage-check">
          <input
            type="checkbox"
            checked={snap}
            onChange={(e) => setSnap(e.target.checked)}
          />
          吸附 0.1 米
        </label>
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
        <button aria-label="查看全场" onClick={fit}>
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
        onPointerUp={() => end(true)}
        onPointerCancel={() => end(false)}
        onLostPointerCapture={() => end(false)}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            e.preventDefault();
            end(false);
          }
          if (e.key.toLowerCase() === "f") fit();
        }}
        onWheel={(e) => {
          if (active.current) return;
          const factor = Math.exp(
            Math.max(-100, Math.min(100, e.deltaY)) * 0.003,
          );
          setCamera((c) => ({
            ...c,
            width: Math.max(1, Math.min(200000, c.width * factor)),
          }));
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
                Number(space.outlineMeters[0]![0]) +
                (gesture?.target?.id === space.id ? gesture.dx : 0) +
                unit
              }
              y={
                -Number(space.outlineMeters[0]![1]) -
                (gesture?.target?.id === space.id ? gesture.dy : 0) -
                unit
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
              <text y={unit * 2} fontSize={unit * 0.95}>
                {project.fixtures.find((f) => f.id === p.fixtureId)?.name}
              </text>
              <title>
                {project.fixtures.find((f) => f.id === p.fixtureId)?.name}
              </title>
            </g>
          );
        })}
      </svg>
      <footer>
        <span>空白处拖动平移 · 滚动缩放 · Esc 取消</span>
        <span>网格 {step} 米</span>
      </footer>
    </section>
  );
}
