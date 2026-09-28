import test from "node:test";
import assert from "node:assert/strict";
import {
  mergeInstallation,
  resumeInstallationReason,
  startInstallationReason,
  installationFinished,
} from "../src/installation-tools.ts";
import type {
  InstallationTask,
  InstallationView,
} from "../src/installation-types.ts";
const task: InstallationTask = {
  id: "one",
  package: {
    projectName: "剧场",
    projectId: "p",
    revisionId: "r",
    digest: "abc",
    bytes: 2048,
    programs: 2,
    loaderBytes: 4096,
  },
  deviceId: "device",
  deviceName: "设备",
  connectionEpoch: 2,
  phase: "reconnect",
  running: false,
  cancelRequested: true,
  confirmedBytes: 1024,
  receipt: null,
  problem: null,
};
function view(): InstallationView {
  return {
    installation: { revision: 8, task },
    destination: {
      revision: 9,
      epoch: 3,
      name: "设备",
      deviceId: "device",
      allowed: true,
      reason: null,
    },
  };
}
test("任务和连接修订独立拒绝迟到状态", () => {
  const old = view();
  const next = view();
  next.installation.revision = 7;
  next.installation.task = null;
  next.destination.revision = 10;
  next.destination.allowed = false;
  const merged = mergeInstallation(old, next);
  assert.equal(merged.installation.task?.id, "one");
  assert.equal(merged.destination.allowed, false);
  const reverse = view();
  reverse.installation.revision = 11;
  reverse.installation.task = null;
  reverse.destination.revision = 8;
  const final = mergeInstallation(merged, reverse);
  assert.equal(final.installation.task, null);
  assert.equal(final.destination.allowed, false);
});
test("无授权与未解决任务都不能开始新安装", () => {
  const state = view();
  assert.ok(startInstallationReason(null, ""));
  assert.ok(startInstallationReason(state, "读取失败"));
  assert.ok(startInstallationReason(state, ""));
  state.installation.task = null;
  state.destination.allowed = false;
  assert.ok(startInstallationReason(state, ""));
  state.destination.allowed = true;
  assert.equal(startInstallationReason(state, ""), null);
});
test("断线恢复必须使用原设备的新连接，业务失败可以同连接重试", () => {
  const state = view();
  assert.equal(resumeInstallationReason(task, state.destination), null);
  state.destination.deviceId = "another";
  assert.match(resumeInstallationReason(task, state.destination)!, /原定/);
  state.destination.deviceId = "device";
  state.destination.epoch = 2;
  assert.match(resumeInstallationReason(task, state.destination)!, /重新连接/);
  assert.equal(
    resumeInstallationReason({ ...task, phase: "failed" }, state.destination),
    null,
  );
  assert.ok(
    resumeInstallationReason({ ...task, running: true }, state.destination),
  );
});
test("只有权威完成状态是终态，传完字节和取消意图都不能冒充完成", () => {
  for (const phase of ["installed", "cancelled", "notStarted"] as const)
    assert.equal(installationFinished(phase), true);
  for (const phase of [
    "verifying",
    "committing",
    "cancelling",
    "failed",
    "reconnect",
  ] as const)
    assert.equal(installationFinished(phase), false);
});
