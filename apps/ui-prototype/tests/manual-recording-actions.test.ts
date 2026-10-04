import test from "node:test";
import assert from "node:assert/strict";
import { manualRecordingActions } from "../src/manual-recording-actions.ts";
test("cancelled draft commit does not send a recording or report success", async () => {
  let calls = 0;
  const a = manualRecordingActions(
    async () => false,
    async () => {
      calls++;
    },
    () => {
      calls++;
    },
  );
  await assert.rejects(a.onMerge(2, "ticket", true), /未完成/);
  await assert.rejects(a.onRecord(2, "ticket", "场景"), /未完成/);
  assert.equal(calls, 0);
});
test("host rejection remains visible and is not converted to a successful merge", async () => {
  const notices: string[] = [];
  const a = manualRecordingActions(
    async (work) => {
      try {
        await work();
        return true;
      } catch {
        return false;
      }
    },
    async () => {
      throw Error("目标场景已变化");
    },
    (s) => notices.push(s),
  );
  await assert.rejects(a.onMerge(2, "ticket", true), /目标场景已变化/);
  assert.deepEqual(notices, []);
});
test("successful merge submits only the reviewed ticket and reports retained runtime", async () => {
  const calls: unknown[] = [];
  const notices: string[] = [];
  const a = manualRecordingActions(
    async (work) => {
      await work();
      return true;
    },
    async (r) => {
      calls.push(r);
    },
    (s) => notices.push(s),
  );
  await a.onMerge(3, "fixed", true);
  assert.deepEqual(calls, [
    { kind: "mergeManualScene", generation: 3, token: "fixed" },
  ]);
  assert.match(notices[0], /后台运行版本保持/);
});

test("unchanged review never promises a new undo step", async () => {
  const notices: string[] = [];
  const a = manualRecordingActions(
    async (work) => {
      await work();
      return true;
    },
    async () => {},
    (s) => notices.push(s),
  );
  await a.onMerge(3, "unchanged", false);
  assert.match(notices[0], /场景未修改/);
  assert.doesNotMatch(notices[0], /撤销/);
});
