import assert from "node:assert/strict";
import test from "node:test";
import {
  performPreviewIntent,
  type PreviewIntent,
} from "../src/components/workbench/preview-intent.ts";
import type { ApplicationHost, Snapshot } from "../src/application-host.ts";
import type { PreviewRequest, PreviewSnapshot } from "../src/sequence-types.ts";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((yes) => {
    resolve = yes;
  });
  return { promise, resolve };
}
function fixture() {
  const calls: unknown[] = [],
    replacements: PreviewSnapshot[] = [];
  let active = true,
    draftCalls = 0;
  const state: PreviewSnapshot = { epoch: 13, controlSerial: 9, loaded: null };
  const host: Pick<ApplicationHost, "request" | "preview"> = {
    request: async (request) => {
      calls.push(request);
      return { generation: 48 } as Snapshot;
    },
    preview: async (request) => {
      calls.push(request);
      if (request.kind === "loadScene")
        state.loaded = {
          sceneId: request.sceneId,
        } as PreviewSnapshot["loaded"];
      return structuredClone(state);
    },
  };
  const intent: PreviewIntent = {
    host,
    sequenceId: "甲",
    command: "load",
    expectedEpoch: 13,
    draftEffect: false,
    beforeAction: async () => {
      draftCalls++;
      return true;
    },
    current: () => active,
    replaced: (value) => replacements.push(value),
  };
  return {
    intent,
    host,
    state,
    calls,
    replacements,
    retire: () => {
      active = false;
    },
    draftCalls: () => draftCalls,
  };
}
test("valid load uses the authoritative post-draft generation, without automatic execution", async () => {
  const f = fixture();
  assert.equal((await performPreviewIntent(f.intent))?.epoch, 13);
  assert.deepEqual(f.calls, [
    { kind: "snapshot" },
    { kind: "load", generation: 48, sequenceId: "甲" },
  ]);
  assert.equal(f.draftCalls(), 1);
});
test("leaving the target while draft application waits prevents even the project read", async () => {
  const f = fixture(),
    draft = deferred<boolean>();
  f.intent.beforeAction = () => draft.promise;
  const result = performPreviewIntent(f.intent);
  f.retire();
  draft.resolve(true);
  assert.equal(await result, undefined);
  assert.deepEqual(f.calls, []);
});
test("A to B to A cannot revive the first A operation", async () => {
  const f = fixture(),
    draft = deferred<boolean>();
  let binding = { target: "甲" };
  const original = binding;
  f.intent.current = () => binding === original;
  f.intent.beforeAction = () => draft.promise;
  const result = performPreviewIntent(f.intent);
  binding = { target: "乙" };
  binding = { target: "甲" };
  draft.resolve(true);
  assert.equal(await result, undefined);
  assert.deepEqual(f.calls, []);
});
test("cancelled or invalid drafts never load or control", async () => {
  const f = fixture();
  f.intent.beforeAction = async () => false;
  assert.equal(await performPreviewIntent(f.intent), undefined);
  assert.deepEqual(f.calls, []);
  f.intent.beforeAction = async () => {
    throw Error("时间输入无效");
  };
  await assert.rejects(performPreviewIntent(f.intent), /时间输入无效/);
  assert.deepEqual(f.calls, []);
});
test("a stale retained callback cannot run the editor flush", async () => {
  const f = fixture();
  f.retire();
  assert.equal(await performPreviewIntent(f.intent), undefined);
  assert.equal(f.draftCalls(), 0);
});
test("a late project snapshot cannot load the previous sequence", async () => {
  const f = fixture();
  f.host.request = async (request) => {
    f.calls.push(request);
    f.retire();
    return { generation: 48 } as Snapshot;
  };
  assert.equal(await performPreviewIntent(f.intent), undefined);
  assert.deepEqual(f.calls, [{ kind: "snapshot" }]);
});
test("an already sent load remains sent, but its late result is not published or retried", async () => {
  const f = fixture();
  const preview = f.host.preview;
  f.host.preview = async (request) => {
    const reply = await preview(request);
    f.retire();
    return reply;
  };
  assert.equal(await performPreviewIntent(f.intent), undefined);
  assert.deepEqual(f.calls, [
    { kind: "snapshot" },
    { kind: "load", generation: 48, sequenceId: "甲" },
  ]);
});
test("control waits for the current serial and revalidates before sending", async () => {
  const f = fixture();
  f.intent.command = { kind: "next" };
  assert.equal((await performPreviewIntent(f.intent))?.epoch, 13);
  assert.deepEqual(f.calls, [
    { kind: "snapshot" },
    { kind: "control", epoch: 13, serial: 10, command: { kind: "next" } },
  ]);
});
test("a control snapshot completing after the workspace changed cannot send", async () => {
  const f = fixture();
  f.intent.command = { kind: "stop" };
  const preview = f.host.preview;
  f.host.preview = async (request) => {
    const reply = await preview(request);
    f.retire();
    return reply;
  };
  assert.equal(await performPreviewIntent(f.intent), undefined);
  assert.deepEqual(f.calls, [{ kind: "snapshot" }]);
  assert.equal(f.draftCalls(), 0);
});
test("changed epoch publishes the real replacement but never controls or automatically retries", async () => {
  const f = fixture();
  f.intent.command = { kind: "next" };
  f.state.epoch = 14;
  await assert.rejects(performPreviewIntent(f.intent), /预览内容已更换/);
  assert.deepEqual(f.calls, [{ kind: "snapshot" }]);
  assert.deepEqual(f.replacements, [f.state]);
});
test("pause, stop, rate and effect-draft resume do not apply unrelated editor input", async () => {
  for (const command of [
    { kind: "pause" },
    { kind: "stop" },
    { kind: "setRate", percent: 200 },
    { kind: "resume" },
  ] as const) {
    const f = fixture();
    f.intent.command = command;
    f.intent.draftEffect = true;
    await performPreviewIntent(f.intent);
    assert.equal(f.draftCalls(), 0);
    assert.equal(f.calls.length, 2);
  }
  const f = fixture();
  f.intent.command = { kind: "resume" };
  await performPreviewIntent(f.intent);
  assert.equal(f.draftCalls(), 1);
});
test("single scene reuses the original load/receipt flow with lifecycle checks", async () => {
  const f = fixture();
  f.intent.command = "startScene";
  f.intent.sceneId = "scene-甲";
  await performPreviewIntent(f.intent);
  assert.deepEqual(f.calls, [
    { kind: "snapshot" },
    { kind: "loadScene", generation: 48, sceneId: "scene-甲" },
    {
      kind: "control",
      epoch: 13,
      serial: 10,
      command: { kind: "execute", stepId: "scene-甲" },
    },
  ]);
  const old = fixture();
  old.intent.command = "startScene";
  old.intent.sceneId = "scene-甲";
  const preview = old.host.preview;
  old.host.preview = async (request) => {
    const reply = await preview(request);
    old.retire();
    return reply;
  };
  await assert.rejects(performPreviewIntent(old.intent), /目标已更换/);
  assert.equal(old.calls.length, 2);
});
test("the original host refusal is reported exactly once without retry", async () => {
  const f = fixture();
  f.intent.command = { kind: "next" };
  f.host.preview = async (request: PreviewRequest) => {
    f.calls.push(request);
    if (request.kind === "control") throw Error("原播放器拒绝旧序号");
    return f.state;
  };
  await assert.rejects(performPreviewIntent(f.intent), /原播放器拒绝旧序号/);
  assert.equal(f.calls.length, 2);
});
