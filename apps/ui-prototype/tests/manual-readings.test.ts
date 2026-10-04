import test from "node:test";
import assert from "node:assert/strict";
import {
  manualReading,
  manualRows,
  selectedManualReading,
} from "../src/manual-readings.ts";
import { manualFixture } from "./execution-manual-fixture.ts";

test("设定值来自实际快照，零电平不改原值，未持有不同于零", () => {
  const runtime = manualFixture(),
    fixtures = runtime.catalog.fixtures!;
  const state = runtime.observation.snapshot!.state.sources.find(
    (s) => s.id === "manual",
  )!;
  state.level = 0;
  state.held = fixtures
    .slice(0, 2)
    .map((f) => ({ fixtureId: f.id, attribute: "dimmer" }));
  state.heldValues = [24576, 24576];
  const rows = manualRows(fixtures, state)!;
  assert.equal(rows[0].reading.label, "37.5%");
  const selected = fixtures.slice(0, 3).map((f) => f.id);
  assert.deepEqual(selectedManualReading(rows, selected, "dimmer"), {
    held: 2,
    unheld: 1,
    label: "37.5%",
    raw: 24576,
  });
  state.heldValues = [24576, 0];
  assert.equal(
    selectedManualReading(manualRows(fixtures, state)!, selected, "dimmer")
      .label,
    "多个不同值",
  );
  assert.equal(
    selectedManualReading(rows, selected, "color-wheel").label,
    "未持有",
  );
  state.heldValues = [0, 0];
  assert.equal(
    selectedManualReading(manualRows(fixtures, state)!, selected, "dimmer")
      .label,
    "0%",
  );
  // Already-read data remains independent of later incoming observations.
  assert.equal(rows[0].reading.raw, 24576);
});
test("每个灯具用自己的功能档案，显示量化后的区间位置并保留精确值", () => {
  const fixtures = manualFixture().catalog.fixtures!;
  const standard = fixtures[0].attributes[1],
    custom = fixtures[2].attributes[1];
  assert.equal(manualReading(standard, 20 * 257)!.label, "红色");
  assert.equal(manualReading(custom, 20 * 257)!.label, "绿色");
  assert.equal(manualReading(standard, 144 * 257)!.label, "旋转 · 50.22%");
  assert.equal(manualReading(standard, 20 * 257 + 1), null);
  assert.equal(manualReading(standard, 19 * 257), null);
  const fine = structuredClone(standard);
  fine.function!.fine = true;
  assert.equal(manualReading(fine, 20)!.label, "红色");
  assert.equal(manualReading(fine, 255)!.label, "旋转 · 100%");
  for (const value of [NaN, -1, 65536, 1.5])
    assert.equal(manualReading(standard, value), null);
});
test("缺少值、错配或非法目标不能假装成空持有或部分有效状态", () => {
  const runtime = manualFixture(),
    fixtures = runtime.catalog.fixtures!;
  const state = runtime.observation.snapshot!.state.sources.find(
    (s) => s.id === "manual",
  )!;
  delete state.heldValues;
  assert.equal(manualRows(fixtures, state), null);
  state.heldValues = [];
  assert.deepEqual(manualRows(fixtures, state), []);
  state.heldValues = [0];
  assert.equal(manualRows(fixtures, state), null);
  state.held = [{ fixtureId: "missing", attribute: "dimmer" }];
  assert.equal(manualRows(fixtures, state), null);
});
