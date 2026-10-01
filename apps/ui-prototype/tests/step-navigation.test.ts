import { test } from "node:test";
import assert from "node:assert/strict";
import {
  navigationTargets,
  stepScrollTop,
} from "../src/components/workbench/step-navigation.ts";
import type { ExecutionPosition } from "../src/components/workbench/execution-position";
const position: ExecutionPosition = {
  epoch: 3,
  sequenceId: "list",
  currentId: "a",
  nextId: "b",
  stale: false,
  status: "running",
};
test("导航只采用同列表有效载入身份，旧版本不映射到编辑数据", () => {
  assert.deepEqual(navigationTargets(position, "list", ["a", "b"]), {
    current: "a",
    next: "b",
  });
  for (const p of [
    { ...position, stale: true },
    { ...position, sequenceId: "other" },
  ])
    assert.deepEqual(navigationTargets(p, "list", ["a", "b"]), {
      current: null,
      next: null,
    });
  assert.deepEqual(navigationTargets(position, "list", ["b"]), {
    current: null,
    next: "b",
  });
  assert.deepEqual(
    navigationTargets({ ...position, currentId: null }, "list", ["a", "b"]),
    { current: null, next: "b" },
  );
});
test("列表跟随最少滚动并尽可能同时容纳当前和下一步", () => {
  assert.equal(
    stepScrollTop(
      100,
      300,
      2000,
      { top: 120, bottom: 180 },
      { top: 180, bottom: 240 },
    ),
    100,
  );
  assert.equal(
    stepScrollTop(
      100,
      300,
      2000,
      { top: 350, bottom: 410 },
      { top: 410, bottom: 470 },
    ),
    178,
  );
  assert.equal(
    stepScrollTop(
      100,
      300,
      2000,
      { top: 350, bottom: 410 },
      { top: 900, bottom: 960 },
    ),
    118,
  );
  assert.equal(stepScrollTop(700, 300, 2000, { top: 350, bottom: 410 }), 342);
  assert.equal(
    stepScrollTop(
      1300,
      300,
      2000,
      { top: 1600, bottom: 1680 },
      { top: 0, bottom: 60 },
    ),
    1388,
  );
});
test("手动定位居中且不越界，超高行只露出起点", () => {
  assert.equal(
    stepScrollTop(0, 300, 2000, { top: 600, bottom: 660 }, undefined, true),
    480,
  );
  assert.equal(
    stepScrollTop(0, 300, 2000, { top: 0, bottom: 60 }, undefined, true),
    0,
  );
  assert.equal(
    stepScrollTop(0, 300, 2000, { top: 1940, bottom: 2000 }, undefined, true),
    1700,
  );
  assert.equal(stepScrollTop(0, 300, 2000, { top: 600, bottom: 1000 }), 592);
  assert.equal(stepScrollTop(0, 300, 100, { top: 0, bottom: 60 }), 0);
});
