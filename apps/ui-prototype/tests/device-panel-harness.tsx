// Dev-only UI acceptance fixture. Not imported by any product entry or build.
// It never creates a BLE adapter. Host lifecycle correctness has separate Rust tests.
import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import { DeviceCenter } from "../src/components/devices/DeviceCenter";
import type { ApplicationHost } from "../src/application-host";
import type { DeviceSnapshot } from "../src/device-types";
import "../src/base.css";
import "../src/workbench.css";

let snapshot: DeviceSnapshot = {
  epoch: 0,
  revision: 0,
  phase: "idle",
  candidates: [],
  scanPerformed: false,
  truncated: false,
  selected: null,
  diagnostics: null,
  description: null,
  heartbeatCount: 0,
  roundTripMs: null,
  lastReplyAgeMs: null,
  problem: null,
};
const candidates = [
  { id: "device-alpha", name: "前区播放设备", rssi: -50 },
  { id: "device-long", name: "StageMaster" + "X".repeat(69), rssi: null },
];
const actions: string[] = [];
let notify = () => {};
let unavailable = false;
let delayNext = false;
let delayed: (() => void) | null = null;
function change(value: Partial<DeviceSnapshot>) {
  snapshot = { ...snapshot, ...value, revision: snapshot.revision + 1 };
  notify();
}
const unsupported = async () => {
  throw new Error("此隔离组件不调用该接口");
};
const host: ApplicationHost = {
  recent: async () => [],
  audio: async () => {throw new Error("此组件不调用音频");},
  audioPrepare: async () => {throw new Error("此组件不调用音频");},
  audioCancel: async () => {},
  kind: "desktop",
  output: async () => { throw new Error("此验收未使用预演总控"); },
  device: async (request) => {
    if (request.kind === "status") {
      if (unavailable) throw new Error("测试注入：宿主连接不可用");
      const captured = structuredClone(snapshot);
      if (delayNext) {
        delayNext = false;
        return new Promise((resolve) => {
          delayed = () => resolve(captured);
          notify();
        });
      }
      return captured;
    }
    actions.push(JSON.stringify(request));
    if (request.epoch !== snapshot.epoch)
      throw new Error("测试检测到过期的操作代号");
    if (request.kind === "scan")
      change({
        epoch: snapshot.epoch + 1,
        phase: "scanning",
        candidates: [],
        scanPerformed: true,
        selected: null,
        problem: null,
      });
    if (request.kind === "connect")
      change({
        epoch: snapshot.epoch + 1,
        phase: "connecting",
        selected: candidates.find((c) => c.id === request.id) ?? null,
        problem: null,
      });
    if (request.kind === "cancel")
      change({
        phase: "idle",
        diagnostics: null,
        description: null,
        lastReplyAgeMs: null,
        roundTripMs: null,
      });
    return structuredClone(snapshot);
  },
  request: unsupported,
  recovery: unsupported,
  preview: unsupported,
  previs: unsupported,
  check: unsupported,
  buildPackage: unsupported,
  exportPackage: unsupported,
  installation: unsupported,
  startInstallation: unsupported,
  onCloseRequested: async () => () => {},
};
function Harness() {
  const [, setRevision] = useState(0);
  notify = () => setRevision((value) => value + 1);
  return (
    <div className="workbench">
      <header className="workbench-header">
        <strong>设备组件隔离验收 · 无真实设备</strong>
        <div className="wb-file-actions">
          <DeviceCenter host={host} />
        </div>
      </header>
      <section style={{ padding: 24, maxWidth: 480 }}>
        <h1>界面故障注入</h1>
        <p>仅用于验证组件状态、选择、竞态与布局。</p>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
          <button
            onClick={() =>
              change({ phase: "idle", candidates, scanPerformed: true })
            }
          >
            完成搜索
          </button>
          <button
            onClick={() =>
              change({
                phase: "connected",
                diagnostics: {
                  selfTest: true,
                  outputDisabled: true,
                  uptimeMs: 123450,
                  ticks: 4000,
                  heapUsed: 41020,
                  heapFree: 90052,
                },
                heartbeatCount: 15,
                roundTripMs: 36,
                lastReplyAgeMs: 200,
              })
            }
          >
            确认连接
          </button>
          <button onClick={() => change({
            description: {
              deviceId: "534d4553503332533300b0a73201020304",
              bootId: "0123456789abcdef0123456789abcdef",
              model: 1,
              modelName: "微雪 ESP32-S3-RS485-CAN",
              firmware: "0.2.0",
              declaredFunctions: ["连接诊断"],
              unknownCapabilities: 0,
              authenticationMethod: 0,
              limits: {
                packageVersion: 0, transferVersion: 0, packageBytes: 0,
                programs: 0, universes: 0, messageBytes: 0, chunkBytes: 0,
                slotBytes: 0, loaderBytes: 0, frameMs: 0,
              },
            },
          })}>注入设备描述</button>
          <button onClick={() => change({ description: null })}>模拟旧诊断固件</button>
          <button
            onClick={() => {
              unavailable = true;
            }}
          >
            注入通信故障
          </button>
          <button
            onClick={() => {
              unavailable = false;
            }}
          >
            恢复宿主通信
          </button>
          <button
            onClick={() =>
              change({
                phase: "fault",
                diagnostics: null,
                description: null,
                lastReplyAgeMs: null,
                roundTripMs: null,
                problem: {
                  code: "lost",
                  message: "测试注入：设备连接中断",
                  detail: null,
                },
              })
            }
          >
            注入断线
          </button>
          <button
            onClick={() => {
              delayNext = true;
            }}
          >
            保持下一次状态回复
          </button>
          <button
            onClick={() => {
              delayed?.();
              delayed = null;
              notify();
            }}
          >
            释放旧状态
          </button>
        </div>
        <p aria-label="延迟回复状态">
          {delayed ? "已保留一份状态回复" : "未保留状态回复"}
        </p>
        <h2>组件发出的操作</h2>
        <pre
          aria-label="组件操作记录"
          style={{ whiteSpace: "pre-wrap", overflowWrap: "anywhere" }}
        >
          {actions.join("\n") || "尚无操作"}
        </pre>
      </section>
    </div>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
