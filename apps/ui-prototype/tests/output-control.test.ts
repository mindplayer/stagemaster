import test from "node:test";
import assert from "node:assert/strict";
import { OutputControlQueue } from "../src/components/output/output-control-queue.ts";
import type { OutputControlView } from "../src/components/output/output-control-queue.ts";
import type {
  OutputControlRequest,
  OutputControlSnapshot,
} from "../src/output-control-types.ts";
const initial: OutputControlSnapshot = {
  epoch: 1,
  serial: 0,
  percent: 100,
  blackout: false,
  uncontrolledFixtures: 0,
};
const tick = () => new Promise((resolve) => setImmediate(resolve));
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (value: Error) => void;
  const promise = new Promise<T>((a, b) => {
    resolve = a;
    reject = b;
  });
  return { promise, resolve, reject };
}
test("coalesces continuous level input and preserves blackout while a receipt is pending", async () => {
  const calls: OutputControlRequest[] = [];
  const first = deferred<OutputControlSnapshot>();
  let current = { ...initial };
  let view: OutputControlView | undefined;
  const queue = new OutputControlQueue(
    async (request) => {
      calls.push(request);
      if (request.kind === "snapshot") return current;
      if (request.serial === 1) return first.promise;
      current = { ...current, ...request };
      return current;
    },
    (value) => {
      view = value;
    },
  );
  await queue.poll();
  queue.set({ percent: 60 });
  for (let i = 59; i >= 40; i--) queue.set({ percent: i });
  queue.set({ blackout: true });
  await queue.poll();
  assert.equal(calls.length, 2);
  assert.equal(view?.desired?.percent, 40);
  assert.equal(view?.desired?.blackout, true);
  first.resolve({ ...initial, serial: 1, percent: 60 });
  await tick();
  assert.equal(calls.length, 3);
  assert.deepEqual(calls[2], {
    kind: "set",
    epoch: 1,
    serial: 2,
    percent: 40,
    blackout: true,
  });
  assert.equal(view?.actual?.blackout, true);
  queue.set({ blackout: false });
  await tick();
  assert.equal(view?.actual?.percent, 40);
  assert.equal(view?.working, false);
  queue.dispose();
});
test("a stale poll cannot roll back newer intent", async () => {
  const stale = deferred<OutputControlSnapshot>();
  let reads = 0;
  let view: OutputControlView | undefined;
  const queue = new OutputControlQueue(
    async (request) => {
      if (request.kind === "snapshot")
        return ++reads === 1 ? initial : stale.promise;
      return { ...initial, ...request };
    },
    (v) => {
      view = v;
    },
  );
  await queue.poll();
  const poll = queue.poll();
  queue.set({ percent: 20 });
  await tick();
  stale.resolve(initial);
  await poll;
  assert.equal(view?.actual?.percent, 20);
  queue.dispose();
});
test("failed uncertain write reconciles new project and does not replay queued darkness", async () => {
  const write = deferred<OutputControlSnapshot>();
  const calls: OutputControlRequest[] = [];
  let reads = 0;
  let view: OutputControlView | undefined;
  const queue = new OutputControlQueue(
    async (request) => {
      calls.push(request);
      if (request.kind === "snapshot") return { ...initial, epoch: ++reads };
      return write.promise;
    },
    (v) => {
      view = v;
    },
  );
  await queue.poll();
  queue.set({ percent: 30 });
  queue.set({ blackout: true });
  write.reject(new Error("工程已更换"));
  await tick();
  assert.equal(calls.filter((c) => c.kind === "set").length, 1);
  assert.equal(view?.actual?.epoch, 2);
  assert.equal(view?.desired?.blackout, false);
  assert.match(view?.error ?? "", /工程已更换/);
  queue.dispose();
  queue.set({ percent: 0 });
  await queue.poll();
  assert.equal(calls.length, 3);
});
