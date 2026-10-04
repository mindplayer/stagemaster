import test from "node:test";
import assert from "node:assert/strict";
import {
  commonManualAttributes,
  manualChanges,
  manualPercent,
  type ManualDraft,
} from "../src/execution-manual.ts";
import { manualFixture } from "./execution-manual-fixture.ts";
test("后台固定灯具能力取交集；同键不同色盘不能批量套用", () => {
  const r = manualFixture(),
    f = r.catalog.fixtures!;
  assert.deepEqual(
    commonManualAttributes(f, [f[0].id, f[1].id]).map((a) => a.key),
    ["dimmer", "color-wheel"],
  );
  assert.deepEqual(
    commonManualAttributes(f, [f[0].id, f[2].id]).map((a) => a.key),
    ["dimmer"],
  );
  assert.deepEqual(commonManualAttributes(f, ["missing"]), []);
  assert.deepEqual(commonManualAttributes(f, []), []);
});
test("精确百分比和功能输入只产生对应的原子语义修改，零值与释放不同", () => {
  const r = manualFixture(),
    targets = r.catalog.fixtures!.slice(0, 2).map((f) => f.id);
  const d: ManualDraft = {
    targets,
    attribute: "dimmer",
    value: "37.5",
    functionKey: "",
  };
  const changes = manualChanges(r, "manual", d);
  assert.equal(changes.length, 2);
  assert.deepEqual(changes[0].value, { kind: "normalized", value: 24576 });
  assert.deepEqual(manualChanges(r, "manual", { ...d, value: "0" })[0].value, {
    kind: "normalized",
    value: 0,
  });
  assert.deepEqual(manualChanges(r, "manual", d, true)[0].value, {
    kind: "release",
  });
  assert.deepEqual(
    manualChanges(r, "manual", {
      ...d,
      attribute: "color-wheel",
      functionKey: "red",
      value: "",
    })[0].value,
    { kind: "function", functionKey: "red", position: 0 },
  );
  assert.throws(
    () =>
      manualChanges(r, "manual", {
        ...d,
        attribute: "color-wheel",
        functionKey: "rotate",
        value: "50",
      }),
    /已屏蔽/,
  );
  assert.deepEqual(
    r.observation.snapshot!.state.sources.find((s) => s.id === "manual")!.held,
    [],
  );
});
test("错误目标、功能、输入、条数和完整字节预算均拒绝，不静默分批", () => {
  const r = manualFixture(),
    target = r.catalog.fixtures![0].id;
  const d: ManualDraft = {
    targets: [target],
    attribute: "dimmer",
    value: "10",
    functionKey: "",
  };
  for (const value of ["", " ", "NaN", "Infinity", "-1", "100.01"])
    assert.throws(() => manualPercent(value));
  for (const draft of [
    { ...d, targets: [] },
    { ...d, targets: [target, target] },
    { ...d, targets: ["other"] },
    { ...d, attribute: "missing" },
    { ...d, attribute: "color-wheel", functionKey: "missing" },
  ])
    assert.throws(() => manualChanges(r, "manual", draft));
  assert.throws(() => manualChanges(r, "source-0", d));
  r.catalog.limits!.manualChanges = 1;
  assert.throws(
    () =>
      manualChanges(r, "manual", {
        ...d,
        targets: r.catalog.fixtures!.slice(0, 2).map((f) => f.id),
      }),
    /最多修改/,
  );
  r.catalog.limits!.manualChanges = 512;
  r.catalog.limits!.requestBytes = 512;
  assert.throws(
    () =>
      manualChanges(r, "manual", {
        ...d,
        targets: r.catalog.fixtures!.slice(0, 10).map((f) => f.id),
      }),
    /请求容量/,
  );
  r.catalog.capabilities = [];
  assert.throws(() => manualChanges(r, "manual", d), /未提供/);
});
