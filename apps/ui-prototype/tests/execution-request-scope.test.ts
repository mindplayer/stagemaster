import assert from "node:assert/strict";
import test from "node:test";
import {
  ExecutionRequestScope,
  requestExecution,
  type ExecutionRequestObserver,
} from "../src/execution-request-scope.ts";
import type {
  ExecutionRequest,
  ExecutionStatus,
} from "../src/execution-types.ts";

function deferred<T>() {
  let resolve!: (value: T) => void, reject!: (error: Error) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}
const value = (name: string): ExecutionStatus => ({
  phase: "empty",
  problem: name,
  runtime: null,
});
function fixture() {
  const calls: ExecutionRequest[] = [],
    replies: ReturnType<typeof deferred<ExecutionStatus>>[] = [];
  const scope = new ExecutionRequestScope((command) => {
    calls.push(command);
    const reply = deferred<ExecutionStatus>();
    replies.push(reply);
    return reply.promise;
  });
  scope.display(true);
  const events: {
    status: ExecutionStatus | null;
    errors: string[];
    polling: boolean[];
    working: boolean;
  } = { status: null, errors: [], polling: [], working: false };
  let active = true;
  const observer: ExecutionRequestObserver = {
    current: () => active,
    received: (v, p) => {
      events.status = v;
      events.polling.push(p);
    },
    failed: (e) => events.errors.push(e),
    working: (v) => {
      events.working = v;
    },
  };
  return {
    scope,
    calls,
    replies,
    events,
    observer,
    problem: () => events.status?.problem,
    retire: () => {
      active = false;
      scope.invalidate();
    },
  };
}
test("new port does not wait for old port, and a late success cannot change its view", async () => {
  const a = fixture(),
    b = fixture();
  const old = requestExecution(a.scope, { kind: "snapshot" }, true, a.observer);
  a.retire();
  const current = requestExecution(
    b.scope,
    { kind: "snapshot" },
    true,
    b.observer,
  );
  assert.equal(b.calls.length, 1);
  b.replies[0].resolve(value("乙"));
  await current;
  a.replies[0].resolve(value("甲"));
  assert.equal(await old, undefined);
  assert.equal(a.events.status, null);
  assert.equal(b.events.status?.problem, "乙");
});
test("late failure and cleanup cannot overwrite a new connection's in-flight action", async () => {
  const a = fixture(),
    b = fixture();
  const old = requestExecution(a.scope, { kind: "snapshot" }, true, a.observer);
  a.retire();
  const current = requestExecution(
    b.scope,
    { kind: "release" },
    false,
    b.observer,
  );
  await Promise.resolve();
  assert.equal(b.events.working, true);
  a.replies[0].reject(new Error("旧甲失败"));
  await old;
  assert.deepEqual(a.events.errors, []);
  assert.equal(b.events.working, true);
  b.replies[0].resolve(value("乙动作原回执"));
  await current;
  assert.equal(b.events.working, false);
});
test("hide and show does not revive a read from the old display lifetime", async () => {
  const f = fixture();
  const old = requestExecution(f.scope, { kind: "snapshot" }, true, f.observer);
  f.scope.display(false);
  f.scope.display(true);
  f.replies[0].resolve(value("旧显示"));
  assert.equal(await old, undefined);
  assert.equal(f.events.status, null);
  const current = requestExecution(
    f.scope,
    { kind: "snapshot" },
    true,
    f.observer,
  );
  f.replies[1].resolve(value("新显示"));
  await current;
  assert.equal(f.problem(), "新显示");
});
test("an action waiting for a read must revalidate before it sends", async () => {
  const f = fixture();
  const poll = requestExecution(
    f.scope,
    { kind: "snapshot" },
    true,
    f.observer,
  );
  const action = requestExecution(
    f.scope,
    { kind: "release" },
    false,
    f.observer,
  );
  f.scope.display(false);
  f.scope.display(true);
  f.replies[0].resolve(value("旧读取"));
  await Promise.all([poll, action]);
  assert.equal(f.calls.length, 1);
  assert.equal(f.scope.changing, false);
  assert.equal(f.scope.inflight, null);
});
test("a retained callback after port replacement never submits to the old port", async () => {
  const f = fixture();
  f.retire();
  assert.equal(
    await requestExecution(f.scope, { kind: "release" }, false, f.observer),
    undefined,
  );
  assert.equal(f.calls.length, 0);
});
test("one read remains in flight and an explicit action waits without being swallowed", async () => {
  const f = fixture();
  const read = requestExecution(
    f.scope,
    { kind: "snapshot" },
    true,
    f.observer,
  );
  assert.equal(
    await requestExecution(f.scope, { kind: "snapshot" }, true, f.observer),
    undefined,
  );
  const action = requestExecution(
    f.scope,
    { kind: "release" },
    false,
    f.observer,
  );
  assert.equal(f.calls.length, 1);
  f.replies[0].resolve(value("读取完成"));
  await read;
  await Promise.resolve();
  assert.deepEqual(
    f.calls.map((c) => c.kind),
    ["snapshot", "release"],
  );
  f.replies[1].resolve(value("动作完成"));
  await action;
  assert.equal(f.events.status?.problem, "动作完成");
  assert.equal(f.scope.inflight, null);
  assert.equal(f.scope.changing, false);
});
test("current read failure keeps the known value and reports only its original error", async () => {
  const f = fixture();
  const first = requestExecution(
    f.scope,
    { kind: "snapshot" },
    true,
    f.observer,
  );
  f.replies[0].resolve(value("原状态"));
  await first;
  const failed = requestExecution(
    f.scope,
    { kind: "snapshot" },
    true,
    f.observer,
  );
  f.replies[1].reject(new Error("当前读取失败"));
  await failed;
  assert.equal(f.events.status?.problem, "原状态");
  assert.deepEqual(f.events.errors, ["当前读取失败"]);
  assert.equal(f.calls.length, 2);
});
test("unmount discards a reply but never retries or claims cancellation of the submitted action", async () => {
  const f = fixture();
  const action = requestExecution(
    f.scope,
    { kind: "release" },
    false,
    f.observer,
  );
  await Promise.resolve();
  assert.equal(f.calls.length, 1);
  f.retire();
  f.replies[0].resolve(value("后台已完成"));
  assert.equal(await action, undefined);
  assert.equal(f.calls.length, 1);
  assert.equal(f.events.status, null);
  assert.equal(f.scope.changing, false);
});
test("A to B to A does not revive first A or run two reads through the same port", async () => {
  const first = fixture();
  const old = requestExecution(
    first.scope,
    { kind: "snapshot" },
    true,
    first.observer,
  );
  first.retire();
  const again = new ExecutionRequestScope(first.scope.port, first.scope.lane);
  again.display(true);
  const received: ExecutionStatus[] = [];
  const observed = (): ExecutionStatus[] => received;
  const observer = {
    ...first.observer,
    current: () => true,
    received: (v: ExecutionStatus) => received.push(v),
  };
  assert.equal(
    await requestExecution(again, { kind: "snapshot" }, true, observer),
    undefined,
  );
  assert.equal(first.calls.length, 1);
  first.replies[0].resolve(value("首次甲"));
  await old;
  assert.deepEqual(observed(), []);
  const current = requestExecution(again, { kind: "snapshot" }, true, observer);
  first.replies[1].resolve(value("新甲"));
  await current;
  assert.deepEqual(
    observed().map((v) => v.problem),
    ["新甲"],
  );
});
test("the new lifetime cannot submit a second action while the old port action is unresolved", async () => {
  const first = fixture();
  const old = requestExecution(
    first.scope,
    { kind: "release" },
    false,
    first.observer,
  );
  await Promise.resolve();
  first.retire();
  const again = new ExecutionRequestScope(first.scope.port, first.scope.lane);
  again.display(true);
  const observer = { ...first.observer, current: () => true };
  assert.equal(
    await requestExecution(
      again,
      { kind: "acquire", takeover: false },
      false,
      observer,
    ),
    undefined,
  );
  assert.equal(first.calls.length, 1);
  first.replies[0].resolve(value("原动作完成"));
  await old;
  assert.equal(again.changing, false);
});
