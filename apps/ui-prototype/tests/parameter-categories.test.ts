import test from "node:test";
import assert from "node:assert/strict";
import {
  availableParameterCategories,
  parameterCategory,
  visibleParameterAttributes,
} from "../src/parameter-categories.ts";
import { commonAttributes, parameterCommands } from "../src/editor-tools.ts";
import type { FixtureView } from "../src/application-host";

function fixture(keys: string[]): FixtureView {
  return {
    id: "a",
    name: "灯",
    profileId: "p",
    profileName: "模式",
    domainId: "d",
    domainName: "域",
    universe: 1,
    address: 1,
    footprint: keys.length,
    attributes: keys.map((key) => ({ key, label: key, defaultValue: 0 })),
  };
}
test("混合选灯按共同能力分类且不改变实际属性命令或原始顺序", () => {
  const a = fixture(["focus", "dimmer", "red", "green", "blue", "iris"]);
  const b = { ...fixture(["iris", "focus", "dimmer"]), id: "b" };
  const before = structuredClone([a, b]);
  const common = commonAttributes([a, b]);
  assert.deepEqual(
    availableParameterCategories(common).map((c) => [c.id, c.count]),
    [
      ["intensity", 1],
      ["optics", 2],
    ],
  );
  assert.deepEqual(
    common.map((a) => a.key),
    ["focus", "dimmer", "iris"],
  );
  assert.deepEqual(
    parameterCommands("s", [a, b], { focus: "50" }).map((c) => [
      c.op,
      "attribute" in c && c.attribute,
    ]),
    [
      ["setSceneValue", "focus"],
      ["setSceneValue", "focus"],
    ],
  );
  assert.deepEqual([a, b], before);
  assert.deepEqual(
    visibleParameterAttributes(a.attributes, "color").map((a) => a.key),
    ["red", "green", "blue"],
  );
  assert.deepEqual(
    visibleParameterAttributes(common, "all").map((a) => a.key),
    ["dimmer", "focus", "iris"],
  );
});
test("未分类兼容属性仍可编辑，空选择无入口，功能不兼容不会假称共同属性", () => {
  const a = fixture(["zoom", "custom-legacy", "pan", "color-wheel"]);
  const spec = {
    functions: [
      {
        key: "open",
        name: "白光",
        mode: "slot" as const,
        dmxFrom: 0,
        dmxTo: 255,
        dmxDefault: 0,
      },
    ],
    default: { functionKey: "open", position: 0 },
    fine: false,
  };
  a.attributes[3].function = spec;
  const b = structuredClone(a);
  b.id = "b";
  b.attributes[3].function!.functions[0].dmxTo = 250;
  const common = commonAttributes([a, b]);
  assert.deepEqual(
    availableParameterCategories(common).map((c) => [c.id, c.count]),
    [
      ["optics", 1],
      ["other", 2],
    ],
  );
  assert.equal(parameterCategory("pan"), "other");
  assert.deepEqual(
    visibleParameterAttributes(common, "all").map((a) => a.key),
    ["zoom", "custom-legacy", "pan"],
  );
  assert.deepEqual(availableParameterCategories(commonAttributes([])), []);
});
