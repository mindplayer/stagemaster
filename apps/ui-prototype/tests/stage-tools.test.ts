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
import {
  footprintArea,
  footprintBounds,
  lShape,
  resized,
  resizedByHandle,
} from "../src/stage-geometry.ts";
test("L 形尺寸生成保留缺口面积，拒绝穿透外边界的缺口", () => {
  const outline = lShape(-2, 3, 8, 6, 2, 3);
  assert.equal(footprintArea(outline), 42);
  assert.deepEqual(footprintBounds(outline), {
    minX: -2,
    minY: 3,
    maxX: 6,
    maxY: 9,
  });
  for (const notch of [
    [0, 2],
    [8, 2],
    [2, 6],
    [2, -1],
  ])
    assert.throws(() => lShape(0, 0, 8, 6, notch[0]!, notch[1]!));
});
test("整体尺寸调整保留凹形拓扑、标高、身份和原始输入", () => {
  if (room.kind !== "space") throw Error("type");
  const source: StageObject = {
    kind: "space",
    value: { ...room.value, outlineMeters: lShape(-2, 3, 8, 6, 2, 3) },
  };
  const before = structuredClone(source);
  const next = resized(source, { minX: -2, minY: 3, maxX: 14, maxY: 6 });
  assert.equal(next.kind, "space");
  if (next.kind !== "space") throw Error("type");
  assert.deepEqual(next.value.outlineMeters, lShape(-2, 3, 16, 3, 4, 1.5));
  assert.equal(next.value.floorElevationMeters, "1.20");
  assert.equal(next.value.id, source.value.id);
  assert.deepEqual(source, before);
});
test("从左下缩放固定对边，跨越对边时限幅且不翻转轮廓", () => {
  const next = resizedByHandle(room, "sw", 1, 2);
  if (next.kind !== "space") throw Error("type");
  assert.deepEqual(next.value.outlineMeters, rectangle(1, 2, 7, 4));
  const crossed = resizedByHandle(room, "sw", 99, 99);
  if (crossed.kind !== "space") throw Error("type");
  assert.deepEqual(crossed.value.outlineMeters, rectangle(7.9, 5.9, 0.1, 0.1));
});
test("空值或非法包围框不应压扁空间，灯位不能被尺寸操作改变", () => {
  for (const maxX of [0, -1, NaN, Infinity])
    assert.throws(() => resized(room, { minX: 0, minY: 0, maxX, maxY: 6 }));
  assert.deepEqual(resizedByHandle(lamp, "ne", 3, 3), lamp);
});

