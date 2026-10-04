import test from "node:test";
import assert from "node:assert/strict";
import {
  manualBrightnessTarget,
  manualBrightnessReading,
} from "../src/manual-brightness-target.ts";
import { brightnessRig, deferred, until } from "./execution-brightness-rig.ts";

function selection(r: ReturnType<typeof brightnessRig>) {
  return r.status.runtime!.catalog.fixtures!.slice(0, 2).map((f) => f.id);
}
function target(r: ReturnType<typeof brightnessRig>, current = () => true) {
  return manualBrightnessTarget(
    r.status.runtime!,
    "manual",
    selection(r),
    current,
  );
}
function manual(r: ReturnType<typeof brightnessRig>) {
  return r.status.runtime!.observation.snapshot!.state.sources.find(
    (s) => s.id === "manual",
  )!;
}

test("未持有、部分持有和不同值有明确读数，不移动不设置任何灯", async () => {
  const r = brightnessRig(),
    ids = selection(r),
    state = manual(r);
  for (const values of [[], [0], [0, 32000], [0, 0]]) {
    state.held = ids
      .slice(0, values.length)
      .map((fixtureId) => ({ fixtureId, attribute: "dimmer" }));
    state.heldValues = values;
    const frozen = target(r);
    const reading = manualBrightnessReading(r.status.runtime!, "manual", ids)!;
    assert.equal(reading.unheld, 2 - values.length);
    if (values.length === 2 && values[1] !== values[0])
      assert.equal(reading.label, "多个不同值");
    if (!values.length) assert.equal(reading.label, "未持有");
    assert.ok(r.controller.beginTarget(frozen));
    r.controller.finish(frozen.key);
    await until(() => !r.controller.busy);
    assert.equal(r.mutations.length, 0);
  }
});

test("整组亮度共用单飞和最新目标，最终零值持有并与来源推子互斥", async () => {
  const r = brightnessRig(),
    gate = deferred(),
    frozen = target(r);
  r.gate = gate;
  assert.ok(r.controller.beginTarget(frozen));
  r.controller.change(frozen.key, 50000);
  await until(() => r.mutations.length === 1);
  for (let i = 49000; i > 0; i -= 1000) r.controller.change(frozen.key, i);
  r.controller.change(frozen.key, 0);
  r.controller.finish(frozen.key);
  assert.equal(r.controller.begin("manual"), false);
  assert.equal(r.controller.begin("source-0"), false);
  gate.resolve();
  await until(() => !r.controller.busy);
  assert.deepEqual(r.values, [50000, 50000, 0, 0]);
  assert.equal(r.maxActive, 1);
  assert.ok(r.mutations[1].at - r.mutations[0].at >= 50);
  assert.deepEqual(manual(r).heldValues, [0, 0]);
  assert.equal(manual(r).level, 65535);
  assert.equal(manual(r).held!.length, 2);
  assert.ok(r.controller.begin("source-0"));
  r.controller.change("source-0", 12345);
  assert.equal(r.controller.beginTarget(frozen), false);
  r.controller.finish("source-0");
  await until(() => !r.controller.busy);
});

test("冻结选择防止外部数组改变，取消只丢未发目标且旧在途不回滚", async () => {
  const r = brightnessRig(),
    gate = deferred(),
    selected = selection(r);
  const frozen = manualBrightnessTarget(
    r.status.runtime!,
    "manual",
    selected,
    () => true,
  );
  const expected = [...selected];
  selected[0] = "new-target";
  r.gate = gate;
  r.controller.beginTarget(frozen);
  r.controller.change(frozen.key, 40000);
  await until(() => r.mutations.length === 1);
  r.controller.change(frozen.key, 1000);
  r.controller.cancel(frozen.key);
  assert.equal(r.controller.begin("source-0"), false);
  gate.resolve();
  await until(() => !r.controller.busy);
  assert.deepEqual(r.values, [40000, 40000]);
  assert.deepEqual(
    manual(r).held!.map((t) => t.fixtureId),
    expected,
  );
});

