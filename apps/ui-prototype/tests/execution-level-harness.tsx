// Actual execution components with a delayed, observable test port. No hardware or media.
import { createRoot } from "react-dom/client";
import { useState } from "react";
import { BackgroundExecution } from "../src/components/execution/BackgroundExecution";
import { applicationHost } from "../src/hosts/application-host";
import type { ApplicationHost } from "../src/application-host";
import type {
  ExecutionRequest,
  ExecutionAction,
  ExecutionOutputAction,
} from "../src/execution-types";
import { stageProject } from "./stage-organization-fixture";
import { levelFixture } from "./execution-level-fixture";
import "../src/base.css";
import "../src/workbench.css";

const status = levelFixture();
const runtime = status.runtime!,
  state = runtime.observation.snapshot!.state;
runtime.catalog.capabilities!.push("outputMaster");
runtime.catalog.output = { uncontrolledFixtures: 0 };
state.output = { percent: 100, blackout: false };
const project = stageProject();
project.id = runtime.catalog.projectId;
const history: unknown[] = [];
let failed = false,
  rejected = false,
  delay = 300,
  serial = 0;
let pending: {
  at: number;
  source?: string;
  action: ExecutionAction | ExecutionOutputAction;
} | null = null;
const host: ApplicationHost = {
  ...applicationHost,
  execution: async (request: ExecutionRequest) => {
    await new Promise((resolve) => setTimeout(resolve, 5));
    if (failed) throw new Error("测试连接中断");
    if (pending && performance.now() >= pending.at) {
      const source = state.sources.find((s) => s.id === pending!.source)!;
      const action = pending.action;
      if (!rejected && action.kind === "level") {
        if ("percent" in action) state.output!.percent = action.percent;
        else source.level = action.value;
      }
      if (!rejected && action.kind === "blackout")
        state.output!.blackout = action.enabled;
      if (!rejected && action.kind === "patch") {
        for (const change of action.changes) {
          const index = source.held!.findIndex(
            (t) =>
              t.fixtureId === change.fixtureId &&
              t.attribute === change.attribute,
          );
          if (change.value.kind === "release") {
            if (index >= 0) {
              source.held!.splice(index, 1);
              source.heldValues!.splice(index, 1);
            }
          } else if (change.value.kind === "normalized") {
            if (index < 0) {
              source.held!.push({
                fixtureId: change.fixtureId,
                attribute: change.attribute,
              });
              source.heldValues!.push(change.value.value);
            } else source.heldValues![index] = change.value.value;
          }
        }
      }
      if (!rejected && action.kind === "stop") {
        source.held = [];
        source.heldValues = [];
      }
      state.revision = String(BigInt(state.revision) + 1n);
      runtime.pending = false;
      runtime.record = {
        serial: String(serial),
        status: "complete",
        outcome: {
          kind: rejected ? "rejected" : "applied",
          message: rejected ? "测试拒绝本次连续调整" : null,
        },
      };
      pending = null;
    }
    if (request.kind !== "snapshot")
      history.push({ at: Math.round(performance.now()), ...request });
    if (request.kind === "apply" || request.kind === "output") {
      if (
        pending ||
        !runtime.controlling ||
        request.revision !== state.revision
      )
        throw new Error("测试发现命令交叉或旧修订");
      pending = {
        at: performance.now() + delay,
        source: request.kind === "apply" ? request.source : undefined,
        action: request.action,
      };
      runtime.pending = true;
      runtime.record = {
        serial: String(++serial),
        status: "pending",
        outcome: null,
      };
    }
    if (request.kind === "release") {
      runtime.controlling = false;
      state.owner = null;
    }
    if (request.kind === "acquire") {
      runtime.controlling = true;
      state.owner = { sessionId: runtime.sessionId!, expiresMs: "60000" };
    }
    document.querySelector("#test-record")!.textContent = JSON.stringify({
      history,
      sources: state.sources,
      output: state.output,
      pending: runtime.pending,
    });
    return structuredClone(status);
  },
};
function Harness() {
  const [visible, setVisible] = useState(true),
    [narrow, setNarrow] = useState(false);
  return (
    <main
      className="workbench"
      style={{
        height: "100vh",
        overflow: "auto",
        display: "block",
        padding: 12,
      }}
    >
      <div style={{ display: "flex", gap: 12, flexWrap: "wrap" }}>
        <label>
          <input
            type="checkbox"
            onChange={(e) => {
              runtime.catalog.output!.uncontrolledFixtures = e.target.checked
                ? 1
                : 0;
            }}
          />
          未识别亮度（测试）
        </label>
        <button onClick={() => setVisible((v) => !v)}>切换显示（测试）</button>
        <button onClick={() => setNarrow((v) => !v)}>切换窄栏（测试）</button>
        <label>
          回执等待（测试）
          <select
            onChange={(e) => {
              delay = Number(e.target.value);
            }}
            defaultValue="300"
          >
            <option value="300">300 毫秒</option>
            <option value="1500">1.5 秒</option>
            <option value="20">20 毫秒</option>
          </select>
        </label>
        <label>
          <input
            type="checkbox"
            onChange={(e) => {
              failed = e.target.checked;
            }}
          />
          连接中断（测试）
        </label>
        <label>
          <input
            type="checkbox"
            onChange={(e) => {
              rejected = e.target.checked;
            }}
          />
          拒绝电平（测试）
        </label>
      </div>
      <div hidden={!visible} style={{ width: narrow ? 310 : "100%" }}>
        <BackgroundExecution
          host={host}
          project={project}
          generation={1}
          visible={visible}
          busy={false}
          beforeAction={async () => true}
        />
      </div>
      <pre
        id="test-record"
        aria-label="测试操作记录"
        style={{ whiteSpace: "pre-wrap", overflowWrap: "anywhere" }}
      />
    </main>
  );
}
const root = createRoot(document.getElementById("root")!);
root.render(<Harness />);
if (import.meta.hot) import.meta.hot.dispose(() => root.unmount());
