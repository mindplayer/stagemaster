import { useEffect, useState } from "react";
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
  onAction,
}: {
  source: ExecutionSource;
  runtime: ExecutionView;
  disabled: boolean;
  onAction(action: ExecutionAction): void;
}) {
  const state = runtime.observation.snapshot?.state.sources.find(
    (s) => s.id === source.id,
  );
  const [step, setStep] = useState(source.steps[0]?.id ?? "");
  const [level, setLevel] = useState<string | null>(null);
  const [confirm, setConfirm] = useState(false);
  const current = source.steps.find((s) => s.id === state?.step);
  const value =
    level ?? String(Math.round((state?.level ?? 65535) / 65.535) / 10);
  useEffect(() => {
    if (level !== null && Math.round(Number(level) * 655.35) === state?.level)
      setLevel(null);
  }, [state?.level]);
  const valid =
    value.trim() !== "" &&
    Number.isFinite(Number(value)) &&
    Number(value) >= 0 &&
    Number(value) <= 100;
  return (
    <article className="execution-source" aria-label={source.name}>
      <header>
        <h3>{source.name}</h3>
        <span>
          {state?.status ? (names[state.status] ?? "状态未知") : "手动层"}
        </span>
      </header>
      {source.selection.kind !== "manual" && (
        <>
          <p className="execution-current">
            {current ? `${current.number} · ${current.name}` : "尚未执行"}
          </p>
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
                disabled={disabled || state?.status !== "Running"}
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
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (valid && !disabled) {
            onAction({
              kind: "level",
              value: Math.round(Number(value) * 655.35),
            });
          }
        }}
      >
        <label>
          亮度电平{" "}
          <input
            aria-label={`${source.name}亮度电平`}
            type="number"
            min="0"
            max="100"
            step="0.1"
            value={value}
            disabled={disabled}
            onChange={(e) => setLevel(e.target.value)}
          />{" "}
          %
        </label>
        <button disabled={disabled || level === null || !valid}>应用</button>
        {level !== null && (
          <button type="button" onClick={() => setLevel(null)}>
            取消
          </button>
        )}
      </form>
      {source.selection.kind === "manual" && (
        <button disabled={disabled} onClick={() => onAction({ kind: "stop" })}>
          释放手动层
        </button>
      )}
    </article>
  );
}
