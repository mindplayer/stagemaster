import test from "node:test";
import assert from "node:assert/strict";
import {
  outputAvailable,
  outputLevel,
  outputMasterTarget,
} from "../src/execution-output-target.ts";
import { brightnessRig, deferred, until } from "./execution-brightness-rig.ts";

function rig() {
  const r = brightnessRig();
  r.status.runtime!.catalog.capabilities!.push("outputMaster");
  r.status.runtime!.catalog.output = { uncontrolledFixtures: 0 };
  r.status.runtime!.observation.snapshot!.state.output = {
    percent: 100,
    blackout: false,
  };
  return r;
}
test("总控无需伪造来源，复用单在途最新目标并以整数实际值确认", async () => {
  const r = rig(),
    target = outputMasterTarget(r.status.runtime!),
    gate = deferred();
  r.gate = gate;
  assert.ok(r.controller.beginTarget(target));
  assert.equal(r.view.source, null);
  assert.equal(r.view.key, target.key);
  r.controller.change(target.key, outputLevel(50));
  await until(() => r.mutations.length === 1);
  for (let i = 49; i >= 0; i--) r.controller.change(target.key, outputLevel(i));
  assert.equal(r.controller.begin("source-0"), false);
  r.controller.finish(target.key);
  gate.resolve();
  await until(() => !r.controller.busy);
  assert.deepEqual(
    r.mutations.map(
      ({ request }) => request.kind === "output" && request.action,
    ),
    [
      { kind: "level", percent: 50 },
      { kind: "level", percent: 0 },
    ],
  );
  assert.equal(r.maxActive, 1);
  assert.ok(r.mutations[1].at - r.mutations[0].at >= 50);
  assert.equal(r.view.problem, "");
  assert.equal(
    r.status.runtime!.observation.snapshot!.state.sources[0].level,
    65535,
  );
  assert.ok(r.controller.beginTarget(target));
  r.controller.change(target.key, 12345); // generic normalized input quantizes to this target's precision
  r.controller.finish(target.key);
  await until(() => !r.controller.busy);
  assert.equal(
    r.status.runtime!.observation.snapshot!.state.output!.percent,
    19,
  );
  assert.equal(r.view.problem, "");
});
test("未移动不提交、取消不回滚，熄灯锁存不被亮度命令覆盖", async () => {
  const r = rig(),
    target = outputMasterTarget(r.status.runtime!),
    gate = deferred();
  r.controller.beginTarget(target);
  r.controller.finish(target.key);
  await until(() => !r.controller.busy);
  assert.equal(r.mutations.length, 0);
  r.gate = gate;
  r.controller.beginTarget(target);
  r.controller.change(target.key, outputLevel(37));
  await until(() => r.mutations.length === 1);
  r.status.runtime!.observation.snapshot!.state.output!.blackout = true;
  r.controller.change(target.key, 0);
  r.controller.cancel(target.key);
  gate.resolve();
  await until(() => !r.controller.busy);
  assert.equal(r.mutations.length, 1);
  assert.deepEqual(r.status.runtime!.observation.snapshot!.state.output, {
    percent: 37,
    blackout: true,
  });
});
test("后台、布局、会话、控制权、能力或总控目标变化停止后续发送", async () => {
  for (const mode of [
    "host",
    "layout",
    "session",
    "owner",
    "capability",
    "count",
    "project",
    "hidden",
    "fault",
    "current",
  ]) {
    const r = rig(),
      gate = deferred();
    let current = true;
    const runtime = r.status.runtime!,
      target = outputMasterTarget(runtime, () => current);
    r.gate = gate;
    r.controller.beginTarget(target);
    r.controller.change(target.key, outputLevel(50));
    await until(() => r.mutations.length === 1);
    r.controller.change(target.key, 0);
    if (mode === "host") runtime.hostId = "other";
    if (mode === "layout") runtime.catalog.layout = "other";
    if (mode === "session") runtime.sessionId = "other";
    if (mode === "owner")
      runtime.observation.snapshot!.state.owner!.sessionId = "other";
    if (mode === "capability") runtime.catalog.capabilities = [];
    if (mode === "count") runtime.catalog.output!.uncontrolledFixtures = 1;
    if (mode === "project") runtime.catalog.projectId = "other";
    if (mode === "hidden") r.available = false;
    if (mode === "fault") runtime.observation.snapshot!.state.fault = true;
    if (mode === "current") current = false;
    r.controller.validate();
    gate.resolve();
    await until(() => !r.controller.busy);
    assert.equal(r.mutations.length, 1, mode);
    assert.ok(r.view.problem, mode);
  }
});
test("只认自己 applied 与实际值；拒绝、未知、旧序号或不完整状态不补发", async () => {
  for (const mode of [
    "rejected",
    "unknown",
    "serial",
    "missing",
    "wrongValue",
    "pending",
  ]) {
    const r = rig(),
      target = outputMasterTarget(r.status.runtime!);
    if (mode === "serial")
      r.status.runtime!.record = {
        serial: "9007199254740993",
        status: "complete",
        outcome: { kind: "applied", message: null },
      };
    r.transform = (s) => {
      if (mode === "missing")
        s.runtime!.observation.snapshot!.state.output = undefined;
      if (mode === "wrongValue")
        s.runtime!.observation.snapshot!.state.output!.percent = 1;
      if (mode === "pending") {
        s.runtime!.pending = true;
        s.runtime!.record!.status = "pending";
      }
      if (mode === "rejected" || mode === "unknown")
        s.runtime!.record!.outcome!.kind = mode;
      return s;
    };
    r.controller.beginTarget(target);
    r.controller.change(target.key, outputLevel(50));
    r.controller.finish(target.key);
    await until(() => !r.controller.busy);
    assert.equal(r.mutations.length, 1, mode);
    assert.ok(r.view.problem, mode);
  }
});
test("旧能力、非法百分比或未识别数量拒绝；未识别灯具本身可显示警告", () => {
  const r = rig(),
    runtime = r.status.runtime!;
  runtime.catalog.output!.uncontrolledFixtures = 1;
  assert.ok(outputAvailable(runtime));
  for (const percent of [-1, 101, 0.5, NaN]) {
    runtime.observation.snapshot!.state.output!.percent = percent;
    assert.equal(outputAvailable(runtime), false);
  }
  runtime.observation.snapshot!.state.output!.percent = 100;
  runtime.catalog.output!.uncontrolledFixtures = 99999;
  assert.throws(() => outputMasterTarget(runtime));
  runtime.catalog.output = undefined;
  assert.throws(() => outputMasterTarget(runtime));
});
