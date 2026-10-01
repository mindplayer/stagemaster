import { StagePlacementFields } from "./StagePlacementFields";
import { StageLockControls } from "./StageLockControls";
import { isStageLocked, stageTarget } from "../../stage-locks";
import { StageOutlineFields } from "./StageOutlineFields";
import { RigFields } from "./RigFields";
import { OutlineDimensions } from "./OutlineDimensions";
import type { RefObject } from "react";
import { CopyIcon, TrashIcon, PlusIcon } from "@phosphor-icons/react";
import type { ProjectView } from "../../application-host";
import type { StageObject } from "../../stage-types";
import { objectOutline } from "../../stage-tools";
export function StageInspector({
  object,
  project,
  pending,
  busy,
  form,
  error,
  onChange,
  onApply,
  onCancel,
  onDelete,
  onDuplicate,
  onEnclose,
  onHang,
  onSelectMounted,
  onDetach,
  onLock,
}: {
  object: StageObject | null;
  project: ProjectView;
  pending: boolean;
  busy: boolean;
  form: RefObject<HTMLFormElement | null>;
  error: string;
  onChange(object: StageObject): void;
  onApply(): void;
  onCancel(): void;
  onDelete(): void;
  onDuplicate(): void;
  onEnclose(): void;
  onHang(): void;
  onSelectMounted(id: string): void;
  onDetach(ids: string[]): void;
  onLock(locked: boolean): void;
}) {
  if (!object)
    return (
      <aside className="stage-inspector">
        <div className="wb-empty">
          <h2>选择空间、构件或灯具</h2>
        </div>
      </aside>
    );
  const update = (fn: (copy: StageObject) => void) => {
    const copy = structuredClone(object);
    fn(copy);
    onChange(copy);
  };
  const target = stageTarget(object);
  const locked = isStageLocked(project.stage, target);
  const outline = objectOutline(object);
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
    <aside className="stage-inspector">
      <form
        noValidate
        ref={form}
        onSubmit={(e) => {
          e.preventDefault();
          onApply();
        }}
      >
        <header>
          <h2>
            {object.kind === "space"
              ? "空间属性"
              : object.kind === "construction"
                ? "构件属性"
                : "安装位置"}
          </h2>
          <div className="stage-actions">
            {object.kind !== "placement" &&
              !(
                object.kind === "construction" &&
                object.value.shape.kind === "enclosure"
              ) && (
                <button
                  type="button"
                  title="复制选中对象"
                  aria-label="复制选中对象"
                  disabled={busy}
                  onClick={onDuplicate}
                >
                  <CopyIcon />
                </button>
              )}
            <button
              type="button"
              title={object.kind === "placement" ? "移除灯位" : "删除选中对象"}
              aria-label={
                object.kind === "placement" ? "移除灯位" : "删除选中对象"
              }
              disabled={busy || locked}
              onClick={onDelete}
            >
              <TrashIcon />
            </button>
          </div>
        </header>
        <StageLockControls
          stage={project.stage}
          targets={[target]}
          busy={busy}
          onLock={onLock}
        />
        <fieldset disabled={busy || locked}>
          {object.kind !== "placement" ? (
            <label>
              名称
              <input
                aria-label="对象名称"
                required
                maxLength={256}
                value={object.value.name}
                onChange={(e) =>
                  update((c) => {
                    if (c.kind !== "placement") c.value.name = e.target.value;
                  })
                }
              />
            </label>
          ) : (
            <strong>
              {
                project.fixtures.find((f) => f.id === object.value.fixtureId)
                  ?.name
              }
            </strong>
          )}
          {outline && <OutlineDimensions object={object} onChange={onChange} />}
          {object.kind === "space" && (
            <>
              {numeric(
                "地面标高（米）",
                object.value.floorElevationMeters,
                (v) =>
                  update((c) => {
                    if (c.kind === "space") c.value.floorElevationMeters = v;
                  }),
                -10000,
                10000,
              )}
              <label className="stage-check">
                <input
                  type="checkbox"
                  checked={object.value.clearHeightMeters !== null}
                  onChange={(e) =>
                    update((c) => {
                      if (c.kind === "space")
                        c.value.clearHeightMeters = e.target.checked
                          ? "5"
                          : null;
                    })
                  }
                />
                有顶空间
              </label>
              {object.value.clearHeightMeters !== null &&
                numeric(
                  "净高（米）",
                  object.value.clearHeightMeters,
                  (v) =>
                    update((c) => {
                      if (c.kind === "space") c.value.clearHeightMeters = v;
                    }),
                  0.1,
                  1000,
                )}
            </>
          )}
          {object.kind === "construction" &&
            object.value.shape.kind === "platform" && (
              <>
                {member(object.value.shape.spaceId, (v) =>
                  update((c) => {
                    if (
                      c.kind === "construction" &&
                      c.value.shape.kind === "platform"
                    )
                      c.value.shape.spaceId = v;
                  }),
                )}
                {numeric(
                  "底部标高（米）",
                  object.value.shape.baseElevationMeters,
                  (v) =>
                    update((c) => {
                      if (
                        c.kind === "construction" &&
                        c.value.shape.kind === "platform"
                      )
                        c.value.shape.baseElevationMeters = v;
                    }),
                  -10000,
                  10000,
                )}
                {numeric(
                  "台高（米）",
                  object.value.shape.heightMeters,
                  (v) =>
                    update((c) => {
                      if (
                        c.kind === "construction" &&
                        c.value.shape.kind === "platform"
                      )
                        c.value.shape.heightMeters = v;
                    }),
                  0.001,
                  1000,
                )}
              </>
            )}
          {object.kind === "construction" &&
            object.value.shape.kind === "enclosure" && (
              <>
                <p className="wb-dim">
                  {
                    project.stage.spaces.find(
                      (s) =>
                        object.value.shape.kind === "enclosure" &&
                        s.id === object.value.shape.spaceId,
                    )?.name
                  }
                </p>
                {numeric(
                  "墙厚（米）",
                  object.value.shape.wallThicknessMeters,
                  (v) =>
                    update((c) => {
                      if (
                        c.kind === "construction" &&
                        c.value.shape.kind === "enclosure"
                      )
                        c.value.shape.wallThicknessMeters = v;
                    }),
                  0.001,
                  10,
                )}
                {numeric(
                  "地板厚（米）",
                  object.value.shape.floorThicknessMeters,
                  (v) =>
                    update((c) => {
                      if (
                        c.kind === "construction" &&
                        c.value.shape.kind === "enclosure"
                      )
                        c.value.shape.floorThicknessMeters = v;
                    }),
                  0.001,
                  10,
                )}
                <label className="stage-check">
                  <input
                    type="checkbox"
                    checked={object.value.shape.ceilingThicknessMeters !== null}
                    onChange={(e) =>
                      update((c) => {
                        if (
                          c.kind === "construction" &&
                          c.value.shape.kind === "enclosure"
                        )
                          c.value.shape.ceilingThicknessMeters = e.target
                            .checked
                            ? "0.1"
                            : null;
                      })
                    }
                  />
                  生成顶板
                </label>
                {object.value.shape.ceilingThicknessMeters !== null &&
                  numeric(
                    "顶板厚（米）",
                    object.value.shape.ceilingThicknessMeters,
                    (v) =>
                      update((c) => {
                        if (
                          c.kind === "construction" &&
                          c.value.shape.kind === "enclosure"
                        )
                          c.value.shape.ceilingThicknessMeters = v;
                      }),
                    0.001,
                    10,
                  )}
              </>
            )}
          {object.kind === "construction" &&
            object.value.shape.kind === "rig" && (
              <>
                <RigFields
                  value={object.value.shape}
                  spaces={project.stage.spaces}
                  onChange={(shape) =>
                    onChange({ ...object, value: { ...object.value, shape } })
                  }
                />
              </>
            )}
          <StagePlacementFields
            object={object}
            project={project}
            onChange={onChange}
            onHang={onHang}
            onDetach={onDetach}
          />
          <StageOutlineFields object={object} onChange={onChange} />
        </fieldset>
        {object.kind === "space" && (
          <fieldset disabled={busy}>
            {" "}
            {!project.stage.constructions.some(
              (c) =>
                c.shape.kind === "enclosure" &&
                c.shape.spaceId === object.value.id,
            ) && (
              <button
                type="button"
                disabled={object.value.clearHeightMeters === null}
                onClick={onEnclose}
              >
                <PlusIcon />
                添加墙体与地板
              </button>
            )}
          </fieldset>
        )}
        {object.kind === "construction" &&
          object.value.shape.kind === "rig" && (
            <fieldset disabled={busy}>
              {" "}
              <div className="rig-member-actions">
                <button type="button" onClick={onHang}>
                  批量挂灯
                </button>
                <button
                  type="button"
                  disabled={
                    !project.stage.attachments.some(
                      (a) => a.constructionId === object.value.id,
                    )
                  }
                  onClick={() => onSelectMounted(object.value.id)}
                >
                  选中全部挂灯
                </button>
              </div>
              <p className="wb-dim">
                已挂{" "}
                {
                  project.stage.attachments.filter(
                    (a) => a.constructionId === object.value.id,
                  ).length
                }{" "}
                台 · 随支撑体移动
              </p>
            </fieldset>
          )}
        {error && (
          <p className="stage-error" role="alert">
            {error}
          </p>
        )}
        <footer>
          <button type="button" disabled={busy || !pending} onClick={onCancel}>
            取消修改
          </button>
          <button
            className="wb-primary"
            disabled={busy || locked || !pending}
            type="submit"
          >
            应用
          </button>
        </footer>
      </form>
    </aside>
  );
}
