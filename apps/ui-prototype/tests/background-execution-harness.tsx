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
const host: ApplicationHost = {
  ...applicationHost,
  execution: async (request) => {
    if (request.kind === "snapshot")
      await new Promise((resolve) => setTimeout(resolve, 250));
    else history.push(request);
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
              name: project.scenes.find((x) => x.id === s.id)!.name,
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
