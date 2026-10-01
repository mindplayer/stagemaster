import { test } from "node:test";
import assert from "node:assert/strict";
import {
  EffectDraftSession,
  type EffectDraftValue,
  type EffectDraftState,
} from "../src/components/workbench/effect-draft-session.ts";
import type { ApplicationHost, Snapshot } from "../src/application-host.ts";
import type { PreviewRequest, PreviewSnapshot } from "../src/sequence-types.ts";
import { createEffect } from "../src/effect-tools.ts";
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => {
    resolve = r;
  });
  return { promise, resolve };
}
const tick = () => new Promise((r) => setImmediate(r));
function value(period = 1000): EffectDraftValue {
  return {
    sceneId: "scene",
    effect: {
      ...createEffect("breathe", "effect", ["lamp"]),
      periodMs: period,
    },
    illuminate: false,
  };
}
function fixture() {
  const calls: PreviewRequest[] = [],
    states: EffectDraftState[] = [];
  let frame = { epoch: 8, controlSerial: 0, loaded: null } as PreviewSnapshot;
  const host: Pick<ApplicationHost, "request" | "preview"> = {
    request: async () => ({ generation: 42 }) as Snapshot,
    preview: async (request) => {
      calls.push(request);
      if (request.kind === "beginEffectDraft")
        frame = {
          ...frame,
          epoch: 9,
          loaded: {
            draftEffectId: "effect",
            status: "running",
          } as PreviewSnapshot["loaded"],
        };
      if (request.kind === "endEffectDraft" && request.epoch === frame.epoch)
        frame = { ...frame, epoch: 10, loaded: null };
      return frame;
    },
  };
  const session = new EffectDraftSession(host, (s) => states.push(s));
  return { session, host, calls, states, frame: () => frame };
}
test("效果即时预演合并在途后的输入，只发送最新候选并保持递增序号", async () => {
  const f = fixture();
  assert.equal(await f.session.begin(value()), true);
  const pending = deferred<PreviewSnapshot>();
  const original = f.host.preview;
  f.host.preview = async (r) => {
    const result = await original(r);
    return r.kind === "updateEffectDraft" && r.serial === 1
      ? pending.promise
      : result;
  };
  f.session.update(value(1100));
  await tick();
  f.session.update(value(1200));
  f.session.update(value(1300));
  assert.equal(f.calls.filter((r) => r.kind === "updateEffectDraft").length, 1);
  pending.resolve(f.frame());
  await tick();
  const updates = f.calls.filter((r) => r.kind === "updateEffectDraft");
  assert.deepEqual(
    updates.map((r) => [r.serial, r.effect.periodMs]),
    [
      [1, 1100],
      [2, 1300],
    ],
  );
  assert.equal(f.states.at(-1)?.active, true);
  await f.session.end();
  assert.equal(f.states.at(-1)?.active, false);
});
test("取消正在开始的预演，会释放实际回执代次且不再开启观看", async () => {
  const f = fixture(),
    pending = deferred<PreviewSnapshot>(),
    original = f.host.preview;
  f.host.preview = async (r) => {
    const result = await original(r);
    return r.kind === "beginEffectDraft" ? pending.promise : result;
  };
  const begin = f.session.begin(value());
  await tick();
  const end = f.session.end();
  pending.resolve(f.frame());
  assert.equal(await begin, false);
  await end;
  assert.deepEqual(f.calls.at(-1), { kind: "endEffectDraft", epoch: 9 });
  assert.equal(f.states.at(-1)?.active, false);
});
test("无效新输入不会被较早成功回执清掉，改正后恢复即时更新", async () => {
  const f = fixture();
  await f.session.begin(value());
  const pending = deferred<PreviewSnapshot>(),
    original = f.host.preview;
  f.host.preview = async (r) => {
    const result = await original(r);
    return r.kind === "updateEffectDraft" && r.serial === 1
      ? pending.promise
      : result;
  };
  f.session.update(value(1100));
  await tick();
  f.session.problem(new Error("循环周期无效"));
  pending.resolve(f.frame());
  await tick();
  assert.match(f.states.at(-1)!.message, /循环周期无效/);
  f.session.update(value(1500));
  await tick();
  assert.match(f.states.at(-1)!.message, /即时预演中/);
});
test("新播放占用后检测结束，旧结束不再发送命令；保存代次更新仍可继续", async () => {
  const f = fixture();
  await f.session.begin(value());
  f.host.request = async () => ({ generation: 43 }) as Snapshot;
  f.session.update(value(1500));
  await tick();
  const last = f.calls.at(-1);
  assert.equal(last?.kind, "updateEffectDraft");
  if (last?.kind === "updateEffectDraft") assert.equal(last.generation, 43);
  f.host.preview = async () => ({ epoch: 11, controlSerial: 0, loaded: null });
  await f.session.inspect();
  assert.equal(f.states.at(-1)?.active, false);
  const length = f.calls.length;
  await f.session.end();
  assert.equal(f.calls.length, length);
});
test("结束会等待在途更新，丢弃候选后只释放所属代次", async () => {
  const f = fixture();
  await f.session.begin(value());
  const pending = deferred<PreviewSnapshot>(),
    original = f.host.preview;
  f.host.preview = async (r) => {
    const result = await original(r);
    return r.kind === "updateEffectDraft" ? pending.promise : result;
  };
  f.session.update(value(1100));
  await tick();
  f.session.update(value(1200));
  const end = f.session.end();
  pending.resolve(f.frame());
  await end;
  assert.equal(f.calls.filter((r) => r.kind === "updateEffectDraft").length, 1);
  assert.deepEqual(f.calls.at(-1), { kind: "endEffectDraft", epoch: 9 });
});
