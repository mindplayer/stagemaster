// Isolated UI fixture. Never imported by production and never opens a hardware adapter.
import type { DeviceSnapshot } from "../src/device-types";
import type {
  DeviceRunPort,
  DeviceRunReply,
  DeviceRunState,
  DeviceRunView,
} from "../src/device-runtime-types";
const key = { kind: "sequence" as const, id: "1".repeat(32) };
const steps = [1, 2].map((n) => ({
  id: String(n).repeat(32),
  number: String(n),
  name: n === 1 ? "开场 · 暖色定点" : "律动 · 蓝色扫动",
}));
export function fixture() {
  const actions: string[] = [];
  let changed = () => {},
    online = false,
    serial = 0n,
    revision = 9007199254740993n;
  let clock = 0n,
    fail = false,
    uncertain = false,
    readFailure = false,
    release: (() => void) | null = null;
  let last: DeviceRunReply | null = null;
  let s: DeviceRunState = {
    mode: "operation",
    package: "a".repeat(64),
    selected: null,
    loaded: null,
    status: null,
    instance: null,
    step: null,
    elapsedMs: "0",
    owner: null,
  };
  let device: DeviceSnapshot = {
    revision: 0,
    epoch: 1,
    phase: "idle",
    candidates: [{ id: "test-device", name: "前区节目设备", rssi: -42 }],
    scanPerformed: true,
    truncated: false,
    selected: null,
    diagnostics: null,
    description: {
      deviceId: "d".repeat(32),
      bootId: "b".repeat(32),
      model: 1,
      modelName: "隔离运行验收设备",
      firmware: "0.2.0",
      declaredFunctions: ["连接诊断", "加密节目安装", "节目运行控制"],
      unknownCapabilities: 0,
      authenticationMethod: 1,
      limits: {
        packageVersion: 1,
        executionSemantics: 2,
        transferVersion: 1,
        packageBytes: 1048576,
        programs: 128,
        universes: 1,
        messageBytes: 1280,
        chunkBytes: 512,
        slotBytes: 1048576,
        loaderBytes: 2097152,
        frameMs: 25,
      },
    },
    heartbeatCount: 0,
    roundTripMs: null,
    lastReplyAgeMs: null,
    problem: null,
  };
  const view = (reply: DeviceRunReply | null = null): DeviceRunView => ({
    epoch: device.epoch,
    connectionEpoch: device.epoch,
    peer: online
      ? {
          device: "d".repeat(32),
          boot: "b".repeat(32),
          session: String(device.epoch),
          control: true,
          installation: true,
        }
      : null,
    pending: uncertain,
    lastResponse: structuredClone(last),
    reply,
  });
  const port: DeviceRunPort = async (request) => {
    if (request.epoch !== device.epoch) throw new Error("过期连接");
    if (request.kind === "connect") {
      online = true;
      device = {
        ...device,
        revision: device.revision + 1,
        epoch: device.epoch + 1,
        phase: "connected",
        selected: device.candidates[0],
        diagnostics: {
          selfTest: true,
          outputDisabled: true,
          uptimeMs: 2000,
          ticks: 80,
          heapUsed: 48000,
          heapFree: 80000,
        },
      };
      last = null;
      uncertain = false;
      s.owner = null;
      changed();
      return view();
    }
    if (request.kind === "snapshot") return view();
    if (!online) throw new Error("连接已中断");
    if (request.kind === "refresh" && readFailure)
      throw new Error("测试：状态读取失败，请稍后刷新");
    clock += 1000n;
    let body: DeviceRunReply["body"] = { kind: "state", state: s, error: null };
    if (request.kind === "apply") {
      actions.push(request.action.kind);
      changed();
      if (fail) {
        fail = false;
        uncertain = true;
        online = false;
        device.phase = "fault";
        device.revision++;
        changed();
        throw new Error("测试：发送后断线，执行结果未确认");
      }
      if (request.revision !== String(revision))
        body = { kind: "state", state: s, error: "操作上下文已变化，请刷新" };
      else {
        const a = request.action;
        if (a.kind === "acquire")
          s.owner = {
            lease: "9007199254740994",
            expiresMs: String(clock + 60000n),
          };
        else if (a.kind === "renew")
          s.owner!.expiresMs = String(clock + 60000n);
        else if (a.kind === "release") s.owner = null;
        else if (a.kind === "select") s.selected = a.program;
        else if (a.kind === "load") {
          s.loaded = s.selected;
          s.status = "idle";
        } else if (a.kind === "start") {
          s.instance = "1";
          s.step = a.step;
          s.status = "running";
        } else if (a.kind === "pause") s.status = "paused";
        else if (a.kind === "resume") s.status = "running";
        else if (a.kind === "next") s.step = steps[1].id;
        else if (a.kind === "stop") {
          s.status = "idle";
          s.instance = null;
          s.step = null;
        } else if (a.kind === "beginMaintenance") {
          s.mode = "maintenance";
          s.loaded = null;
          s.instance = null;
          s.status = null;
          s.step = null;
        } else if (
          a.kind === "finishMaintenance" ||
          a.kind === "cancelMaintenance"
        )
          s.mode = "operation";
        revision++;
      }
    } else if (request.kind === "catalog") {
      if (release)
        await new Promise<void>((resolve) => {
          release = resolve;
          changed();
        });
      body = {
        kind: "program",
        index: request.index,
        program:
          request.index === 0
            ? { key, name: "开场灯光 · 完整场景列表", loaderBytes: 4096 }
            : null,
      };
    } else if (request.kind === "step")
      body = {
        kind: "step",
        index: request.index,
        step: steps[request.index] ?? null,
      };
    if (s.status === "running") s.elapsedMs = String(clock);
    const reply: DeviceRunReply = {
      id: String(++serial),
      boot: "b".repeat(32),
      revision: String(revision),
      observedMs: String(clock),
      programCount: 1,
      stepCount: s.loaded ? steps.length : 0,
      body: structuredClone(body),
    };
    last = reply;
    return view(reply);
  };
  return {
    port,
    actions,
    subscribe: (fn: () => void) => {
      changed = fn;
    },
    device: async (request: import("../src/device-types").DeviceRequest) => {
      if (request.kind === "cancel") {
        online = false;
        s.owner = null;
        device.phase = "idle";
        device.diagnostics = null;
        device.revision++;
      }
      if (request.kind === "connect") {
        device.phase = "connected";
        device.selected = device.candidates[0];
        device.revision++;
      }
      return structuredClone(device);
    },
    fail: () => {
      fail = true;
    },
    failRead: () => {
      readFailure = true;
    },
    recoverRead: () => {
      readFailure = false;
    },
    hold: () => {
      release = () => {};
    },
    release: () => {
      release?.();
      release = null;
    },
    foreign: () => {
      s.owner = { lease: "other", expiresMs: String(clock + 60000n) };
      revision++;
    },
  };
}
