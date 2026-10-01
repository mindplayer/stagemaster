import test from "node:test";
import assert from "node:assert/strict";
import type { FixturePlacement } from "../src/stage-types.ts";
import {
  arrangeSpatialFixtureIds,
  fixtureAxisPositions,
} from "../src/spatial-fixture-order.ts";
function place(
  fixtureId: string,
  x: string,
  y: string,
  z: string,
): FixturePlacement {
  return {
    fixtureId,
    spaceId: null,
    positionMeters: { x, y, z },
    rotationDegreesXYZ: { x: "0", y: "0", z: "0" },
  };
}
const positions = [
  place("a", "-2", "5", "6"),
  place("b", "-2", "-1", "0.5"),
  place("c", "10", "0", "3"),
  place("bad", "NaN", "", "Infinity"),
];
const ids = ["unknown", "b", "c", "bad", "a", "unplaced"];
test("空间排序区分三个世界轴、负坐标和方向，不按字符串或其他轴破坏相同坐标灯序", () => {
  assert.deepEqual(arrangeSpatialFixtureIds(ids, positions, "x", "ascending"), [
    "b",
    "a",
    "c",
    "unknown",
    "bad",
    "unplaced",
  ]);
  assert.deepEqual(
    arrangeSpatialFixtureIds(ids, positions, "x", "descending"),
    ["c", "b", "a", "unknown", "bad", "unplaced"],
  );
  assert.deepEqual(arrangeSpatialFixtureIds(ids, positions, "y", "ascending"), [
    "b",
    "c",
    "a",
    "unknown",
    "bad",
    "unplaced",
  ]);
  assert.deepEqual(
    arrangeSpatialFixtureIds(ids, positions, "y", "descending"),
    ["a", "c", "b", "unknown", "bad", "unplaced"],
  );
  assert.deepEqual(arrangeSpatialFixtureIds(ids, positions, "z", "ascending"), [
    "b",
    "c",
    "a",
    "unknown",
    "bad",
    "unplaced",
  ]);
  assert.deepEqual(
    arrangeSpatialFixtureIds(ids, positions, "z", "descending"),
    ["a", "c", "b", "unknown", "bad", "unplaced"],
  );
});
test("成员、几何和原序不被修改；无位置、空组、重复成员不静默删除，反复排序幂等", () => {
  const before = structuredClone(positions);
  const result = arrangeSpatialFixtureIds(ids, positions, "x", "ascending");
  assert.deepEqual(
    arrangeSpatialFixtureIds(result, positions, "x", "ascending"),
    result,
  );
  assert.deepEqual(positions, before);
  assert.deepEqual(ids, ["unknown", "b", "c", "bad", "a", "unplaced"]);
  assert.deepEqual(arrangeSpatialFixtureIds(ids, [], "z", "descending"), ids);
  assert.deepEqual(
    arrangeSpatialFixtureIds([], positions, "x", "ascending"),
    [],
  );
  assert.deepEqual(
    arrangeSpatialFixtureIds(["c", "b", "b"], positions, "x", "ascending"),
    ["b", "b", "c"],
  );
  assert.notEqual(result, ids);
});
test("仅有限有效米值可参与排序，零不是缺失，位置改变只在下次显式排序生效", () => {
  const samples = [
    "",
    " ",
    "NaN",
    "Infinity",
    "1e3",
    "100001",
    "-100001",
    "1.0000001",
    "0",
    " -0.000001 ",
    "100000",
    "-100000",
  ];
  const placed = samples.map((value, i) => place(String(i), value, "0", "0"));
  assert.deepEqual(
    [...fixtureAxisPositions(placed, "x").keys()],
    ["8", "9", "10", "11"],
  );
  const authoring = arrangeSpatialFixtureIds(
    ["c", "a"],
    positions,
    "x",
    "ascending",
  );
  const relocated = positions.map((p) =>
    p.fixtureId === "a" ? place("a", "20", "0", "0") : p,
  );
  assert.deepEqual(authoring, ["a", "c"]);
  assert.deepEqual(
    arrangeSpatialFixtureIds(authoring, relocated, "x", "ascending"),
    ["c", "a"],
  );
});
