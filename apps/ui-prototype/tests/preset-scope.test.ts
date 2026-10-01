import {
  presetScopeOptions,
  presetScopeId,
} from "../src/preset-attribute-scopes.ts";
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

test("共享颜色范围同时覆盖 RGB 与色盘，亮度动作始终与快门隔离", () => {
  const available = [
    "dimmer",
    "shutter",
    "red",
    "green",
    "blue",
    "color-wheel",
    "gobo-wheel",
    "prism",
    "zoom",
    "focus",
    "iris",
    "pan",
    "tilt",
    "legacy",
  ].map((key) => ({ key }));
  const scopes = presetScopeOptions(available);
  const keys = (id: string) => scopes.find((s) => s.id === id)!.keys;
  assert.deepEqual(keys("color"), ["red", "green", "blue", "color-wheel"]);
  assert.deepEqual(keys("light"), ["dimmer"]);
  assert.deepEqual(keys("shutter"), ["shutter"]);
  assert.deepEqual(keys("beam"), ["gobo-wheel", "prism"]);
  assert.deepEqual(keys("optics"), ["zoom", "focus", "iris"]);
  assert.deepEqual(keys("position"), ["pan", "tilt"]);
  assert.ok(keys("all").includes("legacy"));
  assert.equal(
    presetScopeId(["iris", "focus", "zoom", "zoom"], available),
    "optics",
  );
  assert.equal(presetScopeId(["legacy"], available), "custom");
  assert.equal(presetScopeId(["dimmer", "shutter"], available), "custom");
});

test("快捷范围仅选当前能力，能力变化不扩展显式掩码且空选保持明确", () => {
  const wheel = [{ key: "dimmer" }, { key: "color-wheel" }];
  const mask = presetScopeOptions(wheel).find((s) => s.id === "color")!.keys;
  assert.deepEqual(mask, ["color-wheel"]);
  const mixed = [...wheel, { key: "red" }, { key: "green" }, { key: "blue" }];
  assert.deepEqual(initialPresetAttributes(mixed, mask), ["color-wheel"]);
  assert.equal(presetScopeId(mask, mixed), "custom");
  assert.equal(presetScopeId(mask, [{ key: "dimmer" }]), "custom");
  for (const scope of [[], ["missing"]])
    assert.equal(presetScopeId(scope, mixed), "custom");
  assert.equal(presetScopeId(null, []), "all");
  assert.equal(presetScopeId([], []), "custom");
  assert.deepEqual(
    presetScopeOptions(wheel).find((s) => s.id === "optics")!.keys,
    [],
  );
  assert.deepEqual(presetScopeOptions([...wheel, wheel[0]])[0].keys, [
    "dimmer",
    "color-wheel",
  ]);
  mask.push("red");
  assert.deepEqual(
    presetScopeOptions(wheel).find((s) => s.id === "color")!.keys,
    ["color-wheel"],
  );
});
