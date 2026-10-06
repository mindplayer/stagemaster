import { useEffect, useState } from "react";
import type { ApplicationHost, ProjectView } from "../../application-host";
import { executionCommands } from "./executionCommands";
import { useLiveLevels } from "./useLiveLevels";
import { useExecution } from "./useExecution";
import { PrepareSources } from "./PrepareSources";
import { ExecutionBoard } from "./ExecutionBoard";
import { OutputMasterControls } from "./OutputMasterControls";
import "./execution.css";

export function BackgroundExecution({
  host,
  project,
  generation,
  visible,
  busy,
  beforeAction,
  recording,
}: {
  host: ApplicationHost;
  project: ProjectView;
  generation: number;
  visible: boolean;
  busy: boolean;
  beforeAction(): Promise<boolean>;
  recording?: import("../../manual-capture-types").ManualRecordingContext;
}) {
  const { status, error, working, fresh, request } = useExecution(
    host.execution,
    visible,
  );
  const live = useLiveLevels(
    status,
    visible && fresh && !error,
    working,
    request,
  );
  const interacting = live.view.key !== null;
  const [confirm, setConfirm] = useState<{
    kind: "takeover" | "shutdown" | "reconnect";
    hostId: string | null;
  } | null>(null);
  useEffect(() => {
    if (!visible) setConfirm(null);
  }, [visible]);
  const runtime = status?.runtime;
  const state = runtime?.observation.snapshot?.state;
  const disabled =
    !visible ||
    !fresh ||
    working ||
    interacting ||
    !!error ||
    !!status?.problem ||
    status?.phase !== "connected" ||
    !runtime?.controlling ||
    runtime.observation.phase !== "running" ||
    runtime.pending ||
    !!state?.fault;
  const commands = executionCommands(runtime, disabled, live.isBusy, request);
  const outcome = runtime?.record?.outcome;
  const operationProblem =
    live.view.problem || (!interacting ? outcome?.message : null);
  return (
    <section className="background-execution" aria-label="后台执行">
      <header className="execution-heading">
        <div>
          <span className="wb-eyebrow">后台执行 · 软件输出</span>
          <h2>{runtime ? "已准备节目" : "后台节目"}</h2>
        </div>
        <span>
          {status?.phase === "closing"
            ? "正在关闭"
            : runtime && !fresh
              ? "等待刷新状态"
              : runtime?.controlling
                ? "当前拥有控制权"
                : runtime
                  ? "只读观察"
                  : status?.phase === "empty"
                    ? "尚未载入"
                    : "等待连接"}
        </span>
      </header>
      <p className="wb-dim">
        载入后使用固定版本，关闭编辑窗口后仍继续运行。灯光当前为软件输出；音乐按所选声音输出播放。
      </p>
      <div className="execution-buttons">
        <button
          disabled={working || interacting}
          onClick={() => void request({ kind: "snapshot" })}
        >
          刷新状态
        </button>
        {status?.phase !== "empty" && (
          <button
            disabled={working || interacting}
            onClick={() =>
              setConfirm({ kind: "reconnect", hostId: runtime?.hostId ?? null })
            }
          >
            重新连接
          </button>
        )}
        {runtime && !runtime.controlling && (
          <button
            className="wb-primary"
            disabled={
              !visible ||
              !fresh ||
              working ||
              interacting ||
              runtime.pending ||
              !!error ||
              !!status?.problem
            }
            onClick={() => void request({ kind: "acquire", takeover: false })}
          >
            取得控制权
          </button>
        )}
        {runtime && !runtime.controlling && state?.owner && (
          <button
            disabled={
              !visible || !fresh || working || interacting || runtime.pending
            }
            onClick={() =>
              setConfirm({ kind: "takeover", hostId: runtime?.hostId ?? null })
            }
          >
            接管控制权
          </button>
        )}
        {runtime?.controlling && (
          <button
            disabled={
              !visible || !fresh || working || interacting || runtime.pending
            }
            onClick={() => void request({ kind: "release" })}
          >
            归还控制权
          </button>
        )}
        {runtime && (
          <button
            disabled={working || interacting}
            onClick={() =>
              setConfirm({ kind: "shutdown", hostId: runtime?.hostId ?? null })
            }
          >
            关闭后台
          </button>
        )}
      </div>
      {(error || status?.problem) && (
        <p role="alert" className="wb-preview-warning">
          {error || status?.problem}
        </p>
      )}
      {runtime?.observation.phase === "faulted" && (
        <p role="alert" className="wb-preview-warning">
          后台执行发生故障，请关闭后台后重新载入。
        </p>
      )}
      {runtime && (
        <div
          className="execution-operation-status"
          role={operationProblem ? "alert" : "status"}
        >
          {operationProblem ||
            (runtime.pending && !interacting
              ? "操作结果尚未确认，正在核对原回执。"
              : "")}
        </div>
      )}
      {confirm && (
        <div
          role="alertdialog"
          aria-label="确认后台操作"
          className="execution-confirm"
        >
          <p>
            {confirm.kind === "shutdown"
              ? "关闭后台将停止这组全部节目。是否继续？"
              : confirm.kind === "takeover"
                ? "接管后，原控制端将失去操作权限。是否继续？"
                : "重新连接会重建控制会话，不重复发送未确认操作，也不停止节目。是否继续？"}
          </p>
          <button
            disabled={working || interacting}
            onClick={() => {
              const value = confirm;
              setConfirm(null);
              if (value.hostId !== (runtime?.hostId ?? null)) return;
              void request(
                value.kind === "shutdown"
                  ? { kind: "shutdown", hostId: value.hostId! }
                  : value.kind === "takeover"
                    ? { kind: "acquire", takeover: true }
                    : { kind: "reconnect" },
              );
            }}
          >
            确认
          </button>
          <button
            disabled={working || interacting}
            onClick={() => setConfirm(null)}
          >
            取消
          </button>
        </div>
      )}
      {runtime ? (
        <>
          {runtime.catalog.projectId !== project.id && (
            <p role="status">
              后台运行的是另一工程；当前编辑不会改变后台节目。
            </p>
          )}
          <OutputMasterControls
            key={`output:${runtime.hostId}:${runtime.catalog.layout}`}
            runtime={runtime}
            live={live}
            disabled={disabled}
            active={visible}
            observed={
              fresh &&
              !error &&
              !status?.problem &&
              status?.phase === "connected" &&
              runtime.observation.phase === "running" &&
              !runtime.observation.fault &&
              !state?.fault
            }
            onAction={(action) => {
              void commands.output(action);
            }}
          />
          <ExecutionBoard
            live={live}
            recording={recording}
            key={`${runtime.hostId}:${runtime.catalog.layout}`}
            runtime={runtime}
            disabled={disabled}
            active={visible}
            observed={
              fresh &&
              !error &&
              !status?.problem &&
              status?.phase === "connected" &&
              runtime.observation.phase === "running" &&
              !runtime.observation.fault &&
              !state?.fault
            }
            onAction={commands.source}
            onMedia={commands.media}
            onBatch={commands.batch}
          />
        </>
      ) : (
        <PrepareSources
          key={project.id}
          project={project}
          disabled={working || busy || !status || status.phase === "closing"}
          onPrepare={(selection, audioOutput) => {
            void (async () => {
              if (await beforeAction())
                await request({
                  kind: "prepare",
                  generation,
                  selection,
                  audioOutput,
                });
            })();
          }}
        />
      )}
    </section>
  );
}
