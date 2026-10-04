import { useEffect, useState } from "react";
import { SourceLevelControls } from "./SourceLevelControls";
import type { LiveLevels } from "../../execution-level-gesture";
import { SourceProgress } from "./SourceProgress";
import { sourceProgress } from "../../execution-source-progress";
import type {
  ExecutionAction,
  ExecutionSource,
  ExecutionView,
} from "../../execution-types";
const names: Record<string, string> = {
  Idle: "待执行",
  Running: "运行中",
  Paused: "已暂停",
  Finished: "已结束",
};
export function SourceControls({
  source,
  runtime,
  disabled,
  observed = true,
  active = true,
  hideManualRelease = false,
  live,
  onDraftChange,
  onAction,
}: {
  source: ExecutionSource;
  runtime: ExecutionView;
  disabled: boolean;
  observed?: boolean;
  active?: boolean;
  hideManualRelease?: boolean;
  live?: LiveLevels;
  onDraftChange?(id: string, dirty: boolean): void;
  onAction(action: ExecutionAction): void;
}) {
  const state = runtime.observation.snapshot?.state.sources.find(
    (s) => s.id === source.id,
  );
  const [step, setStep] = useState(source.steps[0]?.id ?? "");
  const [confirm, setConfirm] = useState(false);
  const progress = sourceProgress(source, state);
  useEffect(() => {
    if (!active) setConfirm(false);
  }, [active]);
  return (
    <article className="execution-source" aria-label={source.name}>
      <header>
        <h3>{source.name}</h3>
        <span>
          {!observed
            ? "状态未更新"
            : state?.status
              ? (names[state.status] ?? "状态未知")
              : source.selection.kind === "manual"
                ? "手动层"
                : "状态未知"}
        </span>
      </header>
      {source.selection.kind !== "manual" && (
        <>
          <SourceProgress
            source={source}
            runtime={runtime}
            observed={observed}
          />
          <label>
            起始步骤
            <select
              aria-label={`${source.name}起始步骤`}
              value={step}
              disabled={disabled}
              onChange={(e) => {
                setStep(e.target.value);
                setConfirm(false);
              }}
            >
              {source.steps.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.number} · {s.name}
                </option>
              ))}
            </select>
          </label>
          <div className="execution-buttons">
            <button
              className="wb-primary"
              disabled={disabled || !step}
              onClick={() => {
                if (state?.status === "Running" || state?.status === "Paused")
                  setConfirm(true);
                else onAction({ kind: "start", step });
              }}
            >
              执行所选
            </button>
            <button
              disabled={
                disabled ||
                (state?.status !== "Running" && state?.status !== "Paused")
              }
              onClick={() =>
                onAction({
                  kind: state?.status === "Paused" ? "resume" : "pause",
                })
              }
            >
              {state?.status === "Paused" ? "继续" : "暂停"}
            </button>
            {source.selection.kind === "sequence" && (
              <button
                disabled={
                  disabled ||
                  !observed ||
                  state?.status !== "Running" ||
                  (!!state.progress && !progress?.next)
                }
                onClick={() => onAction({ kind: "next" })}
              >
                下一步
              </button>
            )}
            <button
              disabled={disabled || state?.status === "Idle"}
              onClick={() => onAction({ kind: "stop" })}
            >
              停止
            </button>
          </div>
          {confirm && (
            <div role="alert" className="execution-confirm">
              从所选步骤重新执行？
              <button
                disabled={disabled}
                onClick={() => {
                  setConfirm(false);
                  onAction({ kind: "start", step });
                }}
              >
                确认执行
              </button>
              <button onClick={() => setConfirm(false)}>取消</button>
            </div>
          )}
        </>
      )}
      <SourceLevelControls
        id={source.id}
        name={source.name}
        level={state?.level}
        disabled={disabled}
        active={active}
        observed={observed}
        live={live}
        onDraftChange={onDraftChange}
        onAction={onAction}
      />
      {source.selection.kind === "manual" && !hideManualRelease && (
        <button disabled={disabled} onClick={() => onAction({ kind: "stop" })}>
          释放手动层
        </button>
      )}
    </article>
  );
}
