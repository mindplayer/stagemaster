import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { seatingLayout } from "../src/seating-tools.ts";
import { stageCommand, translated } from "../src/stage-tools.ts";
import {
  planPoints,
  selectionPoints,
} from "../src/components/stage/plan-focus.ts";
import {
  visibleStage,
  revealStageTarget,
} from "../src/components/stage/stage-display.ts";
import type {
  SeatingShape,
  StageObject,
  StageView,
} from "../src/stage-types.ts";
const vector = JSON.parse(
  readFileSync(
    new URL("../../../tools/test-data/seating-layout.json", import.meta.url),
    "utf8",
  ),
);
const shape: SeatingShape = vector.shape;
const object: StageObject = {
  kind: "construction",
  value: { id: "seats", name: "  观众  ", shape },
};
const stage: StageView = {
  spaces: [],
  constructions: [object.value],
  placements: [],
  attachments: [],
};
const near = (a: number[], b: number[]) => {
  assert.equal(a.length, b.length);
  a.forEach((v, i) => assert.ok(Math.abs(v - b[i]!) < 1e-10));
};
test("座区按独立坐标向量生成朝向、中心与净通道", () => {
  const p = seatingLayout(shape)!;
  near(p.worldCenters.flat(), vector.centers.flat());
  near(p.outline.flat(), vector.outline.flat());
  assert.equal(p.width, vector.width);
  assert.equal(p.depth, vector.depth);
  near(planPoints(stage).flat(), vector.outline.flat());
  near(selectionPoints(stage, object, [])!.flat(), vector.outline.flat());
});
test("无效或过大草稿不分配座椅，也不以空字符串替换为零", () => {
  for (const patch of [
    { rows: 0 },
    { rows: 64, columns: 64 },
    { rows: 1.5 },
    { seatWidthMeters: "" },
    { columnSpacingMeters: ".4" },
    { yawDegrees: "NaN" },
    { aisle: { afterColumn: 4, widthMeters: "2" } },
    { aisle: { afterColumn: 2, widthMeters: ".3" } },
    { positionMeters: { x: "100000", y: "0", z: "0" } },
  ])
    assert.equal(seatingLayout({ ...shape, ...patch }), null);
});
test("整区移动不改变排列参数，规范化提交与隐藏后定位恢复", () => {
  const moved = translated(object, 2, -3);
  assert.equal(moved.kind, "construction");
  const cmd = stageCommand(moved);
  assert.equal(cmd.op, "putConstruction");
  if (cmd.op !== "putConstruction" || cmd.shape.kind !== "seating")
    throw Error();
  assert.equal(cmd.name, "观众");
  assert.deepEqual(cmd.shape.positionMeters, { x: "12", y: "17", z: "0.5" });
  assert.equal(cmd.shape.rows, 2);
  const hidden = { hiddenSpaces: [], hiddenLayers: ["seating" as const] };
  assert.equal(visibleStage(stage, hidden).constructions.length, 0);
  assert.deepEqual(
    revealStageTarget(stage, hidden, { kind: "construction", id: "seats" })
      .hiddenLayers,
    [],
  );
  assert.equal(shape.positionMeters.x, "10");
});
