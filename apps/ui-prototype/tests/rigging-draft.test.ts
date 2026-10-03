import test from "node:test";
import assert from "node:assert/strict";
import { stageProject } from "./stage-organization-fixture.ts";
import {
  beginRigging,
  riggingCommand,
} from "../src/components/stage/rigging-session.ts";
import {
  RiggingPreviewQueue,
  type RiggingPreviewResult,
} from "../src/components/stage/rigging-preview-queue.ts";
import type { RiggingProjection } from "../src/rigging-preview-types.ts";

test("挂接草稿保留既有位置，显式均布保留顺序和精确输入，源对象不变", () => {
  const p = stageProject(),
    before = structuredClone(p);
  const s = beginRigging(p, ["audience", "front"], "rig");
  assert.equal(s.dirty, false);
  assert.equal(riggingCommand(p, s).layout, null);
  s.layout = true;
  s.draft.startMarginMeters = "1.2500";
  const c = riggingCommand(p, s);
  assert.deepEqual(c.fixtureIds, ["audience", "front"]);
  assert.equal(c.layout?.startMarginMeters, "1.25");
  assert.deepEqual(p, before);
  p.stage.placements.pop();
  assert.equal(beginRigging(p, ["loose"], "rig").layout, true);
});
test("挂接拒绝无效成员、数字、余量和过期来源，不丢失草稿", () => {
  const p = stageProject(),
    s = beginRigging(p, ["front", "audience"], "rig");
  for (const ids of [[], ["front", "front"], ["deleted"]])
    assert.throws(() => riggingCommand(p, { ...s, ids }));
  assert.throws(() => riggingCommand(p, { ...s, rig: "deleted" }), /支撑体/);
  for (const value of ["", " ", "NaN", "-0.1", "1001"])
    assert.throws(() =>
      riggingCommand(p, {
        ...s,
        layout: true,
        draft: { ...s.draft, dropMeters: value },
      }),
    );
  assert.throws(
    () =>
      riggingCommand(p, {
        ...s,
        layout: true,
        draft: { ...s.draft, endMarginMeters: "8" },
      }),
    /余量/,
  );
  for (const change of [
    (n: typeof p) => n.stage.constructions.pop(),
    (n: typeof p) =>
      (n.stage.editLocks = [{ kind: "placement", targetId: "front" }]),
    (n: typeof p) => n.fixtures.pop(),
    (n: typeof p) => (n.id = "other"),
  ]) {
    const next = structuredClone(p);
    change(next);
    assert.throws(() => riggingCommand(next, s), /已变化/);
  }
  assert.equal(s.dirty, false);
});
const settle = () => new Promise<void>((resolve) => setImmediate(resolve));
function queued() {
  const calls: {
    resolve(value: RiggingProjection): void;
    reject(error: Error): void;
    generation: number;
  }[] = [];
  const results: RiggingPreviewResult[] = [];
  const queue = new RiggingPreviewQueue(
    (generation) =>
      new Promise((resolve, reject) =>
        calls.push({ resolve, reject, generation }),
      ),
    (value) => results.push(value),
  );
  const p = stageProject(),
    command = riggingCommand(p, beginRigging(p, ["front"], "rig"));
  return { calls, results, queue, command };
}
test("快速改参最多一个在途和最新待处理，旧结果不能覆盖新位置", async () => {
  const { queue, calls, command, results } = queued();
  queue.submit("first", 1, command);
  queue.submit("second", 2, command);
  queue.submit("last", 3, command);
  assert.equal(calls.length, 1);
  calls[0].resolve({ generation: 1, placements: [], changed: true });
  await settle();
  assert.equal(calls.length, 2);
  assert.equal(calls[1].generation, 3);
  assert.equal(results.length, 0);
  calls[1].resolve({ generation: 3, placements: [], changed: false });
  await settle();
  assert.deepEqual(
    results.map((r) => r.key),
    ["last"],
  );
});
test("取消与重新打开不会并发或应用旧错误；错误代次拒绝", async () => {
  const { queue, calls, command, results } = queued();
  queue.submit("closed", 1, command);
  queue.cancel();
  queue.submit("reopen", 2, command);
  calls[0].reject(new Error("旧错误"));
  await settle();
  assert.equal(results.length, 0);
  assert.equal(calls.length, 2);
  calls[1].resolve({ generation: 1, placements: [], changed: true });
  await settle();
  assert.match(results[0].error!, /代次/);
  queue.submit("cancelled", 3, command);
  queue.cancel();
  calls[2].resolve({ generation: 3, placements: [], changed: true });
  await settle();
  assert.equal(results.length, 1);
});
