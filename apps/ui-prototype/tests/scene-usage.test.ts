import test from "node:test";
import assert from "node:assert/strict";
import {
  sceneUsages,
  usageMatches,
  hasSceneUsage,
} from "../src/components/workbench/scene-usage.ts";
import { locateSceneUsage } from "../src/components/workbench/scene-usage-navigation.ts";
import { usageProject } from "./scene-usage-fixture.ts";
test("引用依据身份而非名称；保留每个重复使用、停用锁定片段和卡点绑定", () => {
  const project = usageProject();
  const before = structuredClone(project);
  const usages = sceneUsages(project, "s");
  assert.equal(usages.length, 50);
  assert.equal(new Set(usages.map((s) => s.key)).size, 50);
  assert.deepEqual(usages[0].target, {
    kind: "step",
    sequenceId: "a",
    id: "a-0",
  });
  assert.equal(
    usages.at(-2)?.detail,
    "灯光片段 · 00:02.000—00:04.000 · 已停用 · 已锁定",
  );
  assert.equal(usages.at(-1)?.detail, "卡点绑定 · 00:01.000");
  assert.equal(sceneUsages(project, "other").length, 2);
  assert.deepEqual(sceneUsages(project, "missing"), []);
  assert.equal(usageMatches(usages[0], " 排练 "), true);
  assert.equal(
    hasSceneUsage(project, "s", { kind: "step", sequenceId: "b", id: "a-0" }),
    false,
  );
  assert.deepEqual(project, before);
  delete project.audio!.lightingClips;
  assert.equal(sceneUsages(project, "s").at(-1)?.target.kind, "marker");
});
test("引用定位在草稿保护后再验身份，失败不切页，成功仅调用编辑导航", async () => {
  const project = usageProject();
  const calls: string[] = [];
  let reject = true;
  const options = {
    projectId: project.id,
    sceneId: "s",
    target: { kind: "clip" as const, id: "clip" },
    read: () => project,
    run: async (work: () => Promise<void>) => {
      if (reject) return false;
      await work();
      return true;
    },
    reveal: () => {
      calls.push("reveal");
      return true;
    },
    open: (kind: string) => {
      calls.push(kind);
    },
  };
  assert.equal(await locateSceneUsage(options), false);
  assert.deepEqual(calls, []);
  reject = false;
  await locateSceneUsage(options);
  assert.deepEqual(calls, ["reveal", "audio"]);
  calls.length = 0;
  project.audio!.lightingClips![0].sceneId = "other";
  await assert.rejects(locateSceneUsage(options), /已变化/);
  assert.deepEqual(calls, []);
  await assert.rejects(
    locateSceneUsage({ ...options, projectId: "other-project" }),
    /已变化/,
  );
});
