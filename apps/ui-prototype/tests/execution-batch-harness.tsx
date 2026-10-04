// Actual production ExecutionBoard; test data/receipts, never a physical output host.
import { createRoot } from "react-dom/client";
import { useRef, useState } from "react";
import { ExecutionBoard } from "../src/components/execution/ExecutionBoard";
import { boardFixture } from "./execution-board-fixture";
import type {
  ExecutionAction,
  ExecutionBatchRequest,
  ExecutionStatus,
} from "../src/execution-types";
import "../src/base.css";
import "../src/workbench.css";
function ready() {
  const r = boardFixture();
  r.catalog.capabilities = ["sourceBatch"];
  r.observation.snapshot!.state.owner = {
    sessionId: "session",
    expiresMs: "60000",
  };
  return r;
}
function Harness() {
  const [runtime, setRuntime] = useState(ready);
  const [busy, setBusy] = useState(false);
  const [delay, setDelay] = useState(false);
  const [reject, setReject] = useState(false);
  const [throwError, setThrowError] = useState(false);
  const [unknown, setUnknown] = useState(false);
  const [readOnly, setReadOnly] = useState(false);
  const [observed, setObserved] = useState(true);
  const [active, setActive] = useState(true);
  const [narrow, setNarrow] = useState(false);
  const [requests, setRequests] = useState<ExecutionBatchRequest[]>([]);
  const latest = useRef(runtime);
  latest.current = runtime;
  async function batch(
    request: ExecutionBatchRequest,
  ): Promise<ExecutionStatus> {
    const next = structuredClone(runtime);
    setBusy(true);
    setRequests((old) => [...old, request]);
    if (delay) await new Promise((resolve) => setTimeout(resolve, 1500));
    if (throwError) {
      if (
        latest.current.hostId === next.hostId &&
        latest.current.sessionId === next.sessionId
      )
        setBusy(false);
      throw Error("测试：旧桥接失败");
    }
    const state = next.observation.snapshot!.state;
    if (!reject)
      for (const source of state.sources) {
        if (!request.sources.includes(source.id)) continue;
        if (request.action.kind === "pause" && source.status === "Running")
          source.status = "Paused";
        if (request.action.kind === "resume" && source.status === "Paused")
          source.status = "Running";
        if (request.action.kind === "stop") {
          source.status = "Idle";
          source.step = null;
        }
      }
    state.revision = (BigInt(state.revision) + 1n).toString();
    next.record = {
      serial: (BigInt(next.record?.serial ?? "0") + 1n).toString(),
      status: unknown ? "pending" : "complete",
      outcome: unknown
        ? null
        : {
            kind: reject ? "rejected" : "applied",
            message: reject ? "测试：整组拒绝，未部分控制" : null,
          },
    };
    next.pending = unknown;
    if (
      latest.current.hostId === request.hostId &&
      latest.current.sessionId === next.sessionId
    ) {
      setRuntime(next);
      setBusy(false);
    }
    return { phase: "connected", problem: null, runtime: next };
  }
  async function single(source: string, action: ExecutionAction) {
    const next = structuredClone(runtime),
      current = next.observation.snapshot!.state.sources.find(
        (s) => s.id === source,
      )!;
    if (action.kind === "level") current.level = action.value;
    if (action.kind === "start") {
      current.status = "Running";
      current.step = action.step;
    }
    if (action.kind === "pause") current.status = "Paused";
    if (action.kind === "resume") current.status = "Running";
    if (action.kind === "stop") current.status = "Idle";
    next.observation.snapshot!.state.revision = (
      BigInt(next.observation.snapshot!.state.revision) + 1n
    ).toString();
    setRuntime(next);
    return { phase: "connected" as const, problem: null, runtime: next };
  }
  const view = readOnly ? { ...runtime, controlling: false } : runtime;
  return (
    <main
      className="workbench"
      style={{
        display: "block",
        height: "100vh",
        overflow: "auto",
        padding: 20,
      }}
    >
      <div className="execution-buttons">
        {(
          [
            ["延迟回复（测试）", delay, setDelay],
            ["拒绝本次（测试）", reject, setReject],
            ["桥接失败（测试）", throwError, setThrowError],
            ["待确认（测试）", unknown, setUnknown],
            ["只读（测试）", readOnly, setReadOnly],
            ["隐藏执行面（测试）", !active, (v: boolean) => setActive(!v)],
            ["窄栏（测试）", narrow, setNarrow],
          ] as [string, boolean, (v: boolean) => void][]
        ).map(([name, value, set]) => (
          <label key={name}>
            <input
              type="checkbox"
              checked={value}
              onChange={(e) => set(e.target.checked)}
            />
            {name}
          </label>
        ))}
        <button onClick={() => setObserved((v) => !v)}>
          切换观察失败（测试）
        </button>
        <button
          onClick={() => {
            const r = structuredClone(runtime);
            r.sessionId = `${r.sessionId}-new`;
            r.observation.snapshot!.state.owner!.sessionId = r.sessionId;
            r.record = null;
            setRuntime(r);
            setBusy(false);
          }}
        >
          换控制会话（测试）
        </button>
        <button
          onClick={() => {
            setRuntime({ ...ready(), hostId: `${runtime.hostId}-new` });
            setBusy(false);
          }}
        >
          换后台身份（测试）
        </button>
        <button
          onClick={() => {
            const r = structuredClone(runtime);
            r.observation.snapshot!.state.revision = (
              BigInt(r.observation.snapshot!.state.revision) + 1n
            ).toString();
            setRuntime(r);
          }}
        >
          外部修订变化（测试）
        </button>
        <button
          onClick={() => {
            const r = structuredClone(runtime);
            r.pending = false;
            if (r.record) {
              r.record.status = "complete";
              r.record.outcome = { kind: "applied", message: null };
            }
            setRuntime(r);
          }}
        >
          确认原回执（测试）
        </button>
        <button onClick={() => setRequests([])}>清空请求记录（测试）</button>
      </div>
      <div
        hidden={!active}
        className="background-execution"
        style={{ width: narrow ? 266 : "100%", maxWidth: "100%" }}
      >
        <ExecutionBoard
          key={`${view.hostId}:${view.catalog.layout}`}
          runtime={view}
          disabled={busy || readOnly || !observed || runtime.pending}
          active={active}
          observed={observed}
          onAction={single}
          onMedia={async () => null}
          onBatch={batch}
        />
      </div>
      <pre aria-label="测试批量请求记录">{JSON.stringify(requests)}</pre>
    </main>
  );
}
const root = createRoot(document.getElementById("root")!);
root.render(<Harness />);
if (import.meta.hot) import.meta.hot.dispose(() => root.unmount());
