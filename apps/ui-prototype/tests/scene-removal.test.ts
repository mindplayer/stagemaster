import test from "node:test";
import assert from "node:assert/strict";
import { usageProject } from "./scene-usage-fixture.ts";
import {
  sceneRemovalCommands,
  sceneRemovalReview,
} from "../src/components/workbench/scene-removal.ts";
test("混合组不跳过被引用场景；顺序与身份明确，拒绝失效／重复／超限", () => {
  const project = usageProject();
  project.scenes.push(
    { id: "free1", name: "暖场", values: [], effects: [] },
    { id: "free2", name: "暖场", values: [], effects: [] },
  );
  const before = structuredClone(project);
  assert.deepEqual(sceneRemovalCommands(project, ["free2", "free1"]), [
    { op: "removeScene", id: "free1" },
    { op: "removeScene", id: "free2" },
  ]);
  assert.throws(
    () => sceneRemovalCommands(project, ["free1", "s"]),
    /仍被使用/,
  );
  assert.equal(sceneRemovalReview(project, ["s"])[0].usages.length, 50);
  for (const ids of [
    [],
    ["missing"],
    ["free1", "free1"],
    Array(129).fill("free1"),
  ])
    assert.throws(() => sceneRemovalReview(project, ids));
  assert.deepEqual(project, before);
});
test("旧卡点或停用且锁定的片段均保持删除保护", () => {
  const project = usageProject();
  project.sequences = [];
  project.audio!.markers = [];
  assert.throws(() => sceneRemovalCommands(project, ["s"]), /仍被使用/);
  delete project.audio!.lightingClips;
  project.audio!.markers = [
    { id: "marker", name: "旧卡点", sceneId: "s", timeMs: 0 },
  ];
  assert.throws(() => sceneRemovalCommands(project, ["s"]), /仍被使用/);
  project.audio!.markers[0].sceneId = null;
  assert.equal(sceneRemovalCommands(project, ["s"])[0].op, "removeScene");
});
