import test from "node:test";
import assert from "node:assert/strict";
import {
  profileDraft,
  profileDefinition,
  profileChannelLabel,
  compatibleProfile,
  FixtureFieldError,
} from "../src/fixture-tools.ts";
import {
  addEmitter,
  withEmitterFamily,
  removeEmitter,
} from "../src/fixture-emitters.ts";
import {
  addFunctionChannel,
  withLinearFamily,
} from "../src/fixture-function-draft.ts";
import { newControlledFunction } from "../src/fixture-emitter-functions.ts";
import { addSlotBatch } from "../src/fixture-slot-batch.ts";
import { presetScopeOptions } from "../src/preset-attribute-scopes.ts";
import { parameterCategory } from "../src/parameter-categories.ts";
import { profileExchangeReview } from "../src/profile-exchange-review.ts";
import type { FixtureView, ProjectView } from "../src/application-host";
import type { ProfileView } from "../src/fixture-types";
function draft() {
  let d = addEmitter(withLinearFamily(profileDraft(), "dimmer"));
  d = withEmitterFamily(addEmitter(d), "unit2", "rgbw");
  d.emitters![0].name = "图案光源";
  d.emitters![1].name = "染色光源";
  d.channels.forEach((c) => (c.percent = "100"));
  for (const owner of ["unit1", "unit2"]) {
    const key = `emitter.${owner}.shutter`;
    d = addFunctionChannel(d, key);
    const c = d.channels.at(-1)!;
    Object.assign(c.functions![0], {
      dmxFrom: "0",
      dmxTo: "15",
      dmxDefault: "0",
    });
    c.functions!.push({
      ...newControlledFunction(key, "strobe"),
      dmxFrom: "16",
      dmxTo: "255",
      dmxDefault: "16",
    });
  }
  d = addFunctionChannel(d, "emitter.unit1.color-wheel");
  const c = d.channels.at(-1)!;
  Object.assign(c.functions![0], {
    dmxFrom: "0",
    dmxTo: "15",
    dmxDefault: "0",
  });
  d.channels[d.channels.length - 1] = addSlotBatch(
    c,
    { start: "16", width: "16", count: "2", name: "未知色片" },
    "color",
  );
  return d;
}
test("独立双快门和轮盘精确保存，功能值与中文归属无损", () => {
  const d = draft(),
    p = profileDefinition(d);
  assert.deepEqual(profileDefinition(profileDraft(p)), p);
  assert.equal(
    profileChannelLabel(d, "emitter.unit2.shutter"),
    "染色光源 · 快门与频闪",
  );
  assert(p.channels.at(-1)!.functions![1].key.startsWith("slot-"));
  assert.deepEqual(p.channels[6].defaultValue, {
    functionKey: "open",
    position: 0,
  });
});
test("新单元区间不猜原生值，缺值定位；声控自走复位和错误控制方式拒绝", () => {
  const d = addFunctionChannel(draft(), "emitter.unit2.gobo-wheel");
  assert.throws(
    () => profileDefinition(d),
    (e) => e instanceof FixtureFieldError && e.field.endsWith("dmxFrom"),
  );
  for (const key of ["sound-0", "auto-0", "reset", "random", "function-any"]) {
    const d = draft();
    d.channels[6].functions![1].key = key;
    assert.throws(() => profileDefinition(d), FixtureFieldError);
  }
  const bad = draft();
  bad.channels[6].functions![1].mode = "slot";
  assert.throws(
    () => profileDefinition(bad),
    (e) => e instanceof FixtureFieldError && e.field.endsWith("mode"),
  );
});
test("切换连续组合保留功能，取消/移除只改原单元草稿", () => {
  const d = draft(),
    before = structuredClone(d);
  const next = withEmitterFamily(d, "unit1", "rgbw");
  for (const c of d.channels.filter((c) => c.functions))
    assert.deepEqual(
      next.channels.find((n) => n.attribute === c.attribute),
      c,
    );
  assert.deepEqual(d, before);
  assert(
    !removeEmitter(next, "unit1").channels.some((c) =>
      c.attribute.startsWith("emitter.unit1."),
    ),
  );
  assert(
    withLinearFamily(d, "none").channels.some(
      (c) => c.attribute === "emitter.unit1.shutter",
    ),
  );
});
test("选择分类和稀疏预设保留独立快门目标，不附加亮度", () => {
  const available = draft().channels.map((c) => ({ key: c.attribute }));
  assert.equal(parameterCategory("emitter.unit1.color-wheel"), "color");
  assert.equal(parameterCategory("emitter.unit2.shutter"), "intensity");
  assert.deepEqual(
    presetScopeOptions(available).find((s) => s.id === "shutter")!.keys,
    ["emitter.unit1.shutter", "emitter.unit2.shutter"],
  );
  assert.deepEqual(
    presetScopeOptions(available).find((s) => s.id === "light")!.keys,
    ["dimmer", "emitter.unit1.dimmer"],
  );
});
test("单元色盘外观与显式重映射审阅区分准确单元，不猜跨光源映射", () => {
  const d = draft(),
    p = profileDefinition(d);
  p.channels.at(-1)!.functions![1].appearance = {
    kind: "color",
    colors: ["#FF0000"],
  };
  const target = {
    ...structuredClone(p),
    id: "new",
    revision: "2",
    authorable: true,
  } as ProfileView;
  target.channels.at(-1)!.functions![1].dmxDefault = 20;
  const fixture = {
    id: "f",
    name: "灯甲",
    profileId: "old",
    profileName: "原模式",
    attributes: p.channels.map((c) => ({
      key: c.attribute,
      label: profileChannelLabel(d, c.attribute),
      defaultValue: 0,
      ...(c.functions
        ? {
            function: {
              functions: c.functions,
              fine: false,
              default: c.defaultValue,
            },
          }
        : {}),
    })),
  } as FixtureView;
  assert(!compatibleProfile([fixture], target));
  assert(compatibleProfile([fixture], target, true));
  const review = profileExchangeReview(
    { profiles: [{ ...p, id: "old", revision: "1" }, target] } as ProjectView,
    [fixture],
    target,
  );
  assert.equal(
    review.groups[0].changes[0].attribute,
    "emitter.unit1.color-wheel",
  );
  assert.equal(review.groups[0].changes[0].label, "图案光源 · 色盘");
  assert(review.needsRemap);
  target.channels.at(-1)!.attribute = "emitter.unit2.color-wheel";
  assert(!compatibleProfile([fixture], target, true));
});
test("图案抖动仅图案盘可用，固定档外观不能混入受控频闪", () => {
  const d = addFunctionChannel(draft(), "emitter.unit1.gobo-wheel"),
    c = d.channels.at(-1)!;
  Object.assign(c.functions![0], { dmxFrom: "0", dmxTo: "7", dmxDefault: "0" });
  c.functions!.push({
    ...newControlledFunction(c.attribute, "shake"),
    dmxFrom: "119",
    dmxTo: "127",
    dmxDefault: "119",
  });
  assert.doesNotThrow(() => profileDefinition(d));
  assert.throws(() =>
    newControlledFunction("emitter.unit1.color-wheel", "shake"),
  );
  d.channels[6].functions![1].appearance = { kind: "open" };
  assert.throws(() => profileDefinition(d));
});