test("目标、工程、定义、容量、控制权或连接变化阻止任何后续 Patch", async () => {
  for (const mode of [
    "selection",
    "project",
    "definition",
    "limits",
    "owner",
    "host",
    "layout",
    "session",
    "control",
    "hidden",
    "fault",
  ] as const) {
    const r = brightnessRig(),
      gate = deferred();
    let current = true;
    const frozen = target(r, () => current);
    r.gate = gate;
    r.controller.beginTarget(frozen);
    r.controller.change(frozen.key, 40000);
    await until(() => r.mutations.length === 1);
    r.controller.change(frozen.key, 1000);
    const runtime = r.status.runtime!;
    if (mode === "selection") current = false;
    if (mode === "project") runtime.catalog.projectId = "new";
    if (mode === "definition") runtime.catalog.fixtures![0].attributes = [];
    if (mode === "limits") runtime.catalog.limits!.requestBytes--;
    if (mode === "owner")
      runtime.observation.snapshot!.state.owner!.sessionId = "other";
    if (mode === "host") runtime.hostId = "other";
    if (mode === "layout") runtime.catalog.layout = "other";
    if (mode === "session") runtime.sessionId = "other";
    if (mode === "control") runtime.controlling = false;
    if (mode === "hidden") r.available = false;
    if (mode === "fault") runtime.observation.fault = "fault";
    r.controller.validate();
    gate.resolve();
    await until(() => !r.controller.busy);
    assert.deepEqual(r.values, [40000, 40000], mode);
    assert.ok(r.view.problem, mode);
  }
});

test("属性上限、完整请求预算、非亮度档位和缺少数值能力均不接纳手势", () => {
  for (const mode of [
    "changes",
    "bytes",
    "function",
    "capability",
    "duplicate",
    "missing",
    "source",
  ] as const) {
    const r = brightnessRig(),
      runtime = r.status.runtime!,
      ids = selection(r);
    if (mode === "changes") runtime.catalog.limits!.manualChanges = 1;
    if (mode === "bytes") runtime.catalog.limits!.requestBytes = 1;
    if (mode === "function")
      runtime.catalog.fixtures!.forEach((f) => {
        f.attributes[0].function = f.attributes[1].function;
      });
    if (mode === "capability")
      runtime.catalog.capabilities = ["semanticManualPatch", "manualOwnership"];
    if (mode === "duplicate") ids[1] = ids[0];
    if (mode === "missing") ids[0] = "missing";
    assert.throws(() =>
      manualBrightnessTarget(
        runtime,
        mode === "source" ? "source-0" : "manual",
        ids,
        () => true,
      ),
    );
    assert.equal(r.mutations.length, 0);
  }
});

test("只接受自己的 applied 回执和完整实际值；拒绝、未知、旧序号与超时不重发", async () => {
  for (const mode of [
    "rejected",
    "unknown",
    "missing",
    "pending",
    "serial",
    "partial",
  ] as const) {
    const r = brightnessRig(),
      frozen = target(r);
    if (mode === "serial")
      r.status.runtime!.record = {
        serial: "9007199254740993",
        status: "complete",
        outcome: { kind: "applied", message: null },
      };
    r.transform = (s) => {
      if (mode === "missing") s.runtime!.record = null;
      else if (mode === "pending") {
        s.runtime!.pending = true;
        s.runtime!.record!.status = "pending";
      } else if (mode === "partial") manual(r).heldValues![1] = 22;
      else if (mode !== "serial") s.runtime!.record!.outcome!.kind = mode;
      return s;
    };
    r.controller.beginTarget(frozen);
    r.controller.change(frozen.key, 22000);
    r.controller.finish(frozen.key);
    await until(() => !r.controller.busy);
    assert.equal(r.mutations.length, 1, mode);
    assert.ok(r.view.problem, mode);
  }
});

test("在最终确认中继续同一组，保留最新目标", async () => {
  const r = brightnessRig(),
    frozen = target(r),
    gate = deferred();
  r.gate = gate;
  r.controller.beginTarget(frozen);
  r.controller.change(frozen.key, 20000);
  r.controller.finish(frozen.key);
  await until(() => r.mutations.length === 1);
  assert.ok(r.controller.beginTarget(target(r)));
  r.controller.change(frozen.key, 30000);
  r.controller.finish(frozen.key);
  gate.resolve();
  await until(() => !r.controller.busy);
  assert.deepEqual(r.values, [20000, 20000, 30000, 30000]);
  assert.equal(r.view.problem, "");
});

test("回执已 applied 但整组读数尚未追上时仍等待，不只核对第一台", async () => {
  const r = brightnessRig(),
    frozen = target(r);
  let reads = 0;
  r.transform = (s) => {
    if (s.runtime!.record!.serial === "1") manual(r).heldValues![1] = 11;
    return s;
  };
  r.afterSnapshot = () => {
    if (r.status.runtime!.record?.serial !== "1") return;
    reads++;
    assert.equal(r.mutations.length, 1);
    if (reads === 3) manual(r).heldValues![1] = 20000;
  };
  r.controller.beginTarget(frozen);
  r.controller.change(frozen.key, 20000);
  await until(() => r.mutations.length === 1);
  r.controller.change(frozen.key, 40000);
  r.controller.finish(frozen.key);
  await until(() => !r.controller.busy);
  // Three reads observe the whole group, then one fresh revision read precedes the next Patch.
  assert.equal(reads, 4);
  assert.deepEqual(r.values, [20000, 20000, 40000, 40000]);
  assert.equal(r.view.problem, "");
});
