import { useImperativeHandle, useRef, useState } from "react";
import type { Ref } from "react";
import type {
  EditOperation,
  FixtureView,
  ProjectView,
  SceneView,
} from "../../application-host";
import type { PositionEdit } from "../../position-types";
import { PositionReadout } from "./PositionReadout";
import { AimTargetFields } from "./AimTargetFields";
import { FixtureFieldError } from "../../fixture-tools";
import { positionDecimal } from "../../position-tools";
export interface PositionHandle {
  collect(): EditOperation[];
  accept(): void;
}
type Mode = "axes" | "aim" | "calibrate";
export function PositionPanel({
  ref,
  project,
  fixtures,
  scene,
  busy,
  onPending,
  onApply,
  beforeChange,
}: {
  ref: Ref<PositionHandle>;
  project: ProjectView;
  fixtures: FixtureView[];
  scene: SceneView;
  busy: boolean;
  onPending(v: boolean): void;
  onApply(): void;
  beforeChange(): Promise<boolean>;
}) {
  const [mode, setMode] = useState<Mode>("axes");
  const [values, setValues] = useState({
    pan: "",
    tilt: "",
    x: "0",
    y: "0",
    z: "0",
    branch: "",
    panZero: "",
    tiltZero: "",
  });
  const accepted = useRef(values);
  const current = useRef(values),
    pending = useRef(false),
    [dirty, setDirty] = useState(false),
    form = useRef<HTMLFormElement>(null);
  const heads = fixtures.filter((f) => f.positioning);
  const action = useRef<"home" | "release" | "remove" | null>(null);
  function change(next: typeof values, changed = true) {
    action.current = null;
    current.current = next;
    setValues(next);
    pending.current = changed;
    setDirty(changed);
    onPending(changed);
  }
  function reset(next: typeof values) {
    change({ ...next, pan: "", tilt: "", panZero: "", tiltZero: "" }, false);
    form.current
      ?.querySelectorAll<HTMLInputElement>("input")
      .forEach((el) => el.setCustomValidity(""));
  }
  function accept() {
    accepted.current = current.current;
    reset(current.current);
  }
  function cancel() {
    reset(accepted.current);
  }
  function collect(): EditOperation[] {
    if (!pending.current) return [];
    try {
      if (!heads.length || heads.length !== fixtures.length)
        throw new Error("请选择全部具有两轴模型的灯具");
      const v = current.current,
        fixtureIds = heads.map((f) => f.id),
        sceneId = scene.id;
      const decimal = (key: keyof typeof v, label: string, limit = 3600) =>
        positionDecimal(v[key], key, label, limit);
      if (action.current === "home")
        return [
          { op: "position", command: { op: "home", sceneId, fixtureIds } },
        ];
      if (action.current === "release" || action.current === "remove")
        return fixtureIds.flatMap((fixtureId) =>
          ["pan", "tilt"].map((attribute) => ({
            op: "setSceneValue" as const,
            sceneId,
            fixtureId,
            attribute,
            mode: action.current as "release" | "remove",
            value: 0,
          })),
        );
      let command: PositionEdit;
      if (mode === "axes") {
        for (const key of ["pan", "tilt"] as const) {
          if (v[key].trim()) {
            const n = Number(decimal(key, key === "pan" ? "水平角" : "垂直角"));
            for (const f of heads) {
              const a = f.positioning![key];
              if (n < Number(a.minDegrees) || n > Number(a.maxDegrees))
                throw new FixtureFieldError(
                  key,
                  `${f.name}：${key === "pan" ? "水平" : "垂直"}角须在 ${a.minDegrees}–${a.maxDegrees}° 之间`,
                );
            }
          }
        }
        if (!v.pan.trim() && !v.tilt.trim())
          throw new FixtureFieldError("pan", "至少填写一个轴角度");
        command = {
          op: "axes",
          fixtureIds,
          sceneId,
          panDegrees: v.pan.trim() ? decimal("pan", "水平角") : null,
          tiltDegrees: v.tilt.trim() ? decimal("tilt", "垂直角") : null,
        };
      } else if (mode === "aim")
        command = {
          op: "aim",
          fixtureIds,
          sceneId,
          targetMeters: {
            x: decimal("x", "目标 X", 100000),
            y: decimal("y", "目标 Y", 100000),
            z: decimal("z", "目标高度", 100000),
          },
          branch: (v.branch as "front" | "back") || null,
        };
      else {
        if (heads.length !== 1) throw new Error("请单独选择一台灯具设置零偏");
        const old = heads[0].zeroCorrection;
        command = {
          op: "calibrate",
          fixtureId: heads[0].id,
          correction: {
            panDegrees: v.panZero.trim()
              ? decimal("panZero", "水平零偏", 360)
              : (old?.panDegrees ?? "0"),
            tiltDegrees: v.tiltZero.trim()
              ? decimal("tiltZero", "垂直零偏", 360)
              : (old?.tiltDegrees ?? "0"),
          },
        };
      }
      return [{ op: "position", command }];
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
  useImperativeHandle(ref, () => ({ collect, accept }));
  if (!heads.length) return null;
  const input = (key: keyof typeof values, label: string, placeholder = "") => (
    <label>
      {label}
      <input
        name={key}
        aria-label={label}
        inputMode="decimal"
        value={values[key]}
        placeholder={placeholder}
        onChange={(e) => change({ ...current.current, [key]: e.target.value })}
      />
    </label>
  );
  return (
    <form
      ref={form}
      noValidate
      className={`wb-parameters position-panel position-${mode}`}
      onSubmit={(e) => {
        e.preventDefault();
        if (mode === "aim") change({ ...current.current });
        onApply();
      }}
      onInputCapture={(e) => {
        if (e.target instanceof HTMLInputElement)
          e.target.setCustomValidity("");
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape" && !busy) {
          e.preventDefault();
          cancel();
        }
      }}
    >
      <div className="wb-section-title">
        <h2>灯头位置</h2>
        <span>{heads.length} 台摇头灯</span>
      </div>
      {heads.length !== fixtures.length ? (
        <p>当前混选了固定灯，请仅选择摇头灯以编辑位置。</p>
      ) : (
        <>
          <div className="position-tabs">
            {(
              [
                ["axes", "轴角"],
                ["aim", "共同指向"],
                ["calibrate", "单灯零偏"],
              ] as const
            ).map(([key, label]) => (
              <button
                type="button"
                key={key}
                aria-pressed={mode === key}
                disabled={busy || (key === "calibrate" && heads.length !== 1)}
                onClick={async () => {
                  if (await beforeChange()) setMode(key);
                }}
              >
                {label}
              </button>
            ))}
          </div>
          <fieldset disabled={busy}>
            {mode === "axes" && (
              <>
                <div className="position-fields">
                  {input("pan", "水平角（°）", "留空保持")}
                  {input("tilt", "垂直角（°）", "留空保持")}
                </div>
                <PositionReadout {...{ project, heads, scene }} />
              </>
            )}
            {mode === "aim" && (
              <AimTargetFields
                project={project}
                fixtureIds={heads.map((f) => f.id)}
                values={values}
                busy={busy}
                onChange={(patch) => {
                  change({ ...current.current, ...patch });
                  for (const key of Object.keys(patch))
                    (
                      form.current?.elements.namedItem(
                        key,
                      ) as HTMLInputElement | null
                    )?.setCustomValidity("");
                }}
              />
            )}
            {mode === "calibrate" && (
              <>
                <p className="wb-dim">
                  手工修正这台灯的物理零偏，作用于所有场景的指向计算与预演。不会发送设备复位。
                </p>
                <div className="position-fields">
                  {input(
                    "panZero",
                    "水平零偏（°）",
                    heads[0].zeroCorrection?.panDegrees ?? "0",
                  )}
                  {input(
                    "tiltZero",
                    "垂直零偏（°）",
                    heads[0].zeroCorrection?.tiltDegrees ?? "0",
                  )}
                </div>
              </>
            )}
          </fieldset>
          <div className="profile-actions">
            {mode === "axes" &&
              (
                [
                  [
                    "home",
                    "记录默认位置",
                    "将档案默认值写入当前场景，不发送设备复位",
                  ],
                  [
                    "release",
                    "释放位置",
                    "列表执行到本场景时结束之前的位置跟踪",
                  ],
                  [
                    "remove",
                    "清除位置",
                    "删除本场景的两轴记录；列表播放可能沿用之前场景的位置",
                  ],
                ] as const
              ).map(([kind, label, hint]) => (
                <button
                  key={kind}
                  type="button"
                  title={hint}
                  disabled={busy}
                  onClick={async () => {
                    if (await beforeChange()) {
                      change({ ...current.current });
                      action.current = kind;
                      onApply();
                    }
                  }}
                >
                  {label}
                </button>
              ))}
            <button type="button" disabled={busy || !dirty} onClick={cancel}>
              取消修改
            </button>
            <button
              className="wb-primary"
              disabled={busy || (!dirty && mode !== "aim")}
            >
              {mode === "aim"
                ? "对准并记录轴角"
                : mode === "calibrate"
                  ? "应用零偏"
                  : "记录轴角"}
            </button>
          </div>
        </>
      )}
    </form>
  );
}
