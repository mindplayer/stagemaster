import { FixtureLabelStatus } from "./FixtureLabelStatus";
import { useMemo, useRef, useState, type ReactNode } from "react";
import type { ProjectView } from "../../application-host";
import type { PlanLabelMode } from "../../fixture-plan-display";
import { FixtureLabelControl } from "./FixtureLabelControl";
import { FixturePlanLegend } from "./FixturePlanLegend";
import { StagePlanObjects } from "./StagePlanObjects";
import { planPoints, selectionPoints } from "./plan-focus";
import { fittedCamera, usePlanCamera } from "./usePlanCamera";
import { zoomPlan } from "./plan-navigation";
import { usePlanNavigation } from "./usePlanNavigation";
import "./stage-overview.css";

export function StageOverview({
  project,
  visible,
  viewControls,
}: {
  project: ProjectView;
  visible: boolean;
  viewControls: ReactNode;
}) {
  const [hiddenLabels, setHiddenLabels] = useState(0);
  const svg = useRef<SVGSVGElement>(null);
  const [labels, setLabels] = useState<PlanLabelMode>(
    project.fixtures.length <= 30 ? "name" : "none",
  );
  const stage = project.stage;
  const points = useMemo(() => planPoints(stage), [stage]);
  const fixtureIds = stage.placements.map((p) => p.fixtureId);
  const unplaced = project.fixtures.filter((f) => !fixtureIds.includes(f.id));
  const { camera, setCamera, ratio } = usePlanCamera(svg, points, project.id);
  const height = camera.width / ratio;
  function fit(lightsOnly = false) {
    const target = lightsOnly
      ? selectionPoints(stage, null, fixtureIds)
      : points;
    if (target?.length) setCamera(fittedCamera(target, ratio));
  }
  const handlers = usePlanNavigation({
    svg,
    camera,
    ratio,
    setCamera,
    fit,
    scope: `${project.id}:${visible}`,
  });
  return (
    <section className="stage-overview" aria-label="编排场地查看">
      <header>
        {viewControls}
        <span>场地与灯位 · 只读</span>
      </header>
      <div className="stage-overview-toolbar" aria-label="场地查看工具">
        <button
          disabled={!points.length}
          onClick={() => fit()}
          title="F 或 Home"
        >
          全场
        </button>
        <button
          disabled={!fixtureIds.length}
          onClick={() => fit(true)}
          title="⇧ F"
        >
          全部灯位
        </button>
        <button
          aria-label="缩小场地平面"
          onClick={() => setCamera(zoomPlan(camera, ratio, 1.2))}
        >
          −
        </button>
        <button
          aria-label="放大场地平面"
          onClick={() => setCamera(zoomPlan(camera, ratio, 1 / 1.2))}
        >
          ＋
        </button>
        <FixtureLabelControl value={labels} onChange={setLabels} />
        <span>
          {fixtureIds.length} 台灯位
          {unplaced.length > 0 ? ` · ${unplaced.length} 台未布置` : ""}
        </span>
      </div>
      <div className="stage-overview-canvas">
        <svg
          ref={svg}
          className="stage-canvas"
          role="img"
          aria-label="编排场地平面，可平移和缩放"
          tabIndex={0}
          viewBox={`${camera.x - camera.width / 2} ${-camera.y - height / 2} ${camera.width} ${height}`}
          {...handlers}
        >
          <StagePlanObjects
            project={project}
            stage={stage}
            drawn={(o) => o}
            isSelected={() => false}
            selectedIds={[]}
            unit={camera.width / 100}
            labels={labels}
            onHiddenLabels={setHiddenLabels}
            viewport={{
              x: camera.x - camera.width / 2,
              y: -camera.y - height / 2,
              width: camera.width,
              height,
            }}
          />
        </svg>
        {!points.length && (
          <div className="stage-overview-empty">尚未布置场地与灯位</div>
        )}
      </div>
      <FixturePlanLegend fixtures={project.fixtures} ids={fixtureIds} />
      <FixtureLabelStatus count={hiddenLabels} readOnly />
      <footer>
        <span>拖动平移 · 滚动缩放 · F 全场</span>
        <span>平面不显示灯光效果</span>
      </footer>
    </section>
  );
}