import {
  arrangePlacements,
  arrangementDraft,
  placementBatch,
  PlacementInputError,
  selectInBox,
  togglePlacement,
} from "../src/placement-tools.ts";
import type { FixturePlacement } from "../src/stage-types.ts";
const placements: FixturePlacement[] = ["a", "b", "c", "d"].map(
  (fixtureId, i) => ({
    fixtureId,
    spaceId: i === 3 ? "second" : "first",
    positionMeters: { x: String(i * i), y: String(2 * i), z: String(4 + i) },
    rotationDegreesXYZ: { x: "12", y: "-30", z: String(i * 15) },
  }),
);
test("直线布灯以中心等距生成，按灯序放置并保留底座和输入", () => {
  const original = structuredClone(placements),
    draft = {
      ...arrangementDraft(10, 5, 7, "room"),
      spacing: "2",
      angle: "90",
    };
  const next = arrangePlacements(["c", "a", "new"], placements, draft);
  assert.deepEqual(
    next.map((p) => p.positionMeters),
    [
      { x: "10", y: "3", z: "7" },
      { x: "10", y: "5", z: "7" },
      { x: "10", y: "7", z: "7" },
    ],
  );
  assert.deepEqual(
    next.map((p) => p.fixtureId),
    ["c", "a", "new"],
  );
  assert.deepEqual(
    next[0].rotationDegreesXYZ,
    placements[2].rotationDegreesXYZ,
  );
  assert.deepEqual(next[2].rotationDegreesXYZ, { x: "0", y: "0", z: "0" });
  assert.ok(next.every((p) => p.spaceId === "room"));
  assert.deepEqual(placements, original);
});
test("不满矩阵末行保留列网格，不偷偷居中错位；单灯不除零", () => {
  const draft = {
    ...arrangementDraft(),
    mode: "grid" as const,
    columns: "3",
    spacing: "2",
    rowSpacing: "3",
  };
  const next = arrangePlacements(["1", "2", "3", "4", "5"], [], draft);
  assert.deepEqual(
    next.map((p) => [p.positionMeters.x, p.positionMeters.y]),
    [
      ["-2", "-1.5"],
      ["0", "-1.5"],
      ["2", "-1.5"],
      ["-2", "1.5"],
      ["0", "1.5"],
    ],
  );
  assert.deepEqual(arrangePlacements(["a"], [], draft)[0].positionMeters, {
    x: "0",
    y: "0",
    z: "4",
  });
});
test("满圆不重复端点，半圆包含两端并支持起始角度", () => {
  const draft = {
    ...arrangementDraft(0, 0, 4),
    mode: "circle" as const,
    radius: "2",
  };
  const circle = arrangePlacements(["a", "b", "c", "d"], [], draft);
  assert.deepEqual(
    circle.map((p) => [p.positionMeters.x, p.positionMeters.y]),
    [
      ["2", "0"],
      ["0", "2"],
      ["-2", "0"],
      ["0", "-2"],
    ],
  );
  const arc = arrangePlacements(["a", "b", "c"], [], {
    ...draft,
    arc: "180",
    angle: "90",
  });
  assert.deepEqual(
    arc.map((p) => [p.positionMeters.x, p.positionMeters.y]),
    [
      ["0", "2"],
      ["-2", "0"],
      ["0", "-2"],
    ],
  );
});
test("组旋转以包围中心为轴再平移，保留各灯高度差、朝向及空间归属", () => {
  const draft = {
    ...arrangementDraft(),
    mode: "move" as const,
    turn: "90",
    dx: "2",
    dy: "-1",
    dz: "3",
  };
  const next = arrangePlacements(["a", "d"], placements, draft);
  assert.deepEqual(
    next.map((p) => p.positionMeters),
    [
      { x: "9.5", y: "-2.5", z: "7" },
      { x: "3.5", y: "6.5", z: "10" },
    ],
  );
  assert.deepEqual(
    next.map((p) => p.spaceId),
    ["first", "second"],
  );
  assert.deepEqual(
    next.map((p) => p.rotationDegreesXYZ),
    [placements[0].rotationDegreesXYZ, placements[3].rotationDegreesXYZ],
  );
});
test("分布按空间次序保留两端，选灯顺序不导致交叉；高度对齐不改平面", () => {
  const draft = {
    ...arrangementDraft(),
    mode: "align" as const,
    alignment: "distribute" as const,
  };
  const next = arrangePlacements(["d", "b", "a", "c"], placements, draft);
  assert.deepEqual(
    next.map((p) => p.positionMeters.x),
    ["9", "3", "0", "6"],
  );
  const aligned = arrangePlacements(["a", "d"], placements, {
    ...draft,
    axis: "z",
    alignment: "max",
  });
  assert.deepEqual(
    aligned.map((p) => p.positionMeters),
    [
      { x: "0", y: "0", z: "7" },
      { x: "9", y: "6", z: "7" },
    ],
  );
});
test("安装旋转仅明确启用时统一，批次不携带配适或编排命令", () => {
  const next = arrangePlacements(["a", "b"], placements, {
    ...arrangementDraft(),
    setRotation: true,
    rx: "180",
    ry: "20",
    rz: "-90",
  });
  assert.ok(
    next.every(
      (p) =>
        JSON.stringify(p.rotationDegreesXYZ) ===
        JSON.stringify({ x: "180", y: "20", z: "-90" }),
    ),
  );
  assert.deepEqual(placementBatch(next), {
    op: "batch",
    commands: next.map((placement) => ({
      op: "stage",
      command: { op: "putPlacement", placement },
    })),
  });
});
test("非法参数、范围、重复选择和未布置变换均拒绝，错误保留字段身份", () => {
  for (const patch of [
    { spacing: "" },
    { radius: "0", mode: "circle" },
    { columns: "2.5", mode: "grid" },
    { arc: "361", mode: "circle" },
    { turn: "NaN", mode: "move" },
  ])
    assert.throws(
      () =>
        arrangePlacements(["a"], placements, {
          ...arrangementDraft(),
          ...patch,
        } as ReturnType<typeof arrangementDraft>),
      PlacementInputError,
    );
  assert.throws(
    () =>
      arrangePlacements(["missing"], placements, {
        ...arrangementDraft(),
        mode: "move",
      }),
    /已有安装位置/,
  );
  assert.throws(
    () => arrangePlacements(["a", "a"], placements, arrangementDraft()),
    /重复/,
  );
  assert.throws(() => arrangePlacements([], [], arrangementDraft()), /1–256/);
  assert.throws(
    () =>
      arrangePlacements(
        Array.from({ length: 257 }, (_, i) => String(i)),
        [],
        arrangementDraft(),
      ),
    /1–256/,
  );
  assert.throws(
    () =>
      arrangePlacements(["a", "b"], placements, {
        ...arrangementDraft(100000),
        spacing: "2",
      }),
    /超出/,
  );
  assert.throws(() => placementBatch([]), /1–256/);
});
test("框选四向一致且只选灯位，增减选择保序而非按文件重新排序", () => {
  assert.deepEqual(selectInBox(placements, [0, 0], [4, 4]), ["a", "b", "c"]);
  assert.deepEqual(selectInBox(placements, [4, 4], [0, 0]), ["a", "b", "c"]);
  assert.deepEqual(selectInBox(placements, [0, 4], [4, 0]), ["a", "b", "c"]);
  assert.deepEqual(togglePlacement(["c", "a"], "b", true), ["c", "a", "b"]);
  assert.deepEqual(togglePlacement(["c", "a"], "c", true), ["a"]);
  assert.deepEqual(togglePlacement(["c", "a"], "b", false), ["b"]);
});
