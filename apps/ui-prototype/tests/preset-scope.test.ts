import test from "node:test";
import assert from "node:assert/strict";
import { initialPresetAttributes } from "../src/preset-scope.ts";
import { attributeName, matchingValues } from "../src/library-tools.ts";
import type { PresetView } from "../src/library-types";

const available = ["dimmer", "zoom", "focus", "iris"].map((key) => ({ key }));
const preset: PresetView = {
  id: "p",
  name: "镜头",
  usedByScenes: [],
  usedBySequences: [],
  values: [
    {
      fixtureId: "a",
      attribute: "zoom",
      value: 26214,
      mode: "value",
      presetId: null,
      presetName: null,
    },
    {
      fixtureId: "b",
      attribute: "iris",
      value: 52428,
      mode: "value",
      presetId: null,
      presetName: null,
    },
  ],
};
test("预设未限定范围保持旧默认，显式范围不因空选或能力变化而扩大", () => {
  assert.deepEqual(initialPresetAttributes(available, null), [
    "dimmer",
    "zoom",
    "focus",
    "iris",
  ]);
  assert.deepEqual(initialPresetAttributes(available, null, preset), [
    "zoom",
    "iris",
  ]);
  for (const mask of [[], ["red"]]) {
    assert.deepEqual(initialPresetAttributes(available, mask), []);
    assert.deepEqual(initialPresetAttributes(available, mask, preset), []);
  }
  assert.deepEqual(
    initialPresetAttributes(available, ["focus", "focus", "red"], preset),
    ["focus"],
  );
  const mask = ["iris", "zoom"];
  const values = initialPresetAttributes(
    [...available, available[1]],
    mask,
    preset,
  );
  values.push("dimmer");
  assert.deepEqual(mask, ["iris", "zoom"]);
  assert.equal(available.length, 4);
});
test("镜头记录范围和内容摘要保持一致，取消不影响源值，所有光学属性有中文名称", () => {
  const before = structuredClone(preset);
  const attributes = initialPresetAttributes(available, ["zoom"]);
  assert.deepEqual(matchingValues(preset.values, ["a", "b"], attributes), [
    preset.values[0],
  ]);
  assert.deepEqual(preset, before);
  assert.deepEqual(["zoom", "focus", "iris"].map(attributeName), [
    "变焦",
    "调焦",
    "光圈",
  ]);
  assert.deepEqual(initialPresetAttributes([], null, preset), []);
});
