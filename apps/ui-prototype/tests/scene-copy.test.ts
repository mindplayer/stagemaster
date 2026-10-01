import test from "node:test";
import assert from "node:assert/strict";
import { sceneCopyCommands } from "../src/components/workbench/scene-copy-tools.ts";
import { sceneActions } from "../src/components/workbench/scene-actions.ts";
import { stageProject } from "./stage-organization-fixture.ts";
import type { EditCommand, SceneView } from "../src/application-host";
const scenes: SceneView[] = ["暖场", "蓝色", "暖场 副本"].map((name, i) => ({
  id: `s${i}`,
  name,
  values: [],
  effects: [],
}));
test("场景复制按工程顺序、避让现有和同批名称，拒绝失效／重复／超限", () => {
  assert.deepEqual(sceneCopyCommands(scenes, ["s1", "s0"]), [
    { op: "duplicateScene", id: "s0", name: "暖场 副本 2" },
    { op: "duplicateScene", id: "s1", name: "蓝色 副本" },
  ]);
  const sameName = [...scenes, { ...scenes[0], id: "s3" }];
  assert.deepEqual(
    sceneCopyCommands(sameName, ["s3", "s0"]).map((c) =>
      "name" in c ? c.name : "",
    ),
    ["暖场 副本 2", "暖场 副本 3"],
  );
  for (const ids of [[], ["s0", "s0"], ["missing"], Array(129).fill("s0")])
    assert.throws(() => sceneCopyCommands(scenes, ids));
});
test("命令在草稿保护后读取最新源名称，整组一次提交，拒绝时不选择副本", async () => {
  const project = stageProject();
  project.scenes = structuredClone(scenes);
  const edits: EditCommand[] = [];
  const selected: string[] = [];
  let clears = 0;
  let reject = false;
  const actions = sceneActions({
    read: () => project,
    run: async (action) => {
      if (reject) return false;
      project.scenes[0].name = "修正后的暖场";
      await action();
      return true;
    },
    edit: async (command) => {
      edits.push(command);
      if (command.op !== "batch") throw Error("expected batch");
      for (const c of command.commands) {
        if (c.op !== "duplicateScene") throw Error("expected copy");
        project.scenes.push({
          ...scenes[0],
          id: `copy${project.scenes.length}`,
          name: c.name,
        });
      }
    },
    select: (scene) => {
      selected.push(scene.id);
    },
    clearQuery: () => {
      clears++;
    },
    notice: () => {},
  });
  const ids = ["s0", "s1"];
  const result = await actions.copy(ids);
  assert.deepEqual(result, ["copy3", "copy4"]);
  assert.equal(edits.length, 1);
  assert.deepEqual(selected, ["copy3"]);
  assert.equal(clears, 1);
  assert.equal(project.scenes[3].name, "修正后的暖场 副本");
  reject = true;
  assert.equal(await actions.copy(["s0"]), null);
  assert.equal(edits.length, 1);
  assert.deepEqual(selected, ["copy3"]);
});
