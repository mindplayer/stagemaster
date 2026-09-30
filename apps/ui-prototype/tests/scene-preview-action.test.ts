import { test } from "node:test";
import assert from "node:assert/strict";
import { startScenePreview } from "../src/components/workbench/scene-preview-action.ts";
import type { ApplicationHost, Snapshot } from "../src/application-host.ts";
import type { PreviewRequest, PreviewSnapshot } from "../src/sequence-types.ts";

function hostFixture() {
  const calls: unknown[] = [];
  const loaded = {
    epoch: 8,
    controlSerial: 11,
    loaded: { sceneId: "scene-b", status: "idle" },
  } as PreviewSnapshot;
  const host: Pick<ApplicationHost, "request" | "preview"> = {
    request: async (request) => {
      calls.push(request);
      return { generation: 42 } as Snapshot;
    },
    preview: async (request) => {
      calls.push(request);
      return loaded;
    },
  };
  return { host, calls, loaded };
}

test("场景预演使用应用草稿后的宿主版本和载入回执执行所选场景", async () => {
  const { host, calls, loaded } = hostFixture();
  assert.equal(await startScenePreview(host, "scene-b", () => true), loaded);
  assert.deepEqual(calls, [
    { kind: "snapshot" },
    { kind: "loadScene", generation: 42, sceneId: "scene-b" },
    {
      kind: "control",
      epoch: 8,
      serial: 12,
      command: { kind: "execute", stepId: "scene-b" },
    },
  ]);
});

test("迟到的工程快照不能替换用户切换后的播放内容", async () => {
  const { host, calls } = hostFixture();
  let current = true;
  const request = host.request;
  host.request = async (value) => {
    const result = await request(value);
    current = false;
    return result;
  };
  await assert.rejects(
    startScenePreview(host, "scene-b", () => current),
    /目标已更换/,
  );
  assert.deepEqual(calls, [{ kind: "snapshot" }]);
});

test("场景载入过程中离开编辑目标，迟到回执不得继续执行", async () => {
  const { host, calls } = hostFixture();
  let current = true;
  const preview = host.preview;
  host.preview = async (value) => {
    const result = await preview(value);
    current = false;
    return result;
  };
  await assert.rejects(
    startScenePreview(host, "scene-b", () => current),
    /目标已更换/,
  );
  assert.equal(calls.length, 2);
});

test("编译失败和目标不符均不得执行已载入的其他节目", async () => {
  const { host, calls, loaded } = hostFixture();
  loaded.loaded!.sceneId = "scene-a";
  await assert.rejects(
    startScenePreview(host, "scene-b", () => true),
    /载入未完成/,
  );
  assert.equal(calls.length, 2);
  host.preview = async () => {
    throw new Error("场景无法编译");
  };
  await assert.rejects(
    startScenePreview(host, "scene-b", () => true),
    /无法编译/,
  );
});

test("并发播放器拒绝旧纪元时不自动重试或覆盖新播放", async () => {
  const { host, calls } = hostFixture();
  const preview = host.preview;
  host.preview = async (value: PreviewRequest) => {
    if (value.kind === "control") {
      calls.push(value);
      throw new Error("播放纪元已过期");
    }
    return preview(value);
  };
  await assert.rejects(
    startScenePreview(host, "scene-b", () => true),
    /纪元已过期/,
  );
  assert.equal(calls.length, 3);
});
