import type { ProjectView } from "../../application-host";
import type { StageSelection } from "../../stage-types";
import { selectedStage } from "../../stage-tools";
import { StageLockControls } from "./StageLockControls";
import {
  affectedTargets,
  translationProblem,
  zeroTranslation,
} from "./object-translation";
import type { useObjectTranslation } from "./useObjectTranslation";
export function StageObjectGroupInspector({
  project,
  targets,
  translation,
  busy,
  error,
  onApply,
  onCancel,
  onClear,
  onLock,
}: {
  project: ProjectView;
  targets: StageSelection[];
  translation: ReturnType<typeof useObjectTranslation>;
  busy: boolean;
  error: string;
  onApply(): void;
  onCancel(): void;
  onClear(): void;
  onLock(locked: boolean): void;
}) {
  const problem = translationProblem(project.stage, targets);
  const delta = translation.draft?.delta ?? zeroTranslation();
  const followers =
    affectedTargets(project.stage, targets).length - targets.length;
  return (
    <aside className="stage-inspector stage-multi-inspector">
      <header>
        <h2>已选 {targets.length} 个对象</h2>
      </header>
      <StageLockControls
        stage={project.stage}
        targets={targets}
        busy={busy}
        onLock={onLock}
      />
      <form
        ref={translation.form}
        noValidate
        onSubmit={(e) => {
          e.preventDefault();
          onApply();
        }}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            e.preventDefault();
            e.stopPropagation();
            onCancel();
          }
        }}
      >
        <fieldset disabled={busy || !!problem}>
          <legend>整组相对移动</legend>
          {(["x", "y", "z"] as const).map((axis, i) => (
            <label key={axis}>
              {["左右 X", "前后 Y", "升降 Z"][i]}（米）
              <input
                aria-label={`整组${["左右", "前后", "升降"][i]}位移`}
                inputMode="decimal"
                data-translation-axis={axis}
                data-pattern-message="需要最多六位小数的十进制位移"
                required
                pattern="-?[0-9]+(\.[0-9]{1,6})?"
                value={delta[axis]}
                onChange={(e) => translation.update(axis, e.target.value)}
              />
            </label>
          ))}
          <button
            className="wb-primary"
            type="submit"
            disabled={!translation.draft}
          >
            应用移动
          </button>
        </fieldset>
        <button
          type="button"
          disabled={busy || !translation.draft}
          onClick={onCancel}
        >
          取消移动
        </button>
      </form>
      {!!followers && (
        <p role="status">另有 {followers} 台挂接灯具随支撑体移动</p>
      )}
      {(problem || translation.problem || error) && (
        <p role="alert" className="wb-library-error">
          {problem || translation.problem || error}
        </p>
      )}
      <ol>
        {targets.map((t) => {
          const object = selectedStage(project.stage, t);
          const name =
            object?.kind === "placement"
              ? project.fixtures.find((f) => f.id === t.id)?.name
              : object?.value.name;
          return <li key={`${t.kind}:${t.id}`}>{name}</li>;
        })}
      </ol>
      <button disabled={busy} onClick={onClear}>
        清空选择
      </button>
    </aside>
  );
}
