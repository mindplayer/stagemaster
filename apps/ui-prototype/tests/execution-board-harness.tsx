// Isolated 64-source interface fixture. No backend, audio or physical output.
import { createRoot } from "react-dom/client";
import { useState } from "react";
import { SequenceExecutionPanel } from "../src/components/execution/SequenceExecutionPanel";
import { applicationHost } from "../src/hosts/application-host";
import { stageProject } from "./stage-organization-fixture";
import { boardFixture } from "./execution-board-fixture";
import type { ApplicationHost } from "../src/application-host";
import type { ExecutionRequest, ExecutionStatus } from "../src/execution-types";
import "../src/base.css";
import "../src/workbench.css";
const project = stageProject();
project.id = "board-test-project";
let runtime = boardFixture();
let unavailable = false;
let readonly = false;
const history: ExecutionRequest[] = [];
const host: ApplicationHost = {
  ...applicationHost,
  preview: async () => ({ epoch: 0, controlSerial: 0, loaded: null }),
  execution: async (request) => {
    if (request.kind === "snapshot") {
      if (unavailable) throw Error("测试状态读取失败");
    } else history.push(request);
    if (request.kind === "apply") {
      const s = runtime.observation.snapshot!.state.sources.find(
        (s) => s.id === request.source,
      )!;
      if (request.action.kind === "level") s.level = request.action.value;
      if (request.action.kind === "start") {
        s.status = "Running";
        s.step = request.action.step;
      }
      if (request.action.kind === "pause") s.status = "Paused";
      if (request.action.kind === "resume") s.status = "Running";
      if (request.action.kind === "stop") {
        s.status = "Idle";
        s.step = null;
      }
    }
    document.querySelector("#requests")!.textContent = JSON.stringify(history);
    return {
      phase: "connected",
      problem: null,
      runtime: { ...structuredClone(runtime), controlling: !readonly },
    } satisfies ExecutionStatus;
  },
};
function Harness() {
  const [background, setBackground] = useState(true);
  const [execution, setExecution] = useState(true);
  const [visible, setVisible] = useState(true);
  const [narrow, setNarrow] = useState(false);
  return (
    <main
      className="workbench"
      style={{
        display: "block",
        padding: 20,
        height: "100vh",
        overflow: "auto",
      }}
    >
      <div>
        <button onClick={() => setExecution((v) => !v)}>
          切换编排／执行（测试）
        </button>
        <button onClick={() => setVisible((v) => !v)}>
          切换面板显示（测试）
        </button>
        <button
          onClick={() => {
            runtime = { ...boardFixture(), hostId: `${runtime.hostId}-new` };
          }}
        >
          换后台身份（测试）
        </button>
        <label>
          <input
            type="checkbox"
            onChange={(e) => {
              unavailable = e.target.checked;
            }}
          />
          读取失败（测试）
        </label>
        <label>
          <input
            type="checkbox"
            onChange={(e) => {
              readonly = e.target.checked;
            }}
          />
          只读（测试）
        </label>
        <label>
          <input
            type="checkbox"
            onChange={(e) => setNarrow(e.target.checked)}
          />
          窄屏（测试）
        </label>
      </div>
      <div
        hidden={!visible}
        style={{ width: narrow ? 380 : "100%", maxWidth: "100%" }}
      >
        <SequenceExecutionPanel
          host={host}
          project={project}
          generation={1}
          visible={visible}
          busy={false}
          stepId=""
          beforeAction={async () => true}
          execution={execution}
          background={background}
          onBackgroundChange={setBackground}
        />
      </div>
      <pre id="requests" aria-label="测试操作记录">
        []
      </pre>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
