// Isolated component acceptance only. Not included in production entry points or builds.
import React from "react";
import { createRoot } from "react-dom/client";
import type { ApplicationHost } from "../src/application-host";
import type {
  InstallationTask,
  InstallationView,
} from "../src/installation-types";
import { useInstallation } from "../src/components/installation/useInstallation";
import { InstallationCenter } from "../src/components/installation/InstallationCenter";
import "../src/base.css";
import "../src/workbench.css";
const task: InstallationTask = {
  id: "test-install",
  package: {
    projectName: "圆弧剧场",
    projectId: "test",
    revisionId: "test-revision",
    digest: "c0".repeat(32),
    bytes: 110772,
    programs: 28,
    loaderBytes: 64000,
  },
  deviceId: "01".repeat(16),
  deviceName: "微雪播放设备",
  connectionEpoch: 2,
  phase: "transferring",
  running: true,
  cancelRequested: false,
  confirmedBytes: 49152,
  receipt: null,
  problem: null,
};
let state: InstallationView = {
  installation: { revision: 1, task: null },
  destination: {
    revision: 1,
    epoch: 2,
    name: "微雪播放设备",
    deviceId: task.deviceId,
    allowed: false,
    reason: "当前设备尚未取得节目安装权限",
  },
};
const unavailable = async () => {
  throw new Error("本组件验收不调用该接口");
};
const host: ApplicationHost = {
  recent: async () => [],
  audio: async () => {throw new Error("此组件不调用音频");},
  audioPrepare: async () => {throw new Error("此组件不调用音频");},
  audioCancel: async () => {},
  kind: "desktop",
  device: unavailable,
  recovery: unavailable,
  buildPackage: unavailable,
  exportPackage: unavailable,
  check: unavailable,
  preview: unavailable,
  previs: unavailable,
  request: unavailable,
  onCloseRequested: async () => () => {},
  startInstallation: unavailable,
  installation: async (request) => {
    if (request.kind === "cancel" && state.installation.task) {
      state.installation.task.cancelRequested = true;
      state.installation.task.phase = "cancelling";
      state.installation.revision++;
    }
    if (request.kind === "forget") {
      state.installation.task = null;
      state.installation.revision++;
    }
    if (request.kind === "resume" && state.installation.task) {
      state.installation.task.running = true;
      state.installation.task.phase = "querying";
      state.installation.revision++;
    }
    return structuredClone(state);
  },
};
function scenario(
  kind: "transfer" | "reconnect" | "installed" | "wrong" | "none",
) {
  state.installation.revision++;
  state.destination.revision++;
  state.destination.allowed = kind !== "none";
  state.destination.reason =
    kind === "none" ? "当前设备尚未取得节目安装权限" : null;
  state.destination.deviceId =
    kind === "wrong" ? "08".repeat(16) : task.deviceId;
  state.destination.epoch = 3;
  state.installation.task = kind === "none" ? null : structuredClone(task);
  if (state.installation.task && (kind === "reconnect" || kind === "wrong")) {
    state.installation.task.phase = "reconnect";
    state.installation.task.running = false;
    state.installation.task.problem = "通信中断，安装结果尚未确认";
  }
  if (state.installation.task && kind === "installed") {
    state.installation.task.phase = "installed";
    state.installation.task.running = false;
    state.installation.task.confirmedBytes = task.package.bytes;
    state.installation.task.cancelRequested = true;
    state.installation.task.receipt = {
      generation: "8",
      digest: task.package.digest,
      bytes: task.package.bytes,
    };
  }
}
function Harness() {
  const controller = useInstallation(host);
  return (
    <div className="workbench">
      <header className="wb-topbar">
        <strong>安装组件隔离验收 · 无硬件</strong>
        <InstallationCenter controller={controller} />
      </header>
      <main style={{ padding: 32 }}>
        {(
          [
            ["none", "无安装权限"],
            ["transfer", "传输中"],
            ["reconnect", "断线待核对"],
            ["wrong", "连接错误设备"],
            ["installed", "提交完成后取消"],
          ] as const
        ).map(([key, label]) => (
          <button key={key} onClick={() => scenario(key)}>
            {label}
          </button>
        ))}
      </main>
    </div>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
