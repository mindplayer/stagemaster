import type { ProjectView } from "../../application-host";
import type { ArrangementDraft, ArrangementMode } from "../../placement-tools";
const modes: [ArrangementMode, string][] = [
  ["line", "直线"],
  ["grid", "矩阵"],
  ["circle", "圆弧"],
  ["move", "平移与旋转"],
  ["align", "对齐与分布"],
];
export function ArrangementFields({
  project,
  draft,
  update,
}: {
  project: ProjectView;
  draft: ArrangementDraft;
  update(patch: Partial<ArrangementDraft>): void;
}) {
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
            {draft.mode === "circle" && numeric("radius", "半径（米）", 0.001)}
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
                  alignment: e.target.value as ArrangementDraft["alignment"],
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
    </section>
  );
}
