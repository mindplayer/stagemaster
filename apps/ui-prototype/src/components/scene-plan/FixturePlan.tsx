import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import type { ProjectView } from "../../application-host";
import { fixtureMatches } from "../../editor-tools";
import { StagePlanObjects } from "../stage/StagePlanObjects";
import { usePlanCamera, fittedCamera } from "../stage/usePlanCamera";
import { planPoints, selectionPoints } from "../stage/plan-focus";
import { useFixturePlanGesture } from "./useFixturePlanGesture";
import { selectionAfterClick } from "./selection-model";
import { FixturePickList } from "./FixturePickList";
import "./fixture-plan.css";
export function FixturePlan({
  project,
  selected,
  query,
  onlySelected,
  busy,
  visible,
  onQuery,
  onFilter,
  onSelect,
  viewControls,
}: {
  project: ProjectView;
  selected: string[];
  query: string;
  onlySelected: boolean;
  busy: boolean;
  visible: boolean;
  onQuery(value: string): void;
  onFilter(value: boolean): void;
  onSelect(ids: string[]): Promise<boolean>;
  viewControls: ReactNode;
}) {
  const svg = useRef<SVGSVGElement>(null);
  const [labels, setLabels] = useState(project.fixtures.length <= 30);
  const [overlap, setOverlap] = useState<{
    ids: string[];
    additive: boolean;
  } | null>(null);
  const [unplacedOpen, setUnplacedOpen] = useState(false);
  const fixtures = project.fixtures.filter(
    (f) =>
      fixtureMatches(f, query) && (!onlySelected || selected.includes(f.id)),
  );
  const ids = fixtures.map((f) => f.id);
  const stage = useMemo(
    () => ({
      ...project.stage,
      placements: project.stage.placements.filter((p) =>
        ids.includes(p.fixtureId),
      ),
    }),
    [project.stage, ids.join("|")],
  );
  const unplaced = ids.filter(
    (id) => !project.stage.placements.some((p) => p.fixtureId === id),
  );
  const points = stage.placements.length
    ? selectionPoints(
        stage,
        null,
        stage.placements.map((p) => p.fixtureId),
      )!
    : planPoints(stage);
  const { camera, setCamera, ratio } = usePlanCamera(svg, points, project.id);
  const height = camera.width / ratio,
    unit = camera.width / 100;
  const scope = `${project.id}:${visible}:${query}:${onlySelected}`;
  useEffect(() => setOverlap(null), [scope]);
  function pick(id: string, additive: boolean) {
    void onSelect(selectionAfterClick(selected, id, additive)).then((ok) => {
      if (ok) {
        setOverlap(null);
        if (overlap) svg.current?.focus({ preventScroll: true });
      }
    });
  }
  const { tool, setTool, gesture, cancel, handlers } = useFixturePlanGesture({
    svg,
    stage,
    selected,
    camera,
    ratio,
    setCamera,
    busy,
    scope,
    onSelect: (ids) => {
      void onSelect(ids);
    },
    onPick: (ids, additive) => {
      if (ids.length === 1) pick(ids[0], additive);
      else setOverlap({ ids, additive });
    },
  });
  const overlapIds = overlap?.ids.filter((id) => ids.includes(id)) ?? [];
  function fit(selectedOnly = false) {
    const focus = selectedOnly
      ? selectionPoints(stage, null, selected)
      : points;
    if (focus?.length) setCamera(fittedCamera(focus, ratio));
  }
  return (
    <section className="fixture-plan" aria-label="场景平面选灯">
      <header className="fixture-plan-header">
        {viewControls}
        <span>已选 {selected.length} 台</span>
      </header>
      <div className="fixture-plan-toolbar" aria-label="平面选灯工具">
        <div className="stage-tool-switch">
          <button
            aria-pressed={tool === "select"}
            onClick={() => setTool("select")}
          >
            选灯
          </button>
          <button aria-pressed={tool === "pan"} onClick={() => setTool("pan")}>
            平移
          </button>
        </div>
        <input
          aria-label="搜索平面灯具"
          placeholder="搜索灯具、模式或地址"
          value={query}
          onChange={(e) => onQuery(e.target.value)}
        />
        <button
          aria-pressed={onlySelected}
          onClick={() => onFilter(!onlySelected)}
        >
          仅已选
        </button>
        <button
          disabled={busy || !ids.length}
          onClick={() => void onSelect(ids)}
        >
          全选结果
        </button>
        <button
          disabled={busy || !selected.length}
          onClick={() => void onSelect([])}
        >
          清空选择
        </button>
        <button
          disabled={
            !stage.placements.some((p) => selected.includes(p.fixtureId))
          }
          onClick={() => fit(true)}
        >
          聚焦所选
        </button>
        <button onClick={() => fit()}>全部灯位</button>
        <label>
          <input
            type="checkbox"
            checked={labels}
            onChange={(e) => setLabels(e.target.checked)}
          />
          名称
        </label>
        {!!unplaced.length && (
          <button
            aria-expanded={unplacedOpen}
            onClick={() => setUnplacedOpen((v) => !v)}
          >
            未布置 · {unplaced.length}
          </button>
        )}
      </div>
      <div className="fixture-plan-view">
        <svg
          ref={svg}
          className="stage-canvas"
          role="application"
          aria-label="场景灯位选择画布"
          tabIndex={0}
          viewBox={`${camera.x - camera.width / 2} ${-camera.y - height / 2} ${camera.width} ${height}`}
          {...handlers}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              e.preventDefault();
              e.stopPropagation();
              cancel();
              setOverlap(null);
            }
            if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "a") {
              e.preventDefault();
              e.stopPropagation();
              if (!busy) void onSelect(ids);
            }
            if (e.key.toLowerCase() === "f") {
              e.preventDefault();
              fit(!e.shiftKey);
            }
            if (e.key === "+" || e.key === "=" || e.key === "-") {
              e.preventDefault();
              setCamera((c) => ({
                ...c,
                width: Math.max(
                  1,
                  Math.min(200000, c.width * (e.key === "-" ? 1.2 : 1 / 1.2)),
                ),
              }));
            }
          }}
          onWheel={(e) => {
            if (gesture) return;
            const r = svg.current!.getBoundingClientRect();
            const x = (e.clientX - r.left) / r.width - 0.5,
              y = 0.5 - (e.clientY - r.top) / r.height;
            setCamera((c) => {
              const width = Math.max(
                1,
                Math.min(
                  200000,
                  c.width *
                    Math.exp(Math.max(-100, Math.min(100, e.deltaY)) * 0.003),
                ),
              );
              return {
                width,
                x: c.x + x * (c.width - width),
                y: c.y + (y * (c.width - width)) / ratio,
              };
            });
          }}
        >
          <StagePlanObjects
            project={project}
            stage={{
              ...stage,
              placements: [...stage.placements].sort(
                (a, b) =>
                  Number(selected.includes(a.fixtureId)) -
                  Number(selected.includes(b.fixtureId)),
              ),
            }}
            drawn={(o) => o}
            isSelected={(kind, id) =>
              kind === "placement" && selected.includes(id)
            }
            selectedIds={selected}
            unit={unit}
            labels={labels}
          />
          {gesture?.mode === "select" && gesture.moved && (
            <rect
              className="stage-marquee"
              x={Math.min(gesture.from[0], gesture.to[0])}
              y={-Math.max(gesture.from[1], gesture.to[1])}
              width={Math.abs(gesture.from[0] - gesture.to[0])}
              height={Math.abs(gesture.from[1] - gesture.to[1])}
              strokeWidth={unit * 0.12}
              pointerEvents="none"
            />
          )}
        </svg>
        {!stage.placements.length && (
          <div className="fixture-plan-empty">
            {ids.length
              ? "匹配的灯具尚未布置，可从下方列表选择"
              : "没有匹配的灯具"}
          </div>
        )}
        {!!overlapIds.length && (
          <div
            className="fixture-plan-overlap"
            onKeyDown={(e) => {
              if (e.key === "Escape") {
                e.stopPropagation();
                setOverlap(null);
                svg.current?.focus();
              }
            }}
          >
            <FixturePickList
              label="选择重叠灯位"
              project={project}
              ids={overlapIds}
              selected={selected}
              busy={busy}
              onSelect={(id, additive) =>
                pick(id, additive || !!overlap?.additive)
              }
              onClose={() => {
                setOverlap(null);
                svg.current?.focus();
              }}
            />
          </div>
        )}
      </div>
      {!!unplaced.length && (unplacedOpen || !stage.placements.length) && (
        <FixturePickList
          label="未布置灯具"
          project={project}
          ids={unplaced}
          selected={selected}
          busy={busy}
          onSelect={pick}
        />
      )}
      <footer>
        <span>点击选灯 · ⇧ 增减 · 拖框选择 · 滚动缩放</span>
        <span>安装位置已锁定 · {ids.length} 台匹配</span>
      </footer>
    </section>
  );
}
