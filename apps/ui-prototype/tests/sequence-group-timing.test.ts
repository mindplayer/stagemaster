import test from "node:test";
import assert from "node:assert/strict";
import type { StepView } from "../src/sequence-types";
import {
  groupTimingDraft,
  groupTimingCommand,
  GroupTimingError,
} from "../src/sequence-group-timing.ts";
const steps: StepView[] = [
  {
    id: "one",
    number: "1",
    name: "一",
    sceneId: "s",
    delayMs: 1500,
    fadeMs: 2500,
    waitMs: null,
  },
  {
    id: "two",
    number: "2",
    name: "二",
    sceneId: "s",
    delayMs: 0,
    fadeMs: 0,
    waitMs: 500,
  },
];
test("混合值明确保留，未勾选字段不会统一成默认值", () => {
  const d = groupTimingDraft(steps);
  assert.equal(d.delay, "");
  assert.equal(d.fade, "");
  assert.equal(d.advance, "mixed");
  assert.equal(groupTimingCommand("seq", d), null);
  const cmd = groupTimingCommand("seq", {
    ...d,
    fadeEnabled: true,
    fade: "1.005",
    delay: "invalid",
  });
  assert.deepEqual(cmd, {
    kind: "editSteps",
    id: "seq",
    stepIds: ["one", "two"],
    operation: { kind: "timing", patch: { fadeMs: 1005 } },
  });
});
test("手动清等待与自动零等待不同，未选定混合推进方式不能提交", () => {
  const d = { ...groupTimingDraft(steps), advanceEnabled: true };
  assert.throws(
    () => groupTimingCommand("seq", d),
    (e: unknown) => e instanceof GroupTimingError && e.field === "advance",
  );
  assert.deepEqual(
    groupTimingCommand("seq", { ...d, advance: "manual", wait: "invalid" }),
    {
      kind: "editSteps",
      id: "seq",
      stepIds: ["one", "two"],
      operation: { kind: "timing", patch: { advance: { kind: "manual" } } },
    },
  );
  assert.deepEqual(
    groupTimingCommand("seq", { ...d, advance: "after", wait: "0" }),
    {
      kind: "editSteps",
      id: "seq",
      stepIds: ["one", "two"],
      operation: {
        kind: "timing",
        patch: { advance: { kind: "after", waitMs: 0 } },
      },
    },
  );
});
test("时间精度和边界拒绝时携带焦点字段，零和一天均可精确表示", () => {
  const d = { ...groupTimingDraft(steps), fadeEnabled: true };
  for (const value of ["", "-1", "1.0001", "86400.001", "Infinity"])
    assert.throws(
      () => groupTimingCommand("seq", { ...d, fade: value }),
      (e: unknown) => e instanceof GroupTimingError && e.field === "fade",
    );
  for (const value of ["0", "86400"])
    assert.ok(groupTimingCommand("seq", { ...d, fade: value }));
  const single = groupTimingDraft([steps[0]]);
  assert.equal(single.delay, "1.5");
  assert.equal(single.fade, "2.5");
  assert.equal(single.advance, "manual");
});
test("草稿冻结身份，筛选不改写修改范围", () => {
  const d = groupTimingDraft(steps);
  steps[0] = { ...steps[0], id: "changed" };
  assert.deepEqual(d.ids, ["one", "two"]);
  assert.throws(() =>
    groupTimingCommand("seq", {
      ...d,
      ids: [],
      delayEnabled: true,
      delay: "0",
    }),
  );
});
