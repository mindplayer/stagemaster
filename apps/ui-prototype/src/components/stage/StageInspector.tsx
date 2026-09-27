import { RigFields } from "./RigFields";
import { OutlineDimensions } from "./OutlineDimensions";
import type { RefObject } from "react";
import { CopyIcon, TrashIcon, PlusIcon } from "@phosphor-icons/react";
import type { ProjectView } from "../../application-host";
import type { StageObject } from "../../stage-types";
import { decimal, objectOutline } from "../../stage-tools";
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
              disabled={busy}
              onClick={onDelete}
            >
              <TrashIcon />
            </button>
          </div>
        </header>
        <fieldset disabled={busy}>
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
              </>
            )}
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
                    !project.stage.constructions.some(
                      (c) => c.shape.kind === "rig",
                    )
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
          {outline && (
            <>
              <details className="stage-outline">
                <summary>高级轮廓 · {outline.length} 个顶点</summary>
                <h3>
                  顶点坐标 <span>米</span>
                </h3>
                <div className="stage-point-head">
                  <span>顶点</span>
                  <span>X</span>
                  <span>Y</span>
                  <span />
                </div>
                {outline.map((point, index) => (
                  <div className="stage-point" key={index}>
                    <span>{index + 1}</span>
                    {([0, 1] as const).map((axis) => (
                      <input
                        key={axis}
                        aria-label={`顶点 ${index + 1} ${axis === 0 ? "X" : "Y"}`}
                        type="number"
                        required
                        step="any"
                        min={-100000}
                        max={100000}
                        value={point[axis]}
                        onChange={(e) =>
                          update((c) => {
                            const p = objectOutline(c);
                            if (p) p[index]![axis] = e.target.value;
                          })
                        }
                      />
                    ))}
                    <button
                      type="button"
                      aria-label={`删除顶点 ${index + 1}`}
                      disabled={outline.length <= 3}
                      onClick={() =>
                        update((c) => {
                          objectOutline(c)?.splice(index, 1);
                        })
                      }
                    >
                      <TrashIcon />
                    </button>
                  </div>
                ))}
                <button
                  type="button"
                  disabled={outline.length >= 128}
                  onClick={() =>
                    update((c) => {
                      const p = objectOutline(c);
                      if (p && p.length >= 3) {
                        const a = p.at(-1)!,
                          b = p[0]!;
                        p.push([
                          decimal((Number(a[0]) + Number(b[0])) / 2),
                          decimal((Number(a[1]) + Number(b[1])) / 2),
                        ]);
                      }
                    })
                  }
                >
                  <PlusIcon />
                  添加轮廓顶点
                </button>
              </details>
            </>
          )}
        </fieldset>
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
            disabled={busy || !pending}
            type="submit"
          >
            应用
          </button>
        </footer>
      </form>
    </aside>
  );
}
