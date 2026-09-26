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

// Protect topology-preserving UI transforms before Rust performs authoritative validation.
import { footprintArea, footprintBounds, lShape, resized, resizedByHandle } from '../src/stage-geometry.ts';
test('L 形尺寸生成保留缺口面积，拒绝穿透外边界的缺口', () => {
  const outline = lShape(-2, 3, 8, 6, 2, 3);
  assert.equal(footprintArea(outline), 42);
  assert.deepEqual(footprintBounds(outline), { minX: -2, minY: 3, maxX: 6, maxY: 9 });
  for (const notch of [[0, 2], [8, 2], [2, 6], [2, -1]]) assert.throws(() => lShape(0, 0, 8, 6, notch[0]!, notch[1]!));
});
test('整体尺寸调整保留凹形拓扑、标高、身份和原始输入', () => {
  if (room.kind !== 'space') throw Error('type');
  const source: StageObject = { kind: 'space', value: { ...room.value, outlineMeters: lShape(-2, 3, 8, 6, 2, 3) } };
  const before = structuredClone(source);
  const next = resized(source, { minX: -2, minY: 3, maxX: 14, maxY: 6 });
  assert.equal(next.kind, 'space');
  if (next.kind !== 'space') throw Error('type');
  assert.deepEqual(next.value.outlineMeters, lShape(-2, 3, 16, 3, 4, 1.5));
  assert.equal(next.value.floorElevationMeters, '1.20');
  assert.equal(next.value.id, source.value.id);
  assert.deepEqual(source, before);
});
test('从左下缩放固定对边，跨越对边时限幅且不翻转轮廓', () => {
  const next = resizedByHandle(room, 'sw', 1, 2);
  if (next.kind !== 'space') throw Error('type');
  assert.deepEqual(next.value.outlineMeters, rectangle(1, 2, 7, 4));
  const crossed = resizedByHandle(room, 'sw', 99, 99);
  if (crossed.kind !== 'space') throw Error('type');
  assert.deepEqual(crossed.value.outlineMeters, rectangle(7.9, 5.9, .1, .1));
});
test('空值或非法包围框不应压扁空间，灯位不能被尺寸操作改变', () => {
  for (const maxX of [0, -1, NaN, Infinity]) assert.throws(() => resized(room, { minX: 0, minY: 0, maxX, maxY: 6 }));
  assert.deepEqual(resizedByHandle(lamp, 'ne', 3, 3), lamp);
});
