import test from "node:test";
import assert from "node:assert/strict";
import {
  canonical,
  stageCommand,
  translated,
  selectedStage,
  rectangle,
} from "../src/stage-tools.ts";
import type { StageObject, StageView } from "../src/stage-types.ts";
const room: StageObject = {
  kind: "space",
  value: {
    id: "room",
    name: " 主厅 ",
    outlineMeters: rectangle(0, 0, 8, 6),
    floorElevationMeters: "1.20",
    clearHeightMeters: "4.50",
  },
};
const lamp: StageObject = {
  kind: "placement",
  value: {
    fixtureId: "lamp",
    spaceId: "room",
    positionMeters: { x: "2", y: "3", z: "5.5" },
    rotationDegreesXYZ: { x: "0", y: "-30", z: "90" },
  },
};
test("空间输入规范化后提交，非法草稿不得变为零", () => {
  assert.deepEqual(stageCommand(room), {
    op: "putSpace",
    id: "room",
    name: "主厅",
    outlineMeters: rectangle(0, 0, 8, 6),
    floorElevationMeters: "1.2",
    clearHeightMeters: "4.5",
  });
  for (const v of ["", " ", "NaN", "Infinity", "1e6", "2m"])
    assert.throws(() => canonical(v));
  assert.equal(canonical("-0.000"), "0");
  assert.equal(canonical(".25"), "0.25");
  assert.equal(room.value.name, " 主厅 ");
});
test("平面拖动只改变选中对象的世界 XY，不改安装旋转与高度", () => {
  const before = structuredClone(lamp),
    next = translated(lamp, 1.1, -2);
  assert.equal(next.kind, "placement");
  if (next.kind !== "placement") throw Error("wrong kind");
  assert.deepEqual(next.value.positionMeters, { x: "3.1", y: "1", z: "5.5" });
  assert.deepEqual(next.value.rotationDegreesXYZ, {
    x: "0",
    y: "-30",
    z: "90",
  });
  assert.equal(next.value.spaceId, "room");
  assert.deepEqual(lamp, before);
  const moved = translated(room, -1, 2);
  if (moved.kind !== "space") throw Error("wrong kind");
  assert.deepEqual(moved.value.outlineMeters, rectangle(-1, 2, 8, 6));
  assert.equal(moved.value.floorElevationMeters, "1.20");
});
test("无效选择不降级选中其他对象，安装点按灯具身份恢复", () => {
  const stage: StageView = {
    spaces: room.kind === "space" ? [room.value] : [],
    constructions: [],
    placements: lamp.kind === "placement" ? [lamp.value] : [],
  };
  assert.equal(selectedStage(stage, { kind: "space", id: "missing" }), null);
  assert.equal(
    selectedStage(stage, { kind: "construction", id: "room" }),
    null,
  );
  assert.deepEqual(
    selectedStage(stage, { kind: "placement", id: "lamp" }),
    lamp,
  );
});
