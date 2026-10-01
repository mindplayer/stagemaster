import type { ProjectView } from "../../application-host";
import { AimTargetPlane } from "./AimTargetPlane";
type Fields = { x: string; y: string; z: string; branch: string };
export function AimTargetFields({
  project,
  fixtureIds,
  values,
  busy,
  onChange,
}: {
  project: ProjectView;
  fixtureIds: string[];
  values: Fields;
  busy: boolean;
  onChange(patch: Partial<Fields>): void;
}) {
  return (
    <>
      <div className="position-fields">
        {(
          [
            ["x", "目标 X（米）"],
            ["y", "目标 Y（米）"],
            ["z", "目标高度（米）"],
          ] as const
        ).map(([key, label]) => (
          <label key={key}>
            {key === "z" ? "高度（米）" : `${key.toUpperCase()}（米）`}
            <input
              name={key}
              aria-label={label}
              inputMode="decimal"
              value={values[key]}
              onChange={(event) => onChange({ [key]: event.target.value })}
            />
          </label>
        ))}
      </div>
      <AimTargetPlane
        {...{ project, fixtureIds, onChange }}
        x={values.x}
        y={values.y}
        disabled={busy}
      />
      <details className="aim-target-settings">
        <summary>
          指向设置 ·{" "}
          {values.branch === "front"
            ? "正向解"
            : values.branch === "back"
              ? "翻转解"
              : "最短轴角变化"}
        </summary>
        <p className="wb-dim">
          平面选点只修改
          X／Y，高度独立设置。应用后记录轴角，后续移动灯位不会自动追踪此点。
        </p>
        <label>
          解分支
          <select
            aria-label="指向分支"
            value={values.branch}
            onChange={(event) => onChange({ branch: event.target.value })}
          >
            <option value="">最短轴角变化</option>
            <option value="front">正向解</option>
            <option value="back">翻转解</option>
          </select>
        </label>
        <p className="wb-dim">
          精度受通道位数限制；场景之间按轴角渐变，不保证空间直线路径。
        </p>
      </details>
    </>
  );
}
