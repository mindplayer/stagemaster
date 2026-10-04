import test from "node:test";
import assert from "node:assert/strict";
import { functionSelectionAllowed } from "../src/fixture-function-safety.ts";
import { functionDraftValue } from "../src/function-parameter-tools.ts";
import {
  profileDraft,
  profileDefinition,
  FixtureFieldError,
} from "../src/fixture-tools.ts";
import { addFunctionChannel } from "../src/fixture-function-draft.ts";
import { controlledFunctionOptions } from "../src/fixture-emitter-functions.ts";
import { manualFixture } from "./execution-manual-fixture.ts";
import { manualChanges } from "../src/execution-manual.ts";
test("根级有限受控种类和保留的禁用资料按准确属性判断，不根据显示名称", () => {
  for (const attribute of [
    "color-wheel",
    "gobo-wheel",
    "shutter",
    "prism",
    "emitter.pattern.color-wheel",
  ])
    for (const key of [
      "sound",
      "slot-sound-1",
      "soundcontrol",
      "auto.1",
      "automatic",
      "random",
      "reset",
      "macro",
      "program-1",
    ])
      for (const mode of ["slot", "range"] as const)
        assert.equal(functionSelectionAllowed(attribute, { key, mode }), false);
  for (const [attribute, key, mode] of [
    ["color-wheel", "red", "slot"],
    ["gobo-wheel", "dots", "slot"],
    ["gobo-wheel", "shake-dots", "range"],
    ["shutter", "open", "slot"],
    ["shutter", "closed", "slot"],
    ["shutter", "strobe-slow", "range"],
    ["prism", "on", "slot"],
    ["prism", "off", "slot"],
  ] as const)
    assert(functionSelectionAllowed(attribute, { key, mode }));
  assert(
    !functionSelectionAllowed("color-wheel", { key: "custom", mode: "range" }),
  );
  assert(!functionSelectionAllowed("shutter", { key: "custom", mode: "slot" }));
});
test("根级新建不猜原生值，控制方式有界，禁用默认准确定位且取消不写原草稿", () => {
  const before = profileDraft();
  for (const attribute of ["color-wheel", "gobo-wheel", "shutter", "prism"]) {
    const draft = addFunctionChannel(before, attribute);
    assert.equal(draft.channels.at(-1)!.functions![0].dmxFrom, "");
    assert(controlledFunctionOptions(attribute).length);
    draft.channels.at(-1)!.functions![0].name = "资料待确认";
    assert.throws(
      () => profileDefinition(draft),
      (e) => e instanceof FixtureFieldError && e.field.endsWith("dmxFrom"),
    );
  }
  const draft = addFunctionChannel(before, "color-wheel"),
    c = draft.channels.at(-1)!;
  Object.assign(c.functions![0], {
    key: "automatic",
    name: "合法颜色名也不改分类",
    mode: "range",
    dmxFrom: "140",
    dmxTo: "255",
    dmxDefault: "140",
  });
  c.defaultFunction = { functionKey: "automatic", position: "0" };
  assert.throws(
    () => profileDefinition(draft),
    (e) =>
      e instanceof FixtureFieldError && e.field.endsWith("default-function"),
  );
  assert(!before.channels.some((c) => c.attribute === "color-wheel"));
});
test("场景和后台手动预检拒绝旧自动色盘，释放仍独立，受控频闪区间保持", () => {
  const r = manualFixture(),
    fixture = r.catalog.fixtures![0],
    attribute = fixture.attributes[1];
  assert.throws(
    () =>
      functionDraftValue(attribute, {
        function: { functionKey: "rotate", position: 0 },
      }),
    /已屏蔽/,
  );
  const draft = {
    targets: [fixture.id],
    attribute: "color-wheel",
    functionKey: "rotate",
    value: "50",
  };
  assert.throws(() => manualChanges(r, "manual", draft), /已屏蔽/);
  assert.deepEqual(manualChanges(r, "manual", draft, true)[0].value, {
    kind: "release",
  });
  attribute.key = "shutter";
  attribute.function!.functions = [
    {
      key: "strobe",
      name: "受控频闪",
      mode: "range",
      dmxFrom: 16,
      dmxTo: 255,
      dmxDefault: 16,
    },
  ];
  assert.deepEqual(
    manualChanges(r, "manual", {
      ...draft,
      attribute: "shutter",
      functionKey: "strobe",
    })[0].value,
    { kind: "function", functionKey: "strobe", position: 32768 },
  );
  delete attribute.function;
  assert.throws(
    () => manualChanges(r, "manual", { ...draft, attribute: "shutter" }),
    /已屏蔽/,
  );
  assert.deepEqual(
    manualChanges(r, "manual", { ...draft, attribute: "shutter" }, true)[0]
      .value,
    { kind: "release" },
  );
});
