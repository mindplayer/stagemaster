import test from "node:test";
import assert from "node:assert/strict";
import {
  currentOrderedIds,
  toggleOrderedRange,
} from "../src/components/selection/ordered-selection.ts";
import { selectionViewRange } from "../src/components/audio/selection-view.ts";
import { usageProject } from "./scene-usage-fixture.ts";
test("有序选择去重失效对象，筛选内正反范围追加保留隐藏对象", () => {
  const items = ["a", "b", "c", "d"].map((id) => ({ id }));
  assert.deepEqual(currentOrderedIds(items, ["d", "missing", "d", "a"]), [
    "a",
    "d",
  ]);
  assert.deepEqual(
    toggleOrderedRange(["d"], items.slice(0, 3), "a", "c", true),
    ["d", "a", "b", "c"],
  );
  assert.deepEqual(toggleOrderedRange(["a"], items, "d", "b", true), [
    "a",
    "b",
    "c",
    "d",
  ]);
  assert.deepEqual(toggleOrderedRange(["a"], items.slice(1), "c", "a", true), [
    "a",
    "c",
  ]);
  assert.deepEqual(toggleOrderedRange(["a"], items, "a", null, false), []);
  assert.deepEqual(toggleOrderedRange(["a"], items, "missing", "a", true), [
    "a",
  ]);
});
test("卡点组适应只覆盖卡点时间，不混入旧绑定段落或残留片段", () => {
  const track = usageProject().audio!;
  delete track.lightingClips;
  track.markers.push({ id: "late", name: "末拍", timeMs: 97000, sceneId: "s" });
  assert.deepEqual(
    selectionViewRange(track, "mark", undefined, {
      active: true,
      ids: ["late", "mark", "missing", "late"],
    }),
    { startMs: 1000, endMs: 97000, label: "2 个卡点" },
  );
  assert.deepEqual(
    selectionViewRange(track, "mark", undefined, {
      active: true,
      ids: ["mark"],
    }),
    { startMs: 1000, endMs: 1000, label: "1 个卡点" },
  );
  assert.equal(
    selectionViewRange(
      track,
      "mark",
      { active: true, ids: ["clip"] },
      { active: true, ids: [] },
    ),
    null,
  );
  assert.equal(
    selectionViewRange(track, "mark", undefined, {
      active: true,
      ids: ["missing"],
    }),
    null,
  );
  assert.equal(
    selectionViewRange(track, "mark", undefined, { active: false, ids: [] })
      ?.endMs,
    97000,
  );
});
