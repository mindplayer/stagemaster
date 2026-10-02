import test from "node:test";
import assert from "node:assert/strict";
import type { AudioCommand } from "../src/audio-types.ts";
import { AudioActionQueue } from "../src/audio-action-queue.ts";
import { runAudioCommands } from "../src/audio-command-sequence.ts";
test("停止绕过尚未完成的定位，取消旧批次的后续播放并等停止完成后接受新命令", async () => {
  const queue = new AudioActionQueue(),
    slow = deferred(),
    stop = deferred();
  const seen: string[] = [];
  const first = queue.enqueue(
    [{ kind: "seek", positionMs: 100 }, { kind: "play" }],
    "one",
    (commands, current) =>
      runAudioCommands(commands, current, async (command) => {
        seen.push(command.kind);
        return slow.promise;
      }),
  );
  const discarded = queue.enqueue(seek(200), "one", async () => {
    throw new Error("不应执行");
  });
  const priority = queue.interrupt([{ kind: "stop" }], async () => {
    seen.push("stop");
    return stop.promise;
  });
  const next = queue.enqueue([{ kind: "play" }], "one", async () => {
    seen.push("new-play");
    return true;
  });
  assert.equal(await discarded, false);
  assert.deepEqual(seen, ["seek", "stop"]);
  slow.resolve(true);
  assert.equal(await first, false);
  assert.equal(queue.observation(), null);
  assert.deepEqual(seen, ["seek", "stop"]);
  stop.resolve(true);
  assert.equal(await priority, true);
  assert.equal(await next, true);
  assert.deepEqual(seen, ["seek", "stop", "new-play"]);
});
function deferred() {
  let resolve!: (value: boolean) => void;
  const promise = new Promise<boolean>((yes) => {
    resolve = yes;
  });
  return { promise, resolve };
}
const seek = (positionMs: number): AudioCommand[] => [
  { kind: "seek", positionMs },
];
test("快速定位最多保留活动请求和最后一个相邻目标，共享回执且不采信旧轮询", async () => {
  const queue = new AudioActionQueue(),
    gate = deferred();
  const observation = queue.observation()!;
  const seen: AudioCommand[][] = [];
  const first = queue.enqueue(seek(1760), "one", async (commands) => {
    seen.push(commands);
    return gate.promise;
  });
  let second: Promise<boolean> | undefined;
  for (let i = 1; i <= 32; i++) {
    const result = queue.enqueue(
      seek(1760 + i * 10),
      "one",
      async (commands) => {
        seen.push(commands);
        return true;
      },
    );
    if (second) assert.equal(result, second);
    else second = result;
  }
  assert.equal(queue.observation(), null);
  gate.resolve(true);
  assert.equal(await first, true);
  assert.equal(await second, true);
  assert.deepEqual(seen, [seek(1760), seek(2080)]);
  assert.equal(queue.acceptsObservation(observation), false);
  assert.equal(queue.acceptsObservation(queue.observation()!), true);
});
test("播放暂停屏障与上下文阻止跨越合并，定位后播放仍作为同一序列", async () => {
  const queue = new AudioActionQueue(),
    gate = deferred();
  const seen: AudioCommand[][] = [];
  const first = queue.enqueue(seek(0), "one", async () => gate.promise);
  const run = async (commands: AudioCommand[]) => {
    seen.push(commands);
    return true;
  };
  const tasks = [
    queue.enqueue(seek(10), "one", run),
    queue.enqueue([{ kind: "pause" }], "one", run),
    queue.enqueue(seek(20), "one", run),
    queue.enqueue(seek(30), "two", run),
    queue.enqueue(
      [{ kind: "seek", positionMs: 40 }, { kind: "play" }],
      "two",
      run,
    ),
    queue.enqueue(seek(50), "two", run),
    queue.enqueue([{ kind: "stop" }], "two", run),
  ];
  gate.resolve(true);
  await first;
  await Promise.all(tasks);
  assert.deepEqual(seen, [
    seek(10),
    [{ kind: "pause" }],
    seek(20),
    seek(30),
    [{ kind: "seek", positionMs: 40 }, { kind: "play" }],
    seek(50),
    [{ kind: "stop" }],
  ]);
});
test("切换媒体清掉排队意图，活动宿主请求结束前仍不能采信观察", async () => {
  const queue = new AudioActionQueue(),
    gate = deferred();
  let called = false;
  const first = queue.enqueue(seek(0), "old", async () => gate.promise);
  const discarded = queue.enqueue(seek(100), "old", async () => {
    called = true;
    return true;
  });
  queue.invalidate();
  assert.equal(await discarded, false);
  assert.equal(queue.observation(), null);
  gate.resolve(false);
  await first;
  assert.equal(called, false);
  assert.notEqual(queue.observation(), null);
});
test("待执行批次有上限，错误不损坏后续队列；合并音量不制造无限命令", async () => {
  const queue = new AudioActionQueue(),
    gate = deferred();
  const first = queue.enqueue(seek(0), "one", async () => gate.promise);
  const pending = Array.from({ length: 32 }, () =>
    queue.enqueue([{ kind: "pause" }], "one", async () => true),
  );
  await assert.rejects(
    queue.enqueue([{ kind: "stop" }], "one", async () => true),
    /过密/,
  );
  gate.resolve(true);
  await first;
  await Promise.all(pending);
  await assert.rejects(
    queue.enqueue(seek(0), "one", async () => {
      throw new Error("失败");
    }),
    /失败/,
  );
  assert.equal(
    await queue.enqueue([{ kind: "stop" }], "one", async () => true),
    true,
  );
  const wait = deferred(),
    seen: AudioCommand[][] = [];
  const active = queue.enqueue(
    [{ kind: "volume", percent: 5 }],
    "one",
    async () => wait.promise,
  );
  const a = queue.enqueue(
    [{ kind: "volume", percent: 10 }],
    "one",
    async (commands) => {
      seen.push(commands);
      return true;
    },
  );
  const b = queue.enqueue(
    [{ kind: "volume", percent: 20 }],
    "one",
    async (commands) => {
      seen.push(commands);
      return true;
    },
  );
  assert.equal(a, b);
  wait.resolve(true);
  await active;
  await b;
  assert.deepEqual(seen, [[{ kind: "volume", percent: 20 }]]);
});
test("定位失败不得继续原子预演序列的播放，独立停止仍能执行", async () => {
  const queue = new AudioActionQueue();
  const seen: string[] = [];
  const result = await queue.enqueue(
    [{ kind: "seek", positionMs: 100 }, { kind: "play" }],
    "one",
    (commands) =>
      runAudioCommands(
        commands,
        () => true,
        async (value) => {
          seen.push(value.kind);
          return false;
        },
      ),
  );
  assert.equal(result, false);
  assert.deepEqual(seen, ["seek"]);
  assert.equal(
    await queue.enqueue([{ kind: "stop" }], "one", async () => true),
    true,
  );
});
