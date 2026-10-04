import test from "node:test";
import assert from "node:assert/strict";
import {
  addAxisSpeedChannel,
  axisSpeedKey,
  axisSpeedLabel,
} from "../src/fixture-axis-speed.ts";
import {
  profileDraft,
  profileDefinition,
  FixtureFieldError,
  compatibleProfile,
} from "../src/fixture-tools.ts";
import { withMotion, withPositionModel } from "../src/profile-motion.ts";
import { withLinearFamily } from "../src/fixture-function-draft.ts";
import { parameterCategory } from "../src/parameter-categories.ts";
import {
  presetScopeOptions,
  presetScopeId,
} from "../src/preset-attribute-scopes.ts";
import { attributeName } from "../src/library-tools.ts";
import type { FixtureView } from "../src/application-host";

function readyDraft() {
  const draft = addAxisSpeedChannel(withMotion(profileDraft(), true));
  draft.channels.at(-1)!.percent = "37.5";
  return draft;
}

test("两轴速度须完整两轴且默认值显式填写，不擅自赋予零值快慢含义", () => {
  const fixed = profileDraft();
  assert.equal(addAxisSpeedChannel(fixed), fixed);
  const motion = withMotion(fixed, true);
  const draft = addAxisSpeedChannel(motion);
  assert.equal(draft.channels.at(-1)!.percent, "");
  assert.equal(addAxisSpeedChannel(draft), draft);
  assert.throws(
    () => profileDefinition(draft),
    (e) => e instanceof FixtureFieldError && e.field === "channel-6-percent",
  );
  assert.deepEqual(motion, withMotion(fixed, true));
  const valid = readyDraft();
  assert.equal(profileDefinition(valid).channels.at(-1)!.defaultValue, 24576);
});

test("两轴速度 16 位草稿精度及任意粗细往返，基础组合和几何变更不改通道", () => {
  const draft = readyDraft();
  draft.channels.at(-1)!.bits = "16";
  draft.channels.at(-1)!.fine = "14";
  draft.footprint = "14";
  for (const value of [0, 1, 257, 0x1234, 65534, 65535]) {
    const definition = profileDefinition(draft);
    definition.channels.at(-1)!.defaultValue = value;
    assert.deepEqual(profileDefinition(profileDraft(definition)), definition);
  }
  const before = structuredClone(draft);
  const rgb = withLinearFamily(draft, "rgb");
  assert.deepEqual(
    rgb.channels.find((c) => c.attribute === axisSpeedKey),
    draft.channels.at(-1),
  );
  const geometric = withPositionModel(draft, true);
  assert.deepEqual(
    withPositionModel(geometric, false).channels,
    draft.channels,
  );
  assert.deepEqual(draft, before);
  const removed = withMotion(draft, false);
  assert(
    !removed.channels.some((c) =>
      ["pan", "tilt", axisSpeedKey].includes(c.attribute),
    ),
  );
  assert.equal(removed.footprint, "14");
});

test("两轴速度非法值与粗细冲突精确定位，不接受功能区间或残缺两轴", () => {
  for (const [patch, field] of [
    [{ coarse: "1" }, "coarse"],
    [{ coarse: "99" }, "coarse"],
    [{ bits: "16", fine: "13" }, "fine"],
    [{ percent: "" }, "percent"],
    [{ percent: "-1" }, "percent"],
    [{ percent: "101" }, "percent"],
  ] as const) {
    const bad = readyDraft();
    Object.assign(bad.channels.at(-1)!, patch);
    assert.throws(
      () => profileDefinition(bad),
      (e) => e instanceof FixtureFieldError && e.field === `channel-6-${field}`,
    );
  }
  const functionDraft = readyDraft();
  functionDraft.channels.at(-1)!.functions = [];
  assert.throws(() => profileDefinition(functionDraft), /不支持功能区间/);
  for (const missing of [["pan"], ["pan", "tilt"]]) {
    const bad = readyDraft();
    bad.channels = bad.channels.filter((c) => !missing.includes(c.attribute));
    assert.throws(() => profileDefinition(bad), /须同时具备/);
  }
  const duplicate = readyDraft();
  duplicate.channels.push(structuredClone(duplicate.channels.at(-1)!));
  assert.throws(() => profileDefinition(duplicate), /只能定义一次/);
});

test("速度控制分类、中文和稀疏预设不扩大亮度或位置范围，相同属性才可换模式", () => {
  assert.equal(attributeName(axisSpeedKey), axisSpeedLabel);
  assert.equal(parameterCategory(axisSpeedKey), "control");
  const available = ["dimmer", "pan", "tilt", axisSpeedKey].map((key) => ({
    key,
  }));
  const scopes = presetScopeOptions(available);
  assert.deepEqual(scopes.find((s) => s.id === "axisSpeed")!.keys, [
    axisSpeedKey,
  ]);
  assert.deepEqual(scopes.find((s) => s.id === "position")!.keys, [
    "pan",
    "tilt",
  ]);
  assert.deepEqual(scopes.find((s) => s.id === "light")!.keys, ["dimmer"]);
  assert.equal(presetScopeId([axisSpeedKey], available), "axisSpeed");
  const definition = profileDefinition(readyDraft());
  const fixture = {
    attributes: definition.channels.map((c) => ({ key: c.attribute })),
  } as FixtureView;
  assert(compatibleProfile([fixture], definition));
  const absent = {
    ...definition,
    channels: definition.channels.filter((c) => c.attribute !== axisSpeedKey),
  };
  assert(!compatibleProfile([fixture], absent));
});
