import test from "node:test";
import assert from "node:assert/strict";
import { stageProject } from "./stage-organization-fixture.ts";
import {
  ALL_VISIBLE,
  visibleStage,
  revealStageTarget,
} from "../src/components/stage/stage-display.ts";
import { outlineMembers } from "../src/components/stage/stage-outliner-model.ts";
import {
  planPoints,
  selectionPoints,
} from "../src/components/stage/plan-focus.ts";
import { planPreview } from "../src/components/stage/plan-preview.ts";
import { selectInBox } from "../src/placement-tools.ts";
import { translated } from "../src/stage-tools.ts";

test("隐藏空间排除其构件与灯位，未归属对象保留且工程未修改", () => {
  const { stage } = stageProject(),
    before = structuredClone(stage);
  const shown = visibleStage(stage, {
    hiddenSpaces: ["audience"],
    hiddenLayers: [],
  });
  assert.deepEqual(
    shown.spaces.map((s) => s.id),
    ["stage"],
  );
  assert.deepEqual(
    shown.constructions.map((c) => c.id),
    ["rig"],
  );
  assert.deepEqual(
    shown.placements.map((p) => p.fixtureId),
    ["front", "loose"],
  );
  assert.deepEqual(stage, before);
  assert.equal(shown.attachments, stage.attachments);
});
test("空间轮廓、桁架、构件和灯具类别互相独立", () => {
  const { stage } = stageProject();
  const shown = visibleStage(stage, {
    hiddenSpaces: [],
    hiddenLayers: ["spaces", "rigs"],
  });
  assert.equal(shown.spaces.length, 0);
  assert.equal(shown.constructions.length, 24);
  assert.equal(shown.placements.length, 3);
  assert.equal(shown.attachments.length, 1);
  assert.equal(
    visibleStage(stage, {
      hiddenSpaces: [],
      hiddenLayers: ["constructions", "fixtures"],
    }).constructions[0].id,
    "rig",
  );
});
test("定位只恢复目标空间和类别，重复选择保留视图身份以免取消拖动", () => {
  const { stage } = stageProject();
  const state = {
    hiddenSpaces: ["stage", "audience"],
    hiddenLayers: ["fixtures", "rigs"] as const,
  };
  const next = revealStageTarget(
    stage,
    { ...state, hiddenLayers: [...state.hiddenLayers] },
    { kind: "placement", id: "front" },
  );
  assert.deepEqual(next, {
    hiddenSpaces: ["audience"],
    hiddenLayers: ["rigs"],
  });
  assert.equal(
    revealStageTarget(stage, next, { kind: "placement", id: "front" }),
    next,
  );
  assert.equal(
    revealStageTarget(stage, ALL_VISIBLE, { kind: "construction", id: "rig" }),
    ALL_VISIBLE,
  );
});
test("隐藏灯具不能被框选或纳入聚焦边界，即使旧选择仍含它", () => {
  const shown = visibleStage(stageProject().stage, {
    hiddenSpaces: ["audience"],
    hiddenLayers: [],
  });
  assert.deepEqual(selectInBox(shown.placements, [-100, -100], [100, 100]), [
    "front",
    "loose",
  ]);
  assert.deepEqual(selectionPoints(shown, null, ["audience", "front"]), [
    [0, 2],
  ]);
  const empty = visibleStage(stageProject().stage, {
    hiddenSpaces: [],
    hiddenLayers: ["spaces", "fixtures", "constructions", "rigs"],
  });
  assert.deepEqual(planPoints(empty), []);
  assert.equal(selectionPoints(empty, null, ["front"]), null);
});
test("目录使用真实空间及类别，不猜座椅名称，标签舍入不损失持久精度", () => {
  const project = stageProject(),
    before = structuredClone(project);
  project.stage.constructions[1].name = "前桁架";
  const rows = outlineMembers(project);
  assert.equal(rows.filter((m) => m.space === "audience").length, 25);
  assert.equal(
    rows.find((m) => m.target.id === "piece-0")?.category,
    "constructions",
  );
  assert.match(
    rows.find((m) => m.target.id === "rig")!.detail,
    /8.123 米 · 1 台灯/,
  );
  assert.match(
    rows.find((m) => m.target.id === "front")!.detail,
    /前桁架 · 高度 5.877 米/,
  );
  assert.deepEqual(
    project.stage.constructions[0],
    before.stage.constructions[0],
  );
});
test("桁架草稿和拖动仍带动挂接灯位的预览，不改挂接或灯具高度", () => {
  const { stage } = stageProject(),
    before = structuredClone(stage);
  const rig = { kind: "construction" as const, value: stage.constructions[0] };
  const light = { kind: "placement" as const, value: stage.placements[0] };
  const moved = translated(rig, 2, 3);
  const expected = translated(light, 2, 3);
  assert.deepEqual(planPreview(stage, moved, null)(light), expected);
  assert.deepEqual(
    planPreview(stage, null, {
      object: rig,
      target: { kind: "construction", id: "rig" },
      fixtures: [],
      dx: 2,
      dy: 3,
      handle: null,
    })(light),
    expected,
  );
  assert.deepEqual(stage, before);
});
