import test from "node:test";
import assert from "node:assert/strict";
import { addOpticsChannel } from "../src/fixture-optics.ts";
import {
  profileDraft,
  profileDefinition,
  withMotion,
  FixtureFieldError,
} from "../src/fixture-tools.ts";
import { withLinearFamily } from "../src/fixture-function-draft.ts";
test("镜头草稿支持任意粗细和默认值往返，更换基础组合保留镜头映射", () => {
  let draft = profileDraft();
  for (const key of ["zoom", "focus", "iris"])
    draft = addOpticsChannel(draft, key);
  draft.channels[4] = {
    ...draft.channels[4],
    bits: "16",
    fine: "8",
    percent: "7.110704",
  };
  draft.footprint = "8";
  const definition = profileDefinition(draft);
  assert.deepEqual(profileDefinition(profileDraft(definition)), definition);
  const old = structuredClone(draft);
  const rgb = withLinearFamily(draft, "rgb");
  const motion = withMotion(rgb, true);
  assert.deepEqual(
    profileDefinition(motion).channels.filter((c) =>
      ["zoom", "focus", "iris"].includes(c.attribute),
    ),
    definition.channels.slice(4),
  );
  assert.deepEqual(draft, old);
  assert.equal(addOpticsChannel(draft, "zoom"), draft);
  assert.equal(addOpticsChannel(draft, "reset"), draft);
});
test("镜头映射非法范围及重复值定位到准确输入框，不隐式压缩通道占用", () => {
  const draft = addOpticsChannel(profileDraft(), "zoom");
  for (const [patch, field] of [
    [{ coarse: "1" }, "coarse"],
    [{ percent: "101" }, "percent"],
    [{ bits: "16", fine: "5" }, "fine"],
  ] as const) {
    const bad = structuredClone(draft);
    Object.assign(bad.channels[4], patch);
    assert.throws(
      () => profileDefinition(bad),
      (e) => e instanceof FixtureFieldError && e.field === `channel-4-${field}`,
    );
  }
  const bad = structuredClone(draft);
  bad.channels[4].functions = [];
  assert.throws(() => profileDefinition(bad), /不支持功能区间/);
  const removed = { ...draft, channels: draft.channels.slice(0, 4) };
  assert.equal(profileDefinition(removed).footprint, 5);
});
