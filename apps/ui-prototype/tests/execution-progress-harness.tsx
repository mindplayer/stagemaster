// Isolated component fixtures. No physical output, project writes or backend transport.
import { createRoot } from "react-dom/client";
import { useState } from "react";
import { SourceControls } from "../src/components/execution/SourceControls";
import type {
  ExecutionAction,
  ExecutionSource,
  ExecutionView,
} from "../src/execution-types";
import type { SourceProgress } from "../src/execution-source-progress";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/execution/execution.css";
const source: ExecutionSource = {
  id: "source",
  name: "主舞台 · 剧本节目",
  priority: 0,
  selection: { kind: "sequence", id: "list" },
  steps: [
    { id: "a", number: "1", name: "开场·蓝色逆光" },
    { id: "b", number: "2", name: "主段·暖金侧光" },
  ],
};
const initial: ExecutionView = {
  hostId: "isolated",
  catalog: {
    projectId: "test",
    layout: "test",
    sources: [source],
    physicalOutput: false,
  },
  observation: {
    phase: "running",
    fault: null,
    snapshot: {
      cycles: "1",
      missedPeriods: "0",
      state: {
        revision: "1",
        owner: null,
        fault: false,
        sources: [
          {
            id: "source",
            level: 65535,
            status: "Running",
            step: "a",
            progress: {
              phase: "fade",
              elapsedMs: "2000",
              phaseElapsedMs: "1000",
              phaseDurationMs: "4000",
              nextStep: "b",
              nextWrap: false,
            },
          },
        ],
      },
    },
  },
  sessionId: "test",
  controlling: true,
  pending: false,
  record: null,
};
function Harness() {
  const [runtime, setRuntime] = useState(initial);
  const [observed, setObserved] = useState(true);
  const [readOnly, setReadOnly] = useState(false);
  const [narrow, setNarrow] = useState(false);
  const [actions, setActions] = useState<ExecutionAction[]>([]);
  function update(phase: SourceProgress["phase"], final = false, loop = false) {
    setRuntime((old) => {
      const v = structuredClone(old);
      const s = v.observation.snapshot!.state.sources[0];
      s.status = phase === "finished" ? "Finished" : "Running";
      s.step = final ? "b" : "a";
      s.progress = {
        phase,
        elapsedMs: "2500",
        phaseElapsedMs: phase === "finished" ? "0" : "1500",
        phaseDurationMs: ["delay", "fade", "wait"].includes(phase)
          ? "4000"
          : null,
        nextStep: phase === "finished" ? null : loop ? "a" : final ? null : "b",
        nextWrap: loop,
      };
      return v;
    });
  }
  function action(a: ExecutionAction) {
    setActions((v) => [...v, a]);
    setRuntime((old) => {
      const v = structuredClone(old);
      const s = v.observation.snapshot!.state.sources[0];
      if (a.kind === "pause") s.status = "Paused";
      if (a.kind === "resume") s.status = "Running";
      if (a.kind === "level") s.level = a.value;
      if (a.kind === "start" || a.kind === "next") {
        s.status = "Running";
        s.step = a.kind === "start" ? a.step : (s.progress?.nextStep ?? null);
        s.progress = {
          phase: "hold",
          elapsedMs: "0",
          phaseElapsedMs: "0",
          phaseDurationMs: null,
          nextStep: null,
          nextWrap: false,
        };
      }
      if (a.kind === "stop") {
        s.status = "Idle";
        s.step = null;
        s.progress = {
          phase: "idle",
          elapsedMs: "0",
          phaseElapsedMs: "0",
          phaseDurationMs: null,
          nextStep: null,
          nextWrap: false,
        };
      }
      return v;
    });
  }
  return (
    <main style={{ padding: 20, maxWidth: 1000 }}>
      <h1>独立组件验收</h1>
      <nav className="execution-buttons">
        <button onClick={() => update("delay")}>延时采样</button>
        <button onClick={() => update("fade")}>渐变采样</button>
        <button onClick={() => update("wait")}>自动等待采样</button>
        <button onClick={() => update("hold")}>人工等待采样</button>
        <button onClick={() => update("hold", true)}>末步保持采样</button>
        <button onClick={() => update("hold", true, true)}>循环末步采样</button>
        <button onClick={() => update("finished", true)}>结束采样</button>
        <button
          onClick={() =>
            setRuntime((old) => {
              const v = structuredClone(old);
              delete v.observation.snapshot!.state.sources[0].progress;
              return v;
            })
          }
        >
          旧主机采样
        </button>
      </nav>
      <label>
        <input
          type="checkbox"
          checked={!observed}
          onChange={(e) => setObserved(!e.target.checked)}
        />
        观察失败
      </label>
      <label>
        <input
          type="checkbox"
          checked={readOnly}
          onChange={(e) => setReadOnly(e.target.checked)}
        />
        只读连接
      </label>
      <label>
        <input
          type="checkbox"
          checked={narrow}
          onChange={(e) => setNarrow(e.target.checked)}
        />
        窄栏
      </label>
      <section
        className="background-execution"
        style={{ width: narrow ? 280 : 650, maxWidth: "100%" }}
      >
        <div className="execution-sources">
          <SourceControls
            source={source}
            runtime={runtime}
            observed={observed}
            disabled={!observed || readOnly}
            onAction={action}
          />
        </div>
      </section>
      <output aria-label="已发送操作">{JSON.stringify(actions)}</output>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
