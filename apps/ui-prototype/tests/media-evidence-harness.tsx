// Real React components; explicitly synthetic state, no host/audio/lighting control.
import { useState } from "react";
import { createRoot } from "react-dom/client";
import { MediaControls } from "../src/components/execution/MediaControls";
import type { ExecutionMediaAction } from "../src/execution-media-types";
import { evidenceView, http } from "./media-evidence-fixture";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/execution/execution.css";

type Case =
  | "rejected"
  | "refusedState"
  | "maintained"
  | "notSent"
  | "damaged"
  | "refused"
  | "applied"
  | "superseded";
function Harness() {
  const [runtime, setRuntime] = useState(evidenceView);
  const [commands, setCommands] = useState<ExecutionMediaAction[]>([]);
  function show(kind: Case) {
    const next = evidenceView();
    const evidence = next.mediaOperation!;
    if (kind === "refusedState") evidence.receipt!.mediaRequest = "7";
    if (kind === "maintained")
      next.record = {
        serial: "9007199254740994",
        status: "complete",
        outcome: { kind: "renewed", message: null },
      };
    if (kind === "notSent") {
      next.mediaOperation = {
        target: null,
        serial: null,
        attempted: false,
        submission: http(null),
        receiptRead: null,
        receipt: null,
        notSubmittedReason: "invalidTarget",
      };
    }
    if (kind === "refused") {
      evidence.submission = { ...http(503), problem: "httpRefused" };
      evidence.receiptRead = {
        ...http(409),
        code: "notRetained",
        problem: "httpRefused",
      };
      evidence.receipt = null;
      next.pending = true;
      next.record = {
        serial: evidence.serial!,
        status: "pending",
        outcome: null,
      };
    }
    if (["damaged", "applied", "superseded"].includes(kind)) {
      if (kind === "damaged") evidence.submission.problem = "invalidJson";
      evidence.receipt!.outcome = "accepted";
      evidence.receipt!.code = null;
      evidence.receipt!.mediaRequest = "7";
      next.record!.outcome = {
        kind: "accepted",
        message: null,
        state: {
          media: [
            {
              ...next.observation.snapshot!.state.media![0],
              control: { request: "7", status: "pending", problem: null },
            },
          ],
        },
      };
      if (kind === "superseded")
        next.observation.snapshot!.state.media![0].control!.request = "8";
    }
    setRuntime(next);
  }
  return (
    <main className="workbench" style={{ display: "block", padding: 24 }}>
      <h1>音乐请求详情真实组件验收</h1>
      <p>
        测试夹具：不连接后台，不播放声音，不控制灯具。下列按钮只切换合成观察。
      </p>
      <div className="execution-buttons">
        <button onClick={() => show("rejected")}>模拟目标拒绝</button>
        <button onClick={() => show("refusedState")}>
          模拟拒绝附旧媒体状态
        </button>
        <button onClick={() => show("maintained")}>模拟维护换回执</button>
        <button onClick={() => show("notSent")}>模拟本地未提交</button>
        <button onClick={() => show("refused")}>模拟提交503与原查询409</button>
        <button onClick={() => show("damaged")}>
          模拟损坏响应后原回执完成
        </button>
        <button onClick={() => show("applied")}>模拟同一请求完成</button>
        <button onClick={() => show("superseded")}>模拟其他请求完成</button>
      </div>
      <MediaControls
        runtime={runtime}
        disabled={false}
        onAction={async (action) => {
          setCommands((old) => [...old, action]);
          show("rejected");
          return null;
        }}
      />
      <pre aria-label="组件提交指令">{JSON.stringify(commands, null, 2)}</pre>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
