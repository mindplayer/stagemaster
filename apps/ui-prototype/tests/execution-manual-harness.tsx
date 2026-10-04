import { createRoot } from "react-dom/client";
import { SequenceExecutionPanel } from "../src/components/execution/SequenceExecutionPanel";
import { applicationHost } from "../src/hosts/application-host";
import { stageProject } from "./stage-organization-fixture";
import { manualFixture } from "./execution-manual-fixture";
import type { ApplicationHost } from "../src/application-host";
import type { ExecutionRequest } from "../src/execution-types";
import { useState } from "react";
import "../src/base.css";
import "../src/workbench.css";
const project = stageProject();
let runtime = manualFixture();
project.id = runtime.catalog.projectId;
let readonly = false,
  reject = false;
const history: ExecutionRequest[] = [];
const host: ApplicationHost = {
  ...applicationHost,
  preview: async () => ({ epoch: 0, controlSerial: 0, loaded: null }),
  execution: async (request) => {
    if (request.kind !== "snapshot") history.push(request);
    if (request.kind === "apply") {
      const state = runtime.observation.snapshot!.state;
      if (!reject) {
        const source = state.sources.find((s) => s.id === request.source)!;
        if (request.action.kind === "level")
          source.level = request.action.value;
        if (request.action.kind === "stop") source.held = [];
        if (request.action.kind === "patch")
          for (const change of request.action.changes) {
            source.held = source.held!.filter(
              (t) =>
                t.fixtureId !== change.fixtureId ||
                t.attribute !== change.attribute,
            );
            if (change.value.kind !== "release")
              source.held.push({
                fixtureId: change.fixtureId,
                attribute: change.attribute,
              });
          }
      }
      state.revision = String(Number(state.revision) + 1);
      runtime.record = {
        serial: String(history.length),
        status: "complete",
        outcome: {
          kind: reject ? "rejected" : "applied",
          message: reject ? "测试拒绝：后台版本已更新" : null,
        },
      };
    }
    document.querySelector("#requests")!.textContent = JSON.stringify(history);
    return {
      phase: "connected",
      problem: null,
      runtime: { ...structuredClone(runtime), controlling: !readonly },
    };
  },
};
function Harness() {
  const [background, setBackground] = useState(true),
    [narrow, setNarrow] = useState(false);
  return (
    <main
      className="workbench"
      style={{
        display: "block",
        height: "100vh",
        overflow: "auto",
        padding: 16,
      }}
    >
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
          onChange={(e) => {
            reject = e.target.checked;
          }}
        />
        拒绝操作（测试）
      </label>
      <label>
        <input type="checkbox" onChange={(e) => setNarrow(e.target.checked)} />
        窄栏（测试）
      </label>
      <button
        onClick={() => {
          runtime = { ...manualFixture(), hostId: `${runtime.hostId}-new` };
        }}
      >
        更换后台（测试）
      </button>
      <div style={{ width: narrow ? 380 : "100%", maxWidth: "100%" }}>
        <SequenceExecutionPanel
          project={project}
          host={host}
          generation={1}
          visible
          busy={false}
          stepId=""
          beforeAction={async () => true}
          execution
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
