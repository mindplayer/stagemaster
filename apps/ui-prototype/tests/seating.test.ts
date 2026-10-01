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

const arcVector = JSON.parse(
  readFileSync(
    new URL(
      "../../../tools/test-data/seating-arc-layout.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
test("弧排与独立三十度向量一致，逐座面向焦点且保留中心枢轴", () => {
  const p = seatingLayout(arcVector.shape)!;
  near(p.worldCenters.flat(), arcVector.centers.flat());
  near(p.outline.flat(), arcVector.outline.flat());
  near(
    p.angles.map((a) => a + p.yaw),
    arcVector.anglesDegrees,
  );
  near(p.worldFocus!, arcVector.focus);
  assert.ok(Math.abs(p.width - arcVector.width) < 1e-9);
  assert.ok(Math.abs(p.depth - arcVector.depth) < 1e-9);
  const moved = stageCommand(
    translated(
      {
        kind: "construction",
        value: {
          id: "a",
          name: "弧排",
          shape: { ...arcVector.shape, arc: { radiusMeters: "4.00" } },
        },
      },
      2,
      -3,
    ),
  );
  assert.equal(moved.op, "putConstruction");
  if (moved.op !== "putConstruction" || moved.shape.kind !== "seating")
    throw Error();
  assert.equal(moved.shape.arc?.radiusMeters, "4");
  assert.deepEqual(moved.shape.positionMeters, { x: "12", y: "17", z: "0.5" });
});
test("弧排草稿拒绝半径、重叠与半圆越界；前排通道保留净宽", () => {
  for (const patch of [
    { arc: { radiusMeters: "" } },
    { arc: { radiusMeters: "1" } },
    { arc: { radiusMeters: "10001" } },
    { columnSpacingMeters: "0.5" },
  ])
    assert.equal(seatingLayout({ ...arcVector.shape, ...patch }), null);
  const p = seatingLayout({
    ...arcVector.shape,
    columns: 4,
    columnSpacingMeters: "0.6",
    aisle: { afterColumn: 2, widthMeters: "1.2" },
  })!;
  const corner = (i: number, x: number) => {
    const a = (p.angles[i] * Math.PI) / 180;
    return [
      p.centers[i][0] + x * Math.cos(a) - 0.25 * Math.sin(a),
      p.centers[i][1] + x * Math.sin(a) + 0.25 * Math.cos(a),
    ];
  };
  const a = corner(1, 0.25),
    b = corner(2, -0.25);
  assert.ok(Math.abs(Math.hypot(a[0] - b[0], a[1] - b[1]) - 1.2) < 1e-9);
});
