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
  reject = false,
  unavailable = false;
const history: ExecutionRequest[] = [];
const host: ApplicationHost = {
  ...applicationHost,
  preview: async () => ({ epoch: 0, controlSerial: 0, loaded: null }),
  execution: async (request) => {
    if (request.kind === "snapshot" && unavailable)
      throw Error("测试：后台暂时不可读");
    if (request.kind !== "snapshot") history.push(request);
    if (request.kind === "apply") {
      const state = runtime.observation.snapshot!.state;
      if (!reject) {
        const source = state.sources.find((s) => s.id === request.source)!;
        if (request.action.kind === "level")
          source.level = request.action.value;
        if (request.action.kind === "stop") {
          source.held = [];
          source.heldValues = [];
        }
        if (request.action.kind === "patch")
          for (const change of request.action.changes) {
            const index = source.held!.findIndex(
              (t) =>
                t.fixtureId === change.fixtureId &&
                t.attribute === change.attribute,
            );
            if (index >= 0) {
              source.held!.splice(index, 1);
              source.heldValues!.splice(index, 1);
            }
            if (change.value.kind !== "release") {
              let value: number;
              if (change.value.kind === "normalized")
                value = change.value.value;
              else {
                const selected = change.value;
                const table = runtime.catalog
                  .fixtures!.find((f) => f.id === change.fixtureId)!
                  .attributes.find(
                    (a) => a.key === change.attribute,
                  )!.function!;
                const f = table.functions.find(
                  (f) => f.key === selected.functionKey,
                )!;
                const native =
                  f.mode === "slot"
                    ? f.dmxDefault
                    : f.dmxFrom +
                      Math.round(
                        ((f.dmxTo - f.dmxFrom) * selected.position) / 65535,
                      );
                value = native * (table.fine ? 1 : 257);
              }
              source.held!.push({
                fixtureId: change.fixtureId,
                attribute: change.attribute,
              });
              source.heldValues!.push(value);
            }
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
      <label>
        <input
          type="checkbox"
          onChange={(e) => {
            unavailable = e.target.checked;
          }}
        />
        暂停读取（测试）
      </label>
      <button
        onClick={() => {
          const state = runtime.observation.snapshot!.state.sources.find(
            (s) => s.id === "manual",
          )!;
          state.held = runtime.catalog.fixtures!.map((f) => ({
            fixtureId: f.id,
            attribute: "dimmer",
          }));
          state.heldValues = state.held.map((_, i) => (i % 2 ? 0 : 24576));
        }}
      >
        外部填入不同值（测试）
      </button>
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
