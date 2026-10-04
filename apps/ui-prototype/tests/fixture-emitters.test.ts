import test from "node:test";
import assert from "node:assert/strict";
import {
  profileDraft,
  profileDefinition,
  profileChannelLabel,
  compatibleProfile,
  FixtureFieldError,
} from "../src/fixture-tools.ts";
import { withLinearFamily } from "../src/fixture-function-draft.ts";
import {
  addEmitter,
  withEmitterFamily,
  removeEmitter,
} from "../src/fixture-emitters.ts";
import { parameterCategory } from "../src/parameter-categories.ts";
import { presetScopeOptions } from "../src/preset-attribute-scopes.ts";
import type { FixtureView } from "../src/application-host";
function draft() {
  let d = addEmitter(withLinearFamily(profileDraft(), "dimmer"));
  d = withEmitterFamily(addEmitter(d), "unit2", "rgbw");
  d.emitters![0].name = "图案光源";
  d.emitters![1].name = "染色光源";
  d.channels.forEach((c) => (c.percent = "37.5"));
  return d;
}
test("独立光源草稿无损保存精确值、名称和白光，复制保持稳定标识", () => {
  const d = draft(),
    p = profileDefinition(d);
  assert.deepEqual(profileDefinition(profileDraft(p)), p);
  assert.equal(p.channels.at(-1)!.attribute, "emitter.unit2.white");
  assert.equal(
    profileChannelLabel(d, p.channels.at(-1)!.attribute),
    "染色光源 · 白光",
  );
  assert.equal(profileChannelLabel(d, "dimmer"), "总亮度");
  assert.deepEqual(p.emitters, d.emitters);
});
test("新增默认值留空并定位，非法归属、单位标识和组合拒绝", () => {
  const d = addEmitter(withLinearFamily(profileDraft(), "dimmer"));
  assert.throws(
    () => profileDefinition(d),
    (e) => e instanceof FixtureFieldError && e.field.endsWith("percent"),
  );
  for (const change of [
    (d: ReturnType<typeof draft>) => (d.emitters![1].key = "unit1"),
    (d: ReturnType<typeof draft>) => (d.emitters![0].name = " "),
    (d: ReturnType<typeof draft>) => {
      d.channels.pop();
      d.channels.pop();
    },
    (d: ReturnType<typeof draft>) =>
      (d.channels[1].attribute = "emitter.unknown.dimmer"),
    (d: ReturnType<typeof draft>) =>
      (d.channels[1].attribute = "emitter.unit1.fixture-program"),
  ]) {
    const d = draft();
    change(d);
    assert.throws(() => profileDefinition(d), FixtureFieldError);
  }
});
test("总调光可选，移除光源和切换组合只改草稿，不隐式写入工程", () => {
  const original = draft(),
    snapshot = structuredClone(original);
  const noMaster = withLinearFamily(original, "none");
  assert.equal(profileDefinition(noMaster).channels.length, 5);
  assert.deepEqual(noMaster.emitters, original.emitters);
  assert.equal(
    profileDefinition(removeEmitter(original, "unit2")).emitters!.length,
    1,
  );
  assert.deepEqual(original, snapshot);
  assert.equal(
    addEmitter(profileDraft()).emitters,
    undefined,
    "不会静默删除根级 RGB",
  );
});
test("参数分类及稀疏预设保留准确光源目标，不扩大到轴或其他属性", () => {
  const available = draft().channels.map((c) => ({ key: c.attribute }));
  assert.equal(parameterCategory("emitter.unit2.white"), "color");
  assert.equal(parameterCategory("emitter.unit1.dimmer"), "intensity");
  assert.deepEqual(
    presetScopeOptions(available).find((s) => s.id === "light")!.keys,
    ["dimmer", "emitter.unit1.dimmer"],
  );
  assert.deepEqual(
    presetScopeOptions(available).find((s) => s.id === "color")!.keys,
    [
      "emitter.unit2.red",
      "emitter.unit2.green",
      "emitter.unit2.blue",
      "emitter.unit2.white",
    ],
  );
});
test("相同名称不同单元身份不能隐式换模式，显示改名不重绑定", () => {
  const p = profileDefinition(draft());
  const fixtures = [
    { attributes: p.channels.map((c) => ({ key: c.attribute })) },
  ] as FixtureView[];
  assert(compatibleProfile(fixtures, p));
  p.emitters![0].name = "新的图案光源名";
  assert(compatibleProfile(fixtures, p));
  p.channels[1].attribute = "emitter.other.dimmer";
  assert(!compatibleProfile(fixtures, p));
});
