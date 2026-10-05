// Real component interaction fixture, without backend, audio or lighting output.
import { useState } from "react";
import { createRoot } from "react-dom/client";
import { MediaControls } from "../src/components/execution/MediaControls";
import { mediaRequestIdentity } from "../src/media-seek-receipt";
import type { ExecutionView } from "../src/execution-types";
import type {
  ExecutionMediaAction,
  ExecutionMediaState,
} from "../src/execution-media-types";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/execution/execution.css";

function media(
  request = "7",
  status: "pending" | "applied" | "failed" | "timedOut" = "applied",
): ExecutionMediaState {
  return {
    id: "music",
    generation: "1",
    status: "Paused",
    positionMs: 2500,
    control: { request, status, problem: null },
  };
}
function initial(): ExecutionView {
  return {
    hostId: "fixture",
    sessionId: "controller",
    controlling: true,
    pending: false,
    catalog: {
      projectId: "fixture",
      layout: "layout",
      sources: [],
      physicalOutput: false,
      audio: {
        output: "software",
        durationMs: 5000,
        group: "music",
        seekIncludesEnd: true,
        performanceLoops: true,
      },
    },
    record: {
      serial: "3",
      status: "complete",
      outcome: {
        kind: "accepted",
        message: null,
        state: { media: [media("7", "pending")] },
      },
    },
    observation: {
      phase: "running",
      fault: null,
      snapshot: {
        cycles: "1",
        missedPeriods: "0",
        state: {
          revision: "1",
          sources: [],
          owner: { sessionId: "controller", expiresMs: "60000" },
          fault: false,
          media: [media()],
          audio: {
            output: "software",
            status: "paused",
            positionMs: 2500,
            durationMs: 5000,
            instance: "2",
            problem: null,
            loopState: {
              region: 0,
              name: "一秒候场",
              pass: "2",
              exitRequested: false,
              pendingExit: null,
            },
          },
        },
      },
    },
  };
}
function Harness() {
  const [runtime, setRuntime] = useState(initial);
  const [commands, setCommands] = useState<ExecutionMediaAction[]>([]);
  function show(
    kind: "accepted" | "rejected" | "unknown" | "missing",
    request = "7",
    status: "pending" | "applied" | "failed" | "timedOut" = "applied",
  ) {
    const next = structuredClone(runtime);
    next.record = {
      serial: "4",
      status: "complete",
      outcome:
        kind === "missing"
          ? null
          : {
              kind,
              message:
                kind === "rejected"
                  ? "循环播放目标已变化，请确认当前区段和遍次后重新操作"
                  : null,
              state:
                kind === "accepted" ? { media: [media("7", "pending")] } : null,
            },
    };
    next.observation.snapshot!.state.media = [media(request, status)];
    setRuntime(next);
    return next;
  }
  return (
    <main className="workbench" style={{ display: "block", padding: 24 }}>
      <h1>音乐回执真实组件验收</h1>
      <p>测试夹具：不连接后台，不播放声音，不控制灯具。</p>
      <div className="execution-buttons">
        <button onClick={() => show("rejected")}>模拟陈旧目标拒绝</button>
        <button onClick={() => show("unknown")}>模拟结果未知</button>
        <button onClick={() => show("missing")}>模拟回执未确认</button>
        <button onClick={() => show("accepted", "6")}>模拟接纳等待</button>
        <button onClick={() => show("accepted")}>模拟同请求完成</button>
        <button onClick={() => show("accepted", "8")}>模拟请求被替代</button>
        <button onClick={() => show("accepted", "7", "timedOut")}>
          模拟同请求超时
        </button>
      </div>
      {runtime.record?.outcome?.message && (
        <p role="alert">{runtime.record.outcome.message}</p>
      )}
      <MediaControls
        runtime={runtime}
        disabled={false}
        onAction={async (action) => {
          setCommands((old) => [...old, action]);
          return mediaRequestIdentity(show("rejected"));
        }}
      />
      <pre aria-label="组件提交指令">{JSON.stringify(commands, null, 2)}</pre>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
