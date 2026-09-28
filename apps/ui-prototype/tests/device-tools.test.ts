import test from "node:test";
import assert from "node:assert/strict";
import {
  canStartDeviceOperation,
  deviceDeadline,
  deviceError,
  deviceMatches,
  newerDeviceSnapshot,
} from "../src/device-tools.ts";
import type { DeviceSnapshot } from "../src/device-types.ts";
const idle: DeviceSnapshot = {
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
test("older status responses cannot revert an accepted connection command", () => {
  const connected: DeviceSnapshot = {
    ...idle,
    epoch: 2,
    revision: 7,
    phase: "connected",
  };
  assert.equal(newerDeviceSnapshot(connected, idle), connected);
  const fresher = { ...connected, lastReplyAgeMs: 200 };
  assert.equal(newerDeviceSnapshot(connected, fresher), fresher);
  assert.equal(newerDeviceSnapshot(null, idle), idle);
});
test("search is independent of selection and supports native connection handles", () => {
  const candidate = { id: "A3C-72E", name: "舞台设备 StageMaster", rssi: null };
  assert.ok(deviceMatches(candidate, " stagemaster "));
  assert.ok(deviceMatches(candidate, "72e"));
  assert.ok(!deviceMatches(candidate, "另一台"));
});
test("native cleanup and busy phases cannot expose reconnect", () => {
  for (const phase of [
    "preparing",
    "scanning",
    "connecting",
    "connected",
    "stopping",
    "blocked",
  ] as const)
    assert.ok(!canStartDeviceOperation(phase));
  assert.ok(canStartDeviceOperation("fault"));
  assert.ok(canStartDeviceOperation("idle"));
  assert.equal(deviceError({ code: "busy", message: "正在连接" }), "正在连接");
  assert.equal(deviceError(null), "设备状态读取失败，请重试");
});
test("unanswered IPC becomes unavailable and late success cannot restore stale UI", async () => {
  let resolve!: (value: string) => void;
  const delayed = new Promise<string>((done) => {
    resolve = done;
  });
  await assert.rejects(deviceDeadline(delayed, 10), /未及时响应/);
  resolve("late");
  assert.equal(await deviceDeadline(Promise.resolve("current"), 10), "current");
});
