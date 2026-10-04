import test from "node:test";
import assert from "node:assert/strict";
import {
  batchReady,
  reviewBatch,
  validBatchReview,
  selectPrograms,
  batchEffect,
} from "../src/execution-batch.ts";
import { boardFixture } from "./execution-board-fixture.ts";
import { executionCommands } from "../src/components/execution/executionCommands.ts";
import {
  batchReply,
  batchReceiptText,
} from "../src/execution-batch-receipt.ts";

function ready() {
  const r = boardFixture();
  r.catalog.capabilities = ["sourceBatch"];
  r.observation.snapshot!.state.owner = {
    sessionId: "session",
    expiresMs: "60000",
  };
  return r;
}
test("正式请求装配保留冻结修订；忙／连续手势／旧后台不提交，不逐条 fallback", async () => {
  const r = ready();
  const review = reviewBatch(r, ["source-0", "source-1"], "pause");
  const requests: unknown[] = [];
  let liveBusy = false;
  const request = async (v: unknown) => {
    requests.push(v);
    return { phase: "connected" as const, problem: null, runtime: r };
  };
  const commands = executionCommands(r, false, () => liveBusy, request);
  await commands.batch({ ...review.request, hostId: "other" });
  await commands.batch({ ...review.request, revision: "0" });
  liveBusy = true;
  await commands.batch(review.request);
  await executionCommands(r, true, () => false, request).batch(review.request);
  assert.equal(requests.length, 0);
  liveBusy = false;
  await commands.batch(review.request);
  assert.deepEqual(requests, [review.request]);
});
test("选择和搜索只改变 UI，有序加选去重，音乐／手动不可加入，隐藏选择保留", () => {
  const r = ready();
  const before = structuredClone(r);
  const selected = selectPrograms(
    [],
    r.catalog.sources,
    ["source-1", "source-0", "music", "manual"],
    "add",
  );
  assert.deepEqual(selected, ["source-1", "source-0"]);
  assert.deepEqual(
    selectPrograms(
      selected,
      r.catalog.sources,
      ["source-1", "source-2"],
      "add",
    ),
    ["source-1", "source-0", "source-2"],
  );
  assert.deepEqual(
    selectPrograms(selected, r.catalog.sources, ["source-1"], "toggle"),
    ["source-0"],
  );
  const review = reviewBatch(r, selected, "pause");
  assert.deepEqual(review.request.sources, selected);
  assert.equal(review.members.length, 2);
  selected.push("source-2");
  assert.equal(review.request.sources.length, 2);
  assert.equal(validBatchReview(review, r, selected), false);
  assert.deepEqual(r, before);
});
test("冻结后台／布局／工程／会话／修订／选择和名称；进度自然变化不替换名单", () => {
  const r = ready(),
    ids = ["source-0", "source-1"];
  const review = reviewBatch(r, ids, "pause");
  assert(validBatchReview(review, r, ids));
  const variants = [
    (v: typeof r) => (v.hostId = "other"),
    (v: typeof r) => (v.catalog.layout = "other"),
    (v: typeof r) => (v.catalog.projectId = "other"),
    (v: typeof r) => (v.sessionId = "other"),
    (v: typeof r) => (v.observation.snapshot!.state.revision = "2"),
    (v: typeof r) => (v.catalog.sources[0].name = "已变化"),
    (v: typeof r) => (v.controlling = false),
    (v: typeof r) => (v.pending = true),
    (v: typeof r) => (v.observation.snapshot!.state.owner = null),
    (v: typeof r) => (v.observation.phase = "faulted"),
    (v: typeof r) => (v.observation.fault = "故障"),
    (v: typeof r) => (v.observation.snapshot!.state.fault = true),
  ];
  for (const change of variants) {
    const v = structuredClone(r);
    change(v);
    assert(!validBatchReview(review, v, ids));
  }
  r.observation.snapshot!.cycles = "200";
  r.observation.snapshot!.state.sources[0].status = "Finished";
  assert(validBatchReview(review, r, ids));
  assert.deepEqual(
    review.members.map((m) => m.status),
    ["Running", "Running"],
  );
});
test("混合状态不隐式启动；作用数按真实状态，未知／类型／容量／重复整组拒绝", () => {
  const r = ready();
  assert.equal(
    batchEffect(r, ["source-0", "source-2", "source-3", "source-4"], "pause"),
    1,
  );
  assert.equal(
    batchEffect(r, ["source-0", "source-2", "source-3", "source-4"], "resume"),
    1,
  );
  assert.equal(
    batchEffect(r, ["source-0", "source-2", "source-3", "source-4"], "stop"),
    3,
  );
  for (const ids of [
    [],
    ["source-0", "source-0"],
    ["source-0", "manual"],
    ["source-0", "music"],
    ["source-0", "missing"],
    Array(65).fill("source-0"),
  ])
    assert.throws(() => reviewBatch(r, ids, "stop"));
  assert.throws(() => reviewBatch(r, ["source-4"], "resume"));
  r.observation.snapshot!.state.sources[1].status = "Bogus";
  assert.throws(
    () => reviewBatch(r, ["source-0", "source-1"], "stop"),
    /状态未知/,
  );
  const unknown = ready();
  unknown.catalog.capabilities = [];
  assert.equal(batchReady(unknown), false);
  assert.throws(() => reviewBatch(unknown, ["source-0"], "pause"));
});
test("只认同一后台／布局／工程／会话的新回执，待确认与拒绝不能报生效", () => {
  const r = ready(),
    review = reviewBatch(r, ["source-0"], "pause");
  const response = {
    phase: "connected" as const,
    problem: null,
    runtime: structuredClone(r),
  };
  response.runtime.record = {
    serial: "2",
    status: "complete",
    outcome: { kind: "applied", message: null },
  };
  assert.equal(batchReply(review, "1", response)?.serial, "2");
  assert.equal(batchReply(review, "2", response), null);
  for (const edit of [
    (v: typeof r) => (v.hostId = "other"),
    (v: typeof r) => (v.catalog.layout = "other"),
    (v: typeof r) => (v.catalog.projectId = "other"),
    (v: typeof r) => (v.sessionId = "other"),
  ]) {
    const copy = structuredClone(response);
    edit(copy.runtime);
    assert.equal(batchReply(review, "1", copy), null);
  }
  assert.equal(batchReply(review, "1", undefined), null);
  assert.match(
    batchReceiptText({ serial: "2", status: "pending", outcome: null }),
    /待确认/,
  );
  assert.match(
    batchReceiptText({
      serial: "2",
      status: "complete",
      outcome: { kind: "unknown", message: null },
    }),
    /未确认生效/,
  );
  assert.match(
    batchReceiptText({
      serial: "2",
      status: "complete",
      outcome: { kind: "rejected", message: "版本变化" },
    }),
    /版本变化/,
  );
});
