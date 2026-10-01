import test from "node:test";
import assert from "node:assert/strict";
import {
  orderedStepIds,
  toggleStepRange,
  addedStepIds,
} from "../src/sequence-group-tools.ts";
import type { StepView } from "../src/sequence-types";
const steps = ["a", "b", "c", "d"].map((id) => ({
  id,
  name: id,
  number: id,
  sceneId: "scene",
  delayMs: 0,
  fadeMs: 0,
  waitMs: null,
})) satisfies StepView[];
test("步骤组始终按执行顺序、去重并剔除失效身份", () => {
  assert.deepEqual(orderedStepIds(steps, ["d", "a", "a", "removed"]), [
    "a",
    "d",
  ]);
});
test("Shift 范围只扩选筛选中的步骤，保留隐藏选择", () => {
  assert.deepEqual(
    toggleStepRange(["b"], [steps[0], steps[2], steps[3]], "a", "d", true),
    ["b", "a", "c", "d"],
  );
  assert.deepEqual(
    toggleStepRange(["b"], [steps[0], steps[2]], "a", "b", true),
    ["b", "a"],
  );
  assert.deepEqual(toggleStepRange(["b"], steps, "b", "a", false), []);
  assert.deepEqual(toggleStepRange(["b"], steps, "gone", "a", true), ["b"]);
});
test("复制仅选择宿主返回的新身份，移动和撤销不产生虚构步骤", () => {
  const after = [
    steps[0],
    { ...steps[0], id: "copyA" },
    steps[1],
    { ...steps[1], id: "copyB" },
    steps[2],
  ];
  assert.deepEqual(addedStepIds(steps, after), ["copyA", "copyB"]);
  assert.deepEqual(addedStepIds(steps, [...steps].reverse()), []);
  assert.deepEqual(orderedStepIds(steps, addedStepIds(steps, after)), []);
});
