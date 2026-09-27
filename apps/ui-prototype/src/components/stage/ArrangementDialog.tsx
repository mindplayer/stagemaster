import { useRef, useState } from "react";
import type { ProjectView } from "../../application-host";
import type { FixturePlacement } from "../../stage-types";
import {
  arrangePlacements,
  PlacementInputError,
  type ArrangementDraft,
  type ArrangementMode,
} from "../../placement-tools";
import { LibraryDialog } from "../workbench/LibraryDialog";
import { bounds, decimal } from "../../stage-tools";
const modes: [ArrangementMode, string][] = [
  ["line", "直线"],
  ["grid", "矩阵"],
  ["circle", "圆弧"],
  ["move", "平移与旋转"],
  ["align", "对齐与分布"],
];
export function ArrangementDialog({
  project,
  initialIds,
  initial,
  busy,
  error,
  onApply,
  onCancel,
}: {
  project: ProjectView;
  initialIds: string[];
  initial: ArrangementDraft;
  busy: boolean;
  error: string;
  onApply(placements: FixturePlacement[]): Promise<boolean>;
  onCancel(): void;
}) {
  const [ids, setIds] = useState(initialIds);
  const [draft, setDraft] = useState(initial);
  const [query, setQuery] = useState("");
  const [scope, setScope] = useState("all");
  const root = useRef<HTMLDivElement>(null);
  const update = (patch: Partial<ArrangementDraft>) =>
    setDraft((d) => ({ ...d, ...patch }));
  const placed = new Set(project.stage.placements.map((p) => p.fixtureId));
  const group = project.groups.find((g) => g.id === scope);
  const candidates = group
    ? group.fixtureIds.flatMap((id) =>
        project.fixtures.filter((f) => f.id === id),
      )
    : project.fixtures;
  const matches = candidates.filter(
    (f) =>
      (scope === "all" ||
        (scope === "unplaced" && !placed.has(f.id)) ||
        (scope === "selected" && ids.includes(f.id)) ||
        group?.fixtureIds.includes(f.id)) &&
      `${f.name} ${f.address ?? ""}`
        .toLocaleLowerCase()
        .includes(query.trim().toLocaleLowerCase()),
  );
  const rows = [...matches].sort(
    (a, b) =>
      (ids.includes(a.id) ? ids.indexOf(a.id) : ids.length) -
      (ids.includes(b.id) ? ids.indexOf(b.id) : ids.length),
  );
  let preview: FixturePlacement[] = [],
    previewError = "";
  try {
    preview = arrangePlacements(ids, project.stage.placements, draft);
  } catch (e) {
    previewError = (e as Error).message;
  }
  const b = preview.length
    ? bounds(
        preview.map((p) => [
          Number(p.positionMeters.x),
          Number(p.positionMeters.y),
        ]),
      )
    : { minX: -1, minY: -1, maxX: 1, maxY: 1 };
  const previewScale = Math.min(
    480 / Math.max(1, b.maxX - b.minX),
    130 / Math.max(1, b.maxY - b.minY),
  );
  const transform = draft.mode === "move" || draft.mode === "align";
  const numeric = (
    field: keyof ArrangementDraft,
    label: string,
    min = -100000,
    max = 100000,
    step = "any",
  ) => (
    <label key={field}>
      {label}
      <input
        type="number"
        step={step}
        required
        min={min}
        max={max}
        aria-label={label}
        data-placement-field={field}
        value={String(draft[field])}
        onChange={(e) => update({ [field]: e.target.value })}
      />
    </label>
  );
  return (
    <LibraryDialog
      title="灯位排列与调整"
      busy={busy}
      error={error}
      submit={`应用到 ${ids.length} 台灯具`}
      onCancel={onCancel}
      onSubmit={async () => {
        try {
          return await onApply(
            arrangePlacements(ids, project.stage.placements, draft),
          );
        } catch (e) {
          if (e instanceof PlacementInputError)
            root.current
              ?.querySelector<HTMLInputElement>(
                `[data-placement-field="${e.field}"]`,
              )
              ?.focus();
          throw e;
        }
      }}
    >
      <div className="placement-editor" ref={root}>
        <section className="placement-picker">
          <header>
            <strong>灯具与顺序</strong>
            <span>已选 {ids.length} 台</span>
          </header>
          <input
            type="search"
            aria-label="搜索布置灯具"
            placeholder="搜索灯具名称或地址"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <select
            aria-label="布灯范围"
            value={scope}
            onChange={(e) => setScope(e.target.value)}
          >
            <option value="all">全部灯具</option>
            <option value="unplaced">未布置灯具</option>
            <option value="selected">当前已选</option>
            {project.groups.map((g) => (
              <option key={g.id} value={g.id}>
                灯组 · {g.name}
              </option>
            ))}
          </select>
          <div className="placement-picker-actions">
            <button
              type="button"
              onClick={() =>
                setIds((old) => [
                  ...old,
                  ...matches.map((f) => f.id).filter((id) => !old.includes(id)),
                ])
              }
            >
              选中筛选结果
            </button>
            <button type="button" onClick={() => setIds([])}>
              清空
            </button>
            <button
              type="button"
              disabled={ids.length < 2}
              onClick={() => setIds((old) => [...old].reverse())}
            >
              反转灯序
            </button>
          </div>
          <div
            className="placement-fixtures"
            role="group"
            aria-label="布灯选择"
          >
            {rows.map((f) => {
              const rank = ids.indexOf(f.id);
              return (
                <div key={f.id} className={rank >= 0 ? "selected" : ""}>
                  <label>
                    <input
                      type="checkbox"
                      aria-label={`布置 ${f.name}`}
                      checked={rank >= 0}
                      onChange={(e) =>
                        setIds((old) =>
                          e.target.checked
                            ? [...old, f.id]
                            : old.filter((id) => id !== f.id),
                        )
                      }
                    />
                    <span>
                      <strong>{f.name}</strong>
                      <small>
                        {placed.has(f.id) ? "已布置" : "未布置"} · 地址{" "}
                        {f.address ?? "未配适"}
                      </small>
                    </span>
                  </label>
                  {rank >= 0 && (
                    <>
                      <b>{rank + 1}</b>
                      <button
                        type="button"
                        aria-label={`提前 ${f.name}`}
                        disabled={rank === 0}
                        onClick={() =>
                          setIds((old) => {
                            const n = [...old];
                            [n[rank - 1], n[rank]] = [n[rank], n[rank - 1]];
                            return n;
                          })
                        }
                      >
                        ↑
                      </button>
                      <button
                        type="button"
                        aria-label={`延后 ${f.name}`}
                        disabled={rank === ids.length - 1}
                        onClick={() =>
                          setIds((old) => {
                            const n = [...old];
                            [n[rank + 1], n[rank]] = [n[rank], n[rank + 1]];
                            return n;
                          })
                        }
                      >
                        ↓
                      </button>
                    </>
                  )}
                </div>
              );
            })}
            {!matches.length && <p className="wb-dim">没有匹配的灯具</p>}
          </div>
          {ids.some((id) => !matches.some((f) => f.id === id)) && (
            <p className="wb-dim">
              另有{" "}
              {ids.filter((id) => !matches.some((f) => f.id === id)).length}{" "}
              台已选灯具被筛选隐藏，仍参与本次布置。
            </p>
          )}
        </section>
        <section className="placement-settings">
          <div className="placement-modes" aria-label="排列方式">
            {modes.map(([mode, label]) => (
              <button
                type="button"
                key={mode}
                aria-pressed={draft.mode === mode}
                onClick={() => update({ mode })}
              >
                {label}
              </button>
            ))}
          </div>
          {!transform && (
            <>
              <label>
                所属空间
                <select
                  aria-label="排列所属空间"
                  value={draft.spaceId ?? ""}
                  onChange={(e) => update({ spaceId: e.target.value || null })}
                >
                  <option value="">未归属空间</option>
                  {project.stage.spaces.map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.name}
                    </option>
                  ))}
                </select>
              </label>
              <div className="placement-fields">
                {numeric("x", "中心 X（米）")}
                {numeric("y", "中心 Y（米）")}
                {numeric("z", "安装高度（米）")}
              </div>
              <div className="placement-fields">
                {draft.mode !== "circle" &&
                  numeric("spacing", "灯间距（米）", 0.001)}
                {draft.mode === "grid" &&
                  numeric("rowSpacing", "排间距（米）", 0.001)}
                {draft.mode === "grid" &&
                  numeric("columns", "每排灯数", 1, 256, "1")}
                {draft.mode === "circle" &&
                  numeric("radius", "半径（米）", 0.001)}
                {draft.mode === "circle" &&
                  numeric("arc", "圆弧范围（度）", 0.01, 360)}
                {numeric(
                  "angle",
                  draft.mode === "circle" ? "起始角度（度）" : "排列角度（度）",
                  -360,
                  360,
                )}
              </div>
            </>
          )}
          {draft.mode === "move" && (
            <>
              <div className="placement-fields">
                {numeric("dx", "X 平移（米）")}
                {numeric("dy", "Y 平移（米）")}
                {numeric("dz", "Z 平移（米）")}
              </div>
              {numeric("turn", "绕排列中心旋转（度）", -360, 360)}
              <p className="wb-dim">
                绕所选灯位的平面包围中心旋转，再平移；底座朝向保持。
              </p>
            </>
          )}
          {draft.mode === "align" && (
            <div className="placement-fields">
              <label>
                对齐轴
                <select
                  aria-label="对齐轴"
                  value={draft.axis}
                  onChange={(e) =>
                    update({ axis: e.target.value as ArrangementDraft["axis"] })
                  }
                >
                  <option value="x">X · 左右</option>
                  <option value="y">Y · 前后</option>
                  <option value="z">Z · 高度</option>
                </select>
              </label>
              <label>
                对齐方式
                <select
                  aria-label="对齐方式"
                  value={draft.alignment}
                  onChange={(e) =>
                    update({
                      alignment: e.target
                        .value as ArrangementDraft["alignment"],
                    })
                  }
                >
                  <option value="min">对齐最小坐标</option>
                  <option value="center">对齐中心</option>
                  <option value="max">对齐最大坐标</option>
                  <option value="distribute">等距分布</option>
                </select>
              </label>
              <p className="wb-dim">等距分布保留空间次序和两端位置。</p>
            </div>
          )}
          <details>
            <summary>底座安装朝向</summary>
            <label className="stage-check">
              <input
                type="checkbox"
                checked={draft.setRotation}
                onChange={(e) => update({ setRotation: e.target.checked })}
              />
              统一设置底座朝向
            </label>
            {draft.setRotation && (
              <div className="placement-fields">
                {numeric("rx", "底座 X（度）", -3600, 3600)}
                {numeric("ry", "底座 Y（度）", -3600, 3600)}
                {numeric("rz", "底座 Z（度）", -3600, 3600)}
              </div>
            )}
          </details>
          <div className="placement-preview">
            <header>
              <strong>俯视预览</strong>
              <span>
                {preview.length
                  ? `${decimal(b.maxX - b.minX)} × ${decimal(b.maxY - b.minY)} 米`
                  : ""}
              </span>
            </header>
            <svg role="img" aria-label="灯位排列草稿预览" viewBox="0 0 520 180">
              {preview.map((p, i) => (
                <g
                  key={p.fixtureId}
                  transform={`translate(${260 + (Number(p.positionMeters.x) - (b.minX + b.maxX) / 2) * previewScale},${90 - (Number(p.positionMeters.y) - (b.minY + b.maxY) / 2) * previewScale})`}
                >
                  <circle r={9} />
                  <text
                    fontSize={10}
                    textAnchor="middle"
                    dominantBaseline="central"
                  >
                    {i + 1}
                  </text>
                  <title>
                    {project.fixtures.find((f) => f.id === p.fixtureId)?.name} ·{" "}
                    {p.positionMeters.x}, {p.positionMeters.y},{" "}
                    {p.positionMeters.z} 米
                  </title>
                </g>
              ))}
            </svg>
            {previewError && (
              <p role="status" className="wb-library-error">
                {previewError}
              </p>
            )}
          </div>
        </section>
      </div>
    </LibraryDialog>
  );
}
