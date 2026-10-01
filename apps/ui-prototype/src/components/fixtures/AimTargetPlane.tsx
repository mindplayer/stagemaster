import { useId, useMemo, useRef, useState } from "react";
import type { ProjectView } from "../../application-host";
import { StagePlanObjects } from "../stage/StagePlanObjects";
import { planPoints } from "../stage/plan-focus";
import { fittedCamera, usePlanCamera } from "../stage/usePlanCamera";
import { usePlanNavigation } from "../stage/usePlanNavigation";
import { zoomPlan } from "../stage/plan-navigation";
import { readAimPoint, nudgeAimPoint, pointFields } from "./aim-point";
import { useAimPointGesture } from "./useAimPointGesture";
import "./aim-target.css";

export function AimTargetPlane({
  project,
  fixtureIds,
  x,
  y,
  disabled,
  onChange,
}: {
  project: ProjectView;
  fixtureIds: string[];
  x: string;
  y: string;
  disabled: boolean;
  onChange(point: { x: string; y: string }): void;
}) {
  const svg = useRef<SVGSVGElement>(null),
    helpId = useId();
  const [tool, setTool] = useState<"point" | "pan">("point");
  const points = useMemo(() => planPoints(project.stage), [project.stage]);
  const { camera, setCamera, ratio } = usePlanCamera(svg, points, project.id);
  const target = readAimPoint(x, y),
    height = camera.width / ratio;
  const scope = `${project.id}:${fixtureIds.join(",")}:${tool}:${disabled}`;
  const fit = () => setCamera(fittedCamera(points, ratio));
  const navigation = usePlanNavigation({
    svg,
    camera,
    setCamera,
    ratio,
    scope,
    fit,
  });
  const gesture = useAimPointGesture({
    svg,
    camera,
    ratio,
    scope,
    disabled: disabled || tool !== "point",
    onPick: (point) => onChange(pointFields(point)),
  });
  const shown = gesture.preview ?? target;
  const placed = project.stage.placements.filter((p) =>
    fixtureIds.includes(p.fixtureId),
  ).length;
  const unit = camera.width / 100;
  // Target motion does not need to rebuild seating and fixture geometry.
  const geometry = useMemo(
    () => (
      <StagePlanObjects
        project={project}
        stage={project.stage}
        drawn={(o) => o}
        isSelected={(kind, id) =>
          kind === "placement" && fixtureIds.includes(id)
        }
        selectedIds={fixtureIds}
        unit={unit}
        labels="none"
      />
    ),
    [project, fixtureIds, unit],
  );
  return (
    <section className="aim-target-plane" aria-label="共同目标平面">
      <div className="aim-target-tools">
        <button
          type="button"
          aria-pressed={tool === "point"}
          disabled={disabled}
          onClick={() => setTool("point")}
        >
          选点
        </button>
        <button
          type="button"
          aria-pressed={tool === "pan"}
          disabled={disabled}
          onClick={() => setTool("pan")}
        >
          平移
        </button>
        <button type="button" disabled={disabled} onClick={fit}>
          全场
        </button>
        <button
          type="button"
          disabled={disabled || !target}
          onClick={() => target && setCamera({ ...camera, ...target })}
        >
          定位目标
        </button>
        <button
          type="button"
          aria-label="缩小目标平面"
          disabled={disabled}
          onClick={() => setCamera(zoomPlan(camera, ratio, 1.2))}
        >
          −
        </button>
        <button
          type="button"
          aria-label="放大目标平面"
          disabled={disabled}
          onClick={() => setCamera(zoomPlan(camera, ratio, 1 / 1.2))}
        >
          ＋
        </button>
      </div>
      <svg
        ref={svg}
        className={`stage-canvas aim-canvas is-${tool}`}
        role="group"
        aria-label="共同指向选点平面"
        aria-describedby={helpId}
        aria-disabled={disabled}
        tabIndex={disabled ? -1 : 0}
        viewBox={`${camera.x - camera.width / 2} ${-camera.y - height / 2} ${camera.width} ${height}`}
        {...(tool === "point" ? gesture.handlers : navigation)}
        onPointerDown={(event) => {
          if (!disabled)
            (tool === "point" ? gesture.handlers : navigation).onPointerDown(
              event,
            );
        }}
        onBlur={() => {
          gesture.cancel();
          navigation.onPointerCancel();
        }}
        onWheel={(event) => {
          if (!disabled && !gesture.active()) navigation.onWheel(event);
        }}
        onKeyDown={(event) => {
          if (disabled) return;
          if (tool === "pan") return navigation.onKeyDown(event);
          if (
            event.metaKey ||
            event.ctrlKey ||
            event.altKey ||
            event.nativeEvent.isComposing
          )
            return;
          if (event.key === "Escape" && gesture.active()) {
            event.preventDefault();
            event.stopPropagation();
            gesture.cancel();
            return;
          }
          if (event.key === "Escape") return;
          if (event.key.startsWith("Arrow")) {
            event.preventDefault();
            event.stopPropagation();
            if (target && !gesture.active()) {
              const next = nudgeAimPoint(target, event.key, event.shiftKey);
              if (next) onChange(pointFields(next));
            }
          } else navigation.onKeyDown(event);
        }}
      >
        <g pointerEvents="none" aria-hidden="true">
          {geometry}
          {shown && (
            <g
              className="aim-cross"
              transform={`translate(${shown.x} ${-shown.y})`}
            >
              <circle r={unit * 1.5} strokeWidth={unit * 0.2} />
              <path
                d={`M ${-unit * 2.6},0 H ${unit * 2.6} M 0,${-unit * 2.6} V ${unit * 2.6}`}
                strokeWidth={unit * 0.2}
              />
            </g>
          )}
        </g>
      </svg>
      <div className="aim-target-readout">
        {shown
          ? `目标 X ${shown.x.toFixed(3)} · Y ${shown.y.toFixed(3)} 米`
          : "填写有效 X／Y，或在平面重新选点"}
      </div>
      <small id={helpId}>
        {tool === "point"
          ? "点选或拖动 · 方向键 0.1 米 · ⇧ 0.01 米"
          : "拖动平移 · 方向键移动视图"}{" "}
        · 滚动缩放
      </small>
      {placed < fixtureIds.length && (
        <p className="wb-dim">
          {fixtureIds.length - placed} 台未布置，请先在场地中设置灯位。
        </p>
      )}
    </section>
  );
}
