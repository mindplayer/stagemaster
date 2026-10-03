import { useState } from "react";
import { createRoot } from "react-dom/client";
import { DeviceCenter } from "../src/components/devices/DeviceCenter";
import { applicationHost } from "../src/hosts/application-host";
import { fixture } from "./device-runtime-fixture";
import "../src/base.css";
import "../src/workbench.css";
const mock = fixture();
const host = {
  ...applicationHost,
  kind: "desktop" as const,
  device: mock.device,
  deviceRuntime: mock.port,
};
function Harness() {
  const [, redraw] = useState(0);
  mock.subscribe(() => redraw((n) => n + 1));
  return (
    <div className="workbench">
      <header className="workbench-header">
        <strong>设备运行界面验收 · 无真实设备</strong>
        <DeviceCenter host={host} />
      </header>
      <section style={{ padding: 24, maxWidth: 500 }}>
        <h1>设备状态与故障注入</h1>
        <button onClick={mock.fail}>下次操作发送后断线</button>
        <button onClick={mock.hold}>保持下次目录回复</button>
        <button onClick={mock.release}>释放目录回复</button>
        <button onClick={mock.foreign}>其他控制者接管</button>
        <button onClick={mock.failRead}>状态读取失败</button>
        <button onClick={mock.recoverRead}>恢复状态读取</button>
        <pre aria-label="实际请求记录">
          {mock.actions.join("\n") || "没有控制操作"}
        </pre>
      </section>
    </div>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
