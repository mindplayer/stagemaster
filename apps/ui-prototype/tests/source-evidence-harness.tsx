// Real component; synthetic observations only, no background/audio/device connections.
import { useState } from "react";
import { createRoot } from "react-dom/client";
import { SourceControls } from "../src/components/execution/SourceControls";
import type { ExecutionAction, ExecutionView } from "../src/execution-types";
import { manualFixture } from "./execution-manual-fixture";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/execution/execution.css";

function fixture(): ExecutionView {
  const view = manualFixture();
  const source = view.catalog.sources[0];
  view.sourceOperation = {
    target: {
      hostId: view.hostId,
      revision: "9007199254740994",
      source: source.id,
      action: { kind: "pause" },
    },
    serial: "9007199254740995",
    attempted: true,
    submission: { status: 200, bodyComplete: true, code: null, problem: null },
    receiptRead: null,
    receipt: {
      serial: "9007199254740995",
      complete: true,
      outcome: "applied",
      code: null,
      sourceState: {
        revision: "9007199254740996",
        status: "Paused",
        step: null,
      },
    },
    notSubmittedReason: null,
  };
  return view;
}
type Case =
  | "paused"
  | "renewed"
  | "rejected"
  | "unknown"
  | "notSent"
  | "pending"
  | "damaged"
  | "wrongSerial";
function Harness() {
  const [runtime, setRuntime] = useState(manualFixture);
  const [commands, setCommands] = useState<ExecutionAction[]>([]);
  function show(kind: Case) {
    const next = fixture();
    const value = next.sourceOperation!;
    if (kind === "renewed")
      next.record = {
        serial: "999",
        status: "complete",
        outcome: { kind: "renewed", message: null },
      };
    if (kind === "rejected" || kind === "unknown") {
      value.receipt!.outcome = kind;
      value.receipt!.code = "Deadline";
    }
    if (kind === "notSent") {
      value.target = null;
      value.serial = null;
      value.attempted = false;
      value.receipt = null;
      value.submission = {
        status: null,
        bodyComplete: false,
        code: null,
        problem: null,
      };
      value.notSubmittedReason = "sendPreflight";
    }
    if (kind === "pending") {
      value.submission = {
        status: 503,
        bodyComplete: true,
        code: null,
        problem: "httpRefused",
      };
      value.receiptRead = {
        status: 409,
        bodyComplete: true,
        code: "notRetained",
        problem: "httpRefused",
      };
      value.receipt = null;
      next.pending = true;
    }
    if (kind === "damaged") {
      value.submission.problem = "invalidJson";
      value.receiptRead = {
        status: 200,
        bodyComplete: true,
        code: null,
        problem: null,
      };
    }
    if (kind === "wrongSerial") value.receipt!.serial = "123";
    setRuntime(next);
  }
  return (
    <main className="workbench" style={{ display: "block", padding: 24 }}>
      <h1>节目控制原请求真实组件验收</h1>
      <p>仅合成观察，不连接后台、不播放声音、不控制灯具。</p>
      <div className="execution-buttons">
        <button onClick={() => show("paused")}>模拟当次暂停</button>
        <button onClick={() => show("renewed")}>模拟维护与后继运行</button>
        <button onClick={() => show("rejected")}>模拟拒绝附旧状态</button>
        <button onClick={() => show("unknown")}>模拟未知附旧状态</button>
        <button onClick={() => show("notSent")}>模拟本地未提交</button>
        <button onClick={() => show("pending")}>模拟503与原查询409</button>
        <button onClick={() => show("damaged")}>模拟损坏响应后原回执</button>
        <button onClick={() => show("wrongSerial")}>模拟不匹配回执</button>
      </div>
      <SourceControls
        source={runtime.catalog.sources[0]}
        runtime={runtime}
        disabled={runtime.pending}
        onAction={(action) => setCommands((old) => [...old, action])}
      />
      <pre aria-label="组件提交指令">{JSON.stringify(commands, null, 2)}</pre>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
