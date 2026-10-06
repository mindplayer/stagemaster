// Actual execution components with manually completed test reads. No hardware or audio.
import { useState } from "react";
import { createRoot } from "react-dom/client";
import { BackgroundExecution } from "../src/components/execution/BackgroundExecution";
import { applicationHost } from "../src/hosts/application-host";
import type { ApplicationHost } from "../src/application-host";
import type { ExecutionStatus } from "../src/execution-types";
import { levelFixture } from "./execution-level-fixture";
import { stageProject } from "./stage-organization-fixture";
import "../src/base.css";
import "../src/workbench.css";

type Connection = "甲" | "乙";
type Pending = {
  resolve(value: ExecutionStatus): void;
  reject(error: Error): void;
};
const pending: Record<Connection, Pending[]> = { 甲: [], 乙: [] };
const calls: { connection: Connection; kind: string }[] = [];
let notify = () => {};
const status = (connection: Connection): ExecutionStatus => {
  const value = levelFixture();
  value.runtime!.hostId = `host-${connection}`;
  value.runtime!.catalog.sources[0].name = `连接${connection}节目`;
  return value;
};
const hosts = Object.fromEntries(
  (["甲", "乙"] as const).map((connection) => [
    connection,
    {
      ...applicationHost,
      execution: async (request) => {
        calls.push({ connection, kind: request.kind });
        if (calls.length > 100) throw new Error("测试操作超过上限");
        if (request.kind !== "snapshot") {
          notify();
          return status(connection);
        }
        return new Promise<ExecutionStatus>((resolve, reject) => {
          pending[connection].push({ resolve, reject });
          notify();
        });
      },
    } satisfies ApplicationHost,
  ]),
) as Record<Connection, ApplicationHost>;
const project = stageProject();
project.id = levelFixture().runtime!.catalog.projectId;
function Harness() {
  const [connection, setConnection] = useState<Connection>("甲");
  const [visible, setVisible] = useState(true);
  const [, update] = useState(0);
  notify = () => update((value) => value + 1);
  function complete(id: Connection, fail = false) {
    const next = pending[id].shift();
    if (next) {
      if (fail) next.reject(new Error(`连接${id}测试读取失败`));
      else next.resolve(status(id));
    }
    notify();
  }
  return (
    <main
      className="workbench"
      style={{
        display: "block",
        padding: 12,
        height: "100vh",
        overflow: "auto",
      }}
    >
      <h1>执行观察归属隔离验收</h1>
      <p role="status">
        当前连接：{connection}；面板：{visible ? "显示" : "隐藏"}
      </p>
      <button
        onClick={() => setConnection((value) => (value === "甲" ? "乙" : "甲"))}
      >
        切换连接
      </button>
      <button onClick={() => setVisible((value) => !value)}>切换显示</button>
      {(["甲", "乙"] as const).map((id) => (
        <span key={id}>
          <button disabled={!pending[id].length} onClick={() => complete(id)}>
            完成{id}读取
          </button>
          <button
            disabled={!pending[id].length}
            onClick={() => complete(id, true)}
          >
            拒绝{id}读取
          </button>
        </span>
      ))}
      <pre aria-label="测试请求记录">
        {JSON.stringify({
          calls,
          pending: { 甲: pending.甲.length, 乙: pending.乙.length },
        })}
      </pre>
      <div hidden={!visible}>
        <BackgroundExecution
          host={hosts[connection]}
          project={project}
          generation={1}
          visible={visible}
          busy={false}
          beforeAction={async () => true}
        />
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
