import test from "node:test";
import assert from "node:assert/strict";
import { stageProject } from "./stage-organization-fixture.ts";
import {
  nearbyFixtures,
  selectionAfterBox,
  selectionAfterClick,
} from "../src/components/scene-plan/selection-model.ts";

test("俯视同位置不同安装高度返回全部候选，不静默选择顶层灯具", () => {
  const stage = stageProject().stage;
  stage.placements.push({
    ...stage.placements[0],
    fixtureId: "floor",
    positionMeters: { x: "0", y: "2", z: "0.5" },
  });
  assert.deepEqual(nearbyFixtures(stage.placements, [0, 2], 0.5), [
    "front",
    "floor",
  ]);
  assert.deepEqual(nearbyFixtures(stage.placements, [5, 4], 0.5), []);
});
test("增减选择保留灯序，普通单击替换，取消不靠反向事务模拟", () => {
  const before = ["audience", "front"];
  assert.deepEqual(selectionAfterClick(before, "loose", true), [
    "audience",
    "front",
    "loose",
  ]);
  assert.deepEqual(selectionAfterClick(before, "audience", true), ["front"]);
  assert.deepEqual(selectionAfterClick(before, "loose", false), ["loose"]);
  assert.deepEqual(before, ["audience", "front"]);
});
test("双向框选边界一致，增选去重且按原灯位顺序追加", () => {
  const stage = stageProject().stage;
  assert.deepEqual(selectionAfterBox(stage, [], [-1, 0], [21, 3], false), [
    "front",
    "audience",
  ]);
  assert.deepEqual(
    selectionAfterBox(stage, ["loose", "front"], [21, 3], [-1, 0], true),
    ["loose", "front", "audience"],
  );
  assert.deepEqual(
    selectionAfterBox(stage, ["front"], [90, 90], [91, 91], false),
    [],
  );
});
