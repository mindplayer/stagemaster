import { test } from "node:test";
import assert from "node:assert/strict";
import { createPrevisResponder } from "../src/previs-responses.ts";
import { PrevisInteractionScope } from "../src/previs-interaction-scope.ts";
import type { PrevisInteractions } from "../src/previs-types.ts";
import type { PrevisTarget } from "../src/previs-objects.ts";
const targets: PrevisTarget[] = [
  { kind: "construction", id: "rig" },
  { kind: "placement", id: "lamp" },
];
const proposal = {
  kind: "objectTranslation",
  requestId: "a".repeat(32),
  generation: 7,
  version: "9007199254740993",
  targets,
  deltaMeters: { x: "1", y: "0", z: "0" },
};
function deferred<T>() {
  let resolve!: (v: T) => void, reject!: (e: Error) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}
function setup() {
  const scope = new PrevisInteractionScope(),
    selectionScope = new PrevisInteractionScope();
  scope.update("stage", true);
  selectionScope.update("stage", true);
  const replies: object[] = [];
  const control = { active: true, connection: 1, time: 0 };
  const host: PrevisInteractions = {
    selectedIds: ["lamp"],
    selectedTargets: targets,
    onSelect: async () => true,
    onPrepareMove: async () => true,
    onTransform: async () => true,
    onTranslation: async () => true,
  };
  const respond = createPrevisResponder({
    active: () => control.active,
    callbacks: () => host,
    state: () => {},
    send: (v) => replies.push(v),
    scope,
    selectionScope,
    connection: () => control.connection,
    now: () => control.time,
  });
  return {
    host,
    control,
    scope,
    selectionScope,
    replies,
    send: (v: unknown) => respond(JSON.stringify(v)),
  };
}
const flush = async () => {
  await Promise.resolve();
  await Promise.resolve();
};
test("混合组绝不降为仅灯具移动，完整对象提案只交给宿主一次", async () => {
  const s = setup();
  let calls = 0;
  s.host.onObjectTranslation = async (p, valid) => {
    assert.deepEqual(p.targets, targets);
    assert.equal(valid(), true);
    calls++;
    return true;
  };
  s.host.onTranslation = async () => {
    throw new Error("不能丢弃构件");
  };
  s.send({
    ...proposal,
    kind: "translation",
    targets: undefined,
    fixtureIds: ["lamp"],
  });
  await flush();
  assert.deepEqual(s.replies, [
    {
      action: "placementResult",
      requestId: proposal.requestId,
      accepted: false,
    },
  ]);
  s.send({ ...proposal, requestId: "b".repeat(32) });
  s.send({ ...proposal, requestId: "b".repeat(32) });
  await flush();
  assert.equal(calls, 1);
  assert.equal(s.replies.length, 2);
  assert.equal((s.replies[1] as { accepted: boolean }).accepted, true);
});
test("等待草稿提交期间，换选择、换连接、换来源、超时或关闭使旧提案失效", async () => {
  for (const change of [
    "selection",
    "connection",
    "scope",
    "deadline",
    "closed",
  ]) {
    const s = setup(),
      pending = deferred<boolean>();
    let valid = () => true;
    s.host.onObjectTranslation = (_p, isActive) => {
      valid = isActive;
      return pending.promise;
    };
    s.send(proposal);
    assert.equal(valid(), true);
    if (change === "selection")
      s.send({ kind: "selectionTargets", targets: [] });
    if (change === "connection") s.control.connection++;
    if (change === "scope") s.scope.invalidate();
    if (change === "deadline") s.control.time = 2500;
    if (change === "closed") s.control.active = false;
    assert.equal(valid(), false);
    pending.resolve(false);
    await flush();
    if (change === "connection" || change === "closed")
      assert.equal(s.replies.length, 0);
  }
});
test("选择失败恢复整个宿主混合组；过时回复不覆盖新连接或新选择", async () => {
  const s = setup(),
    first = deferred<boolean>(),
    second = deferred<boolean>();
  let calls = 0;
  s.host.onSelectTargets = () =>
    ++calls === 1 ? first.promise : second.promise;
  s.send({ kind: "selectionTargets", targets: [] });
  s.send({
    kind: "selectionTargets",
    targets: [{ kind: "construction", id: "other" }],
  });
  first.resolve(false);
  await flush();
  assert.deepEqual(s.replies, []);
  second.resolve(false);
  await flush();
  assert.deepEqual(s.replies, [{ action: "selectTargets", targets }]);
  const old = deferred<boolean>();
  s.host.onSelectTargets = () => old.promise;
  s.send({ kind: "selectionTargets", targets: [] });
  s.control.connection++;
  old.resolve(false);
  await flush();
  assert.equal(s.replies.length, 1);
});
test("宿主异常拒绝提案并恢复选择，未知对象回调不冒充成功", async () => {
  const s = setup();
  s.host.onSelectTargets = async () => {
    throw new Error("草稿未提交");
  };
  s.send({ kind: "selectionTargets", targets: [] });
  await flush();
  assert.deepEqual(s.replies, [{ action: "selectTargets", targets }]);
  s.send(proposal);
  await flush();
  assert.equal((s.replies[1] as { accepted: boolean }).accepted, false);
  s.host.onObjectTranslation = async () => {
    throw new Error("保存拒绝");
  };
  s.send({ ...proposal, requestId: "b".repeat(32) });
  await flush();
  assert.equal((s.replies[2] as { accepted: boolean }).accepted, false);
});
