import type { ProjectView } from "../../application-host";
import type { StageObject } from "../../stage-types";
export function StagePlacementFields({
  object,
  project,
  onChange,
  onHang,
  onDetach,
}: {
  object: StageObject;
  project: ProjectView;
  onChange(value: StageObject): void;
  onHang(): void;
  onDetach(ids: string[]): void;
}) {
  const update = (fn: (copy: StageObject) => void) => {
    const copy = structuredClone(object);
    fn(copy);
    onChange(copy);
  };
  const numeric = (
    label: string,
    value: string,
    change: (value: string) => void,
    min = -100000,
    max = 100000,
  ) => (
    <label>
      {label}
      <input
        aria-label={label}
        type="number"
        step="any"
        min={min}
        max={max}
        required
        value={value}
        onChange={(e) => change(e.target.value)}
      />
    </label>
  );
  const member = (
    value: string | null,
    change: (value: string | null) => void,
  ) => (
    <label>
      所属空间
      <select
        aria-label="所属空间"
        value={value ?? ""}
        onChange={(e) => change(e.target.value || null)}
      >
        <option value="">未归属</option>
        {project.stage.spaces.map((s) => (
          <option key={s.id} value={s.id}>
            {s.name}
          </option>
        ))}
      </select>
    </label>
  );
  return (
    <>
      {object.kind === "placement" && (
        <>
          <div className="rig-member-actions">
            <span>
              {project.stage.constructions.find(
                (c) =>
                  c.id ===
                  project.stage.attachments.find(
                    (a) => a.fixtureId === object.value.fixtureId,
                  )?.constructionId,
              )?.name ?? "未挂接支撑体"}
            </span>
            <button
              type="button"
              disabled={
                !project.stage.constructions.some((c) => c.shape.kind === "rig")
              }
              onClick={onHang}
            >
              挂接／换挂
            </button>
            <button
              type="button"
              disabled={
                !project.stage.attachments.some(
                  (a) => a.fixtureId === object.value.fixtureId,
                )
              }
              onClick={() => onDetach([object.value.fixtureId])}
            >
              解除挂接
            </button>
          </div>
          {member(object.value.spaceId, (v) =>
            update((c) => {
              if (c.kind === "placement") c.value.spaceId = v;
            }),
          )}
          <div className="stage-fields">
            {(["x", "y", "z"] as const).map((axis) => (
              <div key={axis}>
                {numeric(
                  `${axis.toUpperCase()} 位置（米）`,
                  object.value.positionMeters[axis],
                  (v) =>
                    update((c) => {
                      if (c.kind === "placement")
                        c.value.positionMeters[axis] = v;
                    }),
                )}
              </div>
            ))}
          </div>
          <h3>底座安装朝向</h3>
          <div className="stage-fields">
            {(["x", "y", "z"] as const).map((axis) => (
              <div key={axis}>
                {numeric(
                  `${axis.toUpperCase()} 旋转（度）`,
                  object.value.rotationDegreesXYZ[axis],
                  (v) =>
                    update((c) => {
                      if (c.kind === "placement")
                        c.value.rotationDegreesXYZ[axis] = v;
                    }),
                  -3600,
                  3600,
                )}
              </div>
            ))}
          </div>
        </>
      )}
    </>
  );
}
