import type { ExecutionSource, ExecutionView } from "../../execution-types";
import { sourceProgress } from "../../execution-source-progress";
import "./source-progress.css";

export function SourceProgress({
  source,
  runtime,
  observed,
}: {
  source: ExecutionSource;
  runtime: ExecutionView;
  observed: boolean;
}) {
  const state = runtime.observation.snapshot?.state.sources.find(
    (s) => s.id === source.id,
  );
  const current = source.steps.find((s) => s.id === state?.step);
  const progress = sourceProgress(source, state);
  return (
    <div className="source-progress">
      <p className="execution-current">
        <span>当前</span>
        {current ? `${current.number} · ${current.name}` : "尚未执行"}
      </p>
      {!observed ? (
        <p className="source-progress-unavailable">
          状态更新中断 · 当前为最后已知步骤
        </p>
      ) : !progress ? (
        <p className="source-progress-unavailable">步骤进度不可用</p>
      ) : progress.phase === "idle" ? null : (
        <>
          <div className="source-progress-phase">
            <strong>{progress.label}</strong>
            {progress.remaining !== null && (
              <span>剩余 {progress.remaining}</span>
            )}
          </div>
          {progress.percent !== null && (
            <progress
              max={100}
              value={progress.percent}
              aria-label={`${source.name}${progress.label}进度`}
            />
          )}
          <p className="source-progress-elapsed">
            本步已执行 {progress.elapsed}
          </p>
          {source.selection.kind === "sequence" && (
            <p className="source-progress-next">
              <span>下一步</span>
              {progress.next ?? "无后续步骤"}
            </p>
          )}
        </>
      )}
    </div>
  );
}
