import { useImperativeHandle, useRef, useState } from "react";
import type { Ref } from "react";
import type {
  EditOperation,
  FixtureView,
  ProjectView,
  SceneView,
} from "../../application-host";
import {
  referenceCapture,
  type ReferenceDraft,
} from "../../position-reference";
import { FixtureFieldError } from "../../fixture-field-error";
import type { PositionHandle } from "./PositionAxisPanel";
import { AimTargetPlane } from "./AimTargetPlane";
import { PositionReferenceList } from "./PositionReferenceList";
import "./position-reference.css";
export function PositionReferencePanel({
  ref,
  project,
  fixture,
  scene,
  busy,
  onPending,
  onApply,
}: {
  ref: Ref<PositionHandle>;
  project: ProjectView;
  fixture: FixtureView;
  scene: SceneView;
  busy: boolean;
  onPending(value: boolean): void;
  onApply(): void;
}) {
  const [draft, setDraft] = useState<ReferenceDraft>({
    name: "",
    x: "0",
    y: "0",
    z: "0",
  });
  const current = useRef(draft),
    accepted = useRef(draft),
    pending = useRef(false);
  const action = useRef<EditOperation | null>(null);
  const [dirty, setDirty] = useState(false),
    form = useRef<HTMLFormElement>(null);
  function change(next: ReferenceDraft, changed = true) {
    action.current = null;
    current.current = next;
    setDraft(next);
    pending.current = changed;
    setDirty(changed);
    onPending(changed);
  }
  function reset(next: ReferenceDraft) {
    action.current = null;
    change(next, false);
    form.current
      ?.querySelectorAll<HTMLInputElement>("input")
      .forEach((el) => el.setCustomValidity(""));
  }
  function collect(): EditOperation[] {
    if (!pending.current) return [];
    if (action.current) return [action.current];
    try {
      return [referenceCapture(project, fixture, scene.id, current.current)];
    } catch (e) {
      if (e instanceof FixtureFieldError) {
        const el = form.current?.elements.namedItem(
          e.field,
        ) as HTMLInputElement | null;
        el?.focus();
        el?.setCustomValidity(e.message);
        el?.reportValidity();
      }
      throw e;
    }
  }
  useImperativeHandle(ref, () => ({
    collect,
    accept() {
      accepted.current = { ...current.current, name: "" };
      reset(accepted.current);
    },
  }));
  function remove(pointId?: string) {
    change(current.current);
    action.current = {
      op: "position",
      command: pointId
        ? { op: "removeReference", fixtureId: fixture.id, pointId }
        : { op: "clearReferences", fixtureId: fixture.id },
    };
    onApply();
  }
  const cannotCapture =
    !fixture.positioning ||
    fixture.positionReference?.compatible === false ||
    !project.stage.placements.some((p) => p.fixtureId === fixture.id) ||
    (fixture.positionReference?.points.length ?? 0) >= 16;
  return (
    <form
      ref={form}
      className="wb-parameters position-reference-panel"
      noValidate
      onSubmit={(e) => {
        e.preventDefault();
        change(current.current);
        onApply();
      }}
      onInputCapture={(e) => {
        if (e.target instanceof HTMLInputElement)
          e.target.setCustomValidity("");
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape" && !busy) {
          e.preventDefault();
          reset(accepted.current);
        }
      }}
    >
      <h2>参考点检查</h2>
      <p className="wb-dim">
        保存世界目标与当前场景轴设定，用于检查安装和零偏模型。记录不移动灯具，也不代表实灯已到位。
      </p>
      <fieldset disabled={busy || cannotCapture}>
        <label>
          参考点名称
          <input
            name="name"
            aria-label="参考点名称"
            value={draft.name}
            onChange={(e) =>
              change({ ...current.current, name: e.target.value })
            }
          />
        </label>
        <div className="position-fields">
          {(["x", "y", "z"] as const).map((key) => (
            <label key={key}>
              {key === "z" ? "高度" : key.toUpperCase()}（米）
              <input
                name={key}
                aria-label={`参考点 ${key.toUpperCase()}（米）`}
                inputMode="decimal"
                value={draft[key]}
                onChange={(e) =>
                  change({ ...current.current, [key]: e.target.value })
                }
              />
            </label>
          ))}
        </div>
        <AimTargetPlane
          project={project}
          fixtureIds={[fixture.id]}
          x={draft.x}
          y={draft.y}
          disabled={busy || cannotCapture}
          targetLabel="参考点"
          planeLabel="参考点平面"
          onChange={(patch) => change({ ...current.current, ...patch })}
        />
      </fieldset>
      {cannotCapture && (
        <p className="wb-dim">
          记录需要已布置的两轴灯、相同档案版本且少于 16
          个点；旧记录仍可查看和删除。
        </p>
      )}
      <div className="profile-actions">
        <button
          type="button"
          disabled={busy || !dirty}
          onClick={() => reset(accepted.current)}
        >
          取消录入
        </button>
        <button className="wb-primary" disabled={busy || cannotCapture}>
          记录参考点
        </button>
      </div>
      <PositionReferenceList
        record={fixture.positionReference}
        disabled={busy || dirty}
        onRemove={(id) => remove(id)}
        onClear={() => remove()}
      />
      {dirty && <p className="wb-dim">完成或取消当前录入后可删除旧参考点。</p>}
    </form>
  );
}
