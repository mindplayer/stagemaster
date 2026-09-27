import test from "node:test";
import assert from "node:assert/strict";
import { axisReadout, positionDecimal } from "../src/position-tools.ts";
import {
  withMotion,
  profileDraft,
  profileDefinition,
  FixtureFieldError,
  compatibleProfile,
} from "../src/fixture-tools.ts";
test("摇头模式独立粗细映射与物理范围往返，关闭运动保留颜色", () => {
  const draft = withMotion(profileDraft(), true);
  assert.equal(draft.footprint, "8");
  assert.deepEqual(
    draft.channels.slice(-2).map((c) => [c.coarse, c.fine]),
    [
      ["5", "6"],
      ["7", "8"],
    ],
  );
  draft.positioning!.pan.reversed = true;
  draft.positioning!.pan.minDegrees = "-200.5";
  const d = profileDefinition(draft);
  assert.deepEqual(profileDefinition(profileDraft(d)), d);
  const fixed = withMotion(draft, false);
  assert.deepEqual(
    fixed.channels.map((c) => c.attribute),
    ["dimmer", "red", "green", "blue"],
  );
  assert.equal(profileDefinition(fixed).positioning, undefined);
  assert.equal(profileDraft().channels.length, 4);
});
test("物理角范围、缺轴与十进制错误定位，拒绝非有限值", () => {
  const d = withMotion(profileDraft(), true);
  d.positioning!.tilt.maxDegrees = d.positioning!.tilt.minDegrees;
  assert.throws(
    () => profileDefinition(d),
    (e: unknown) =>
      e instanceof FixtureFieldError && e.field === "tilt-maxDegrees",
  );
  for (const raw of ["", "NaN", "Infinity", "1e4", "361", "-361"])
    assert.throws(() => positionDecimal(raw, "panZero", "零偏", 360));
  assert.equal(positionDecimal(" -1.25 ", "x", "目标", 100000), "-1.25");
});
test("轴角读数使用实际高字节与反向定义，八位步进不会假装十六位精度", () => {
  const a = { minDegrees: "-270", maxDegrees: "270", reversed: true };
  assert.equal(axisReadout(a, 0, true), 270);
  assert.equal(axisReadout(a, 65535, true), -270);
  assert.equal(axisReadout(a, 0x8000, false), axisReadout(a, 0x80ff, false));
  assert.notEqual(axisReadout(a, 0x8000, true), axisReadout(a, 0x80ff, true));
});

test("换灯候选不能把已记录的位置解释为另一种运动映射", () => {
  const definition = profileDefinition(withMotion(profileDraft(), true));
  const f = {
    positioning: structuredClone(definition.positioning),
    attributes: definition.channels.map((c) => ({ key: c.attribute })),
  } as import("../src/application-host.ts").FixtureView;
  assert.equal(compatibleProfile([f], definition), true);
  definition.positioning!.pan.reversed = true;
  assert.equal(compatibleProfile([f], definition), false);
  definition.positioning!.pan.reversed = false;
  definition.positioning!.tilt.minDegrees = "-90";
  assert.equal(compatibleProfile([f], definition), false);
});
