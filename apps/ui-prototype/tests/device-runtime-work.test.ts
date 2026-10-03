import test from "node:test";
import assert from "node:assert/strict";
import { DeviceRuntimeWork } from "../src/device-runtime-work.ts";

function barrier() {
  let resolve!: () => void;
  let reject!: (reason: Error) => void;
  const promise = new Promise<void>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

test("a click during a slow poll executes once using the completed observation", async () => {
  const queue = new DeviceRuntimeWork(), wait = barrier();
  const calls: string[] = [];
  let revision = 1;
  const read = queue.observe(async () => {
    calls.push("read");
    await wait.promise;
    revision = 2;
  });
  const click = queue.command(async () => { calls.push(`pause:${revision}`); });
  await Promise.resolve();
  assert.equal(queue.commandPending, true);
  await queue.command(async () => { calls.push("duplicate"); });
  await queue.observe(async () => { calls.push("second poll"); });
  assert.deepEqual(calls, ["read"]);
  wait.resolve();
  await Promise.all([read, click]);
  assert.deepEqual(calls, ["read", "pause:2"]);
  assert.equal(queue.commandPending, false);
});

test("connection invalidation drops the queued click and future work", async () => {
  const queue = new DeviceRuntimeWork(), wait = barrier();
  let sends = 0;
  const read = queue.observe(() => wait.promise);
  const click = queue.command(async () => { sends++; });
  queue.invalidate();
  wait.resolve();
  await Promise.all([read, click]);
  await queue.command(async () => { sends++; });
  await queue.observe(async () => { sends++; });
  assert.equal(sends, 0);
  const next = new DeviceRuntimeWork();
  await next.command(async () => { sends++; });
  assert.equal(sends, 1);
});

test("read failure rejects the waiting action rather than using stale state", async () => {
  const queue = new DeviceRuntimeWork(), wait = barrier();
  let sends = 0;
  const read = queue.observe(() => wait.promise);
  const click = queue.command(async () => { sends++; });
  const checks = Promise.all([
    assert.rejects(read, /offline/), assert.rejects(click, /offline/),
  ]);
  wait.reject(new Error("offline"));
  await checks;
  assert.equal(sends, 0);
  assert.equal(queue.commandPending, false);
  await queue.observe(async () => {});
  await queue.command(async () => { sends++; });
  assert.equal(sends, 1);
});

test("failed commands release the slot and polling never overlaps commands", async () => {
  const queue = new DeviceRuntimeWork(), wait = barrier();
  let reads = 0;
  const command = queue.command(() => wait.promise);
  await queue.observe(async () => { reads++; });
  assert.equal(reads, 0);
  const check = assert.rejects(command, /rejected/);
  wait.reject(new Error("rejected"));
  await check;
  assert.equal(queue.commandPending, false);
  await queue.observe(async () => { reads++; });
  assert.equal(reads, 1);
});
