// Isolated UI interaction harness. No hardware, real project or production backend is contacted.
import { createRoot } from "react-dom/client";
import { useState } from "react";
import { BackgroundExecution } from "../src/components/execution/BackgroundExecution";
import { applicationHost } from "../src/hosts/application-host";
import { stageProject } from "./stage-organization-fixture";
import type { ExecutionRequest, ExecutionStatus } from "../src/execution-types";
import type { ApplicationHost } from "../src/application-host";
import "../src/base.css";
import "../src/workbench.css";
const project = stageProject();
project.scenes = [
  { id: "scene-one", name: "暖金侧光", effects: [], values: [] },
  { id: "scene-two", name: "蓝色呼吸", effects: [], values: [] },
];
project.sequences = [];
let status: ExecutionStatus = { phase: "empty", problem: null, runtime: null };
const history: ExecutionRequest[] = [];
let failPreparation = false;
let failObservation = false;
let observations = 0;
const host: ApplicationHost = {
  ...applicationHost,
  execution: async (request) => {
    if (request.kind === "snapshot") {
      await new Promise((resolve) => setTimeout(resolve, 250));
      document.querySelector("#observations")!.textContent = String(++observations);
      if (failObservation) throw new Error("测试读取暂时失败");
    } else history.push(request);
    if (request.kind === "prepare" && failPreparation)
      throw new Error("测试声音输出被另一窗口占用");
    if (request.kind === "prepare")
      status = {
        phase: "connected",
        problem: null,
        runtime: {
          hostId: "host",
          catalog: {
            projectId: project.id,
            layout: "test",
            physicalOutput: false,
            sources: request.selection.map((s, i) => ({
              id: String(i),
              name:
                "id" in s
                  ? project.scenes.find((x) => x.id === s.id)!.name
                  : "音乐编排",
              priority: 0,
              selection: s,
              steps: [{ id: `step-${i}`, name: "保持", number: "1" }],
            })),
          },
          observation: {
            phase: "running",
            fault: null,
            snapshot: {
              cycles: "100",
              missedPeriods: "0",
              state: {
                revision: "1",
                owner: null,
                fault: false,
                sources: request.selection.map((_, i) => ({
                  id: String(i),
                  level: 65535,
                  status: "Idle",
                  step: null,
                })),
              },
            },
          },
          sessionId: null,
          controlling: false,
          pending: false,
          record: null,
        },
      };
    if (request.kind === "acquire" && status.runtime) {
      status.runtime.controlling = true;
      status.runtime.sessionId = "session";
      status.runtime.observation.snapshot!.state.owner = {
        sessionId: "session",
        expiresMs: "60000",
      };
    }
    if (request.kind === "release" && status.runtime) {
      status.runtime.controlling = false;
      status.runtime.observation.snapshot!.state.owner = null;
    }
    if (request.kind === "apply" && status.runtime) {
      const state = status.runtime.observation.snapshot!.state;
      const source = state.sources.find((s) => s.id === request.source)!;
      if (request.action.kind === "start") {
        source.status = "Running";
        source.step = request.action.step;
      }
      if (request.action.kind === "pause") source.status = "Paused";
      if (request.action.kind === "resume") source.status = "Running";
      if (request.action.kind === "stop") {
        source.status = "Idle";
        source.step = null;
      }
      if (request.action.kind === "level") source.level = request.action.value;
      state.revision = String(Number(state.revision) + 1);
    }
    if (request.kind === "reconnect" && status.runtime) {
      status.runtime.controlling = false;
      status.runtime.sessionId = null;
    }
    if (request.kind === "shutdown")
      status = { phase: "empty", problem: null, runtime: null };
    document.querySelector("#requests")!.textContent = JSON.stringify(history);
    return structuredClone(status);
  },
};
function Harness() {
  const [visible, setVisible] = useState(true);
  return (
    <main
      className="workbench"
      style={{
        height: "100vh",
        overflow: "auto",
        padding: 24,
        display: "block",
      }}
    >
      <button onClick={() => setVisible((v) => !v)}>切换面板显示</button>
      <label>
        <input type="checkbox" onChange={(e) => { failPreparation = e.target.checked; }} />
        拒绝载入（测试）
      </label>
      <label>
        <input type="checkbox" onChange={(e) => { failObservation = e.target.checked; }} />
        读取失败（测试）
      </label>
      <output id="observations" aria-label="测试读取次数">0</output>
      <div hidden={!visible}>
        <BackgroundExecution
          host={host}
          project={project}
          generation={1}
          visible={visible}
          busy={false}
          beforeAction={async () => true}
        />
      </div>
      <pre id="requests" aria-label="测试操作记录" />
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
