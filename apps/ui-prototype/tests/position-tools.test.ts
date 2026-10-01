import test from "node:test";
import { withPositionModel } from "../src/profile-motion.ts";
import assert from "node:assert/strict";
import { axisReadout, positionDecimal } from "../src/position-tools.ts";
import {
  withMotion,
  profileDraft,
  profileDefinition,
  FixtureFieldError,
  compatibleProfile,
} from "../src/fixture-tools.ts";
function confirmedDraft() {
  const draft = withPositionModel(withMotion(profileDraft(), true), true);
  draft.positioning!.pan = {
    minDegrees: "-270",
    maxDegrees: "270",
    reversed: false,
  };
  draft.positioning!.tilt = {
    minDegrees: "-135",
    maxDegrees: "135",
    reversed: false,
  };
  return draft;
}
test("摇头模式独立粗细映射与物理范围往返，关闭运动保留颜色", () => {
  const draft = confirmedDraft();
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
  const d = confirmedDraft();
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
  const definition = profileDefinition(confirmedDraft());
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

test("目标平面按世界坐标向上为正，缩放和非等比视口保持点位", async () => {
  const { aimPointAt } =
    await import("../src/components/fixtures/aim-point.ts");
  const camera = { x: 5, y: 3, width: 20 },
    rect = { left: 10, top: 20, width: 400, height: 200 };
  assert.deepEqual(aimPointAt(camera, 2, rect, 210, 120), { x: 5, y: 3 });
  assert.deepEqual(aimPointAt(camera, 2, rect, 310, 70), { x: 10, y: 5.5 });
  assert.deepEqual(aimPointAt({ ...camera, width: 10 }, 2, rect, 310, 70), {
    x: 7.5,
    y: 4.25,
  });
  assert.deepEqual(aimPointAt(camera, 1, rect, 310, 20), { x: 15, y: 13 });
  assert.deepEqual(aimPointAt(camera, 2, rect, -999, 999), { x: -5, y: -2 });
  assert.equal(aimPointAt(camera, 2, { ...rect, width: 0 }, 0, 0), null);
  assert.equal(aimPointAt({ ...camera, x: 100001 }, 2, rect, 210, 120), null);
  assert.equal(aimPointAt(camera, NaN, rect, 210, 120), null);
});

test("目标坐标草稿拒绝空值与非十进制，键盘毫米精度且有界", async () => {
  const { readAimPoint, nudgeAimPoint, pointFields } =
    await import("../src/components/fixtures/aim-point.ts");
  for (const value of ["", " ", "1e3", "NaN", "100001", "1."])
    assert.equal(readAimPoint(value, "0"), null);
  assert.deepEqual(readAimPoint(" -2.25 ", "3.5"), { x: -2.25, y: 3.5 });
  assert.deepEqual(nudgeAimPoint({ x: 0, y: 0 }, "ArrowUp", false), {
    x: 0,
    y: 0.1,
  });
  assert.deepEqual(nudgeAimPoint({ x: 0.09, y: 0 }, "ArrowRight", true), {
    x: 0.1,
    y: 0,
  });
  assert.deepEqual(nudgeAimPoint({ x: 100000, y: 0 }, "ArrowRight", false), {
    x: 100000,
    y: 0,
  });
  assert.equal(nudgeAimPoint({ x: 0, y: 0 }, "Enter", false), null);
  assert.deepEqual(pointFields({ x: 1.25, y: 0 }), { x: "1.25", y: "0" });
});

test("新建两轴不猜测物理行程，显式定义前只保留通道", () => {
  const draft = withMotion(profileDraft(), true);
  assert.equal(draft.positioning, null);
  const definition = profileDefinition(draft);
  assert.equal(definition.positioning, undefined);
  assert.deepEqual(profileDefinition(profileDraft(definition)), definition);
  const withModel = withPositionModel(draft, true);
  assert.equal(withModel.positioning!.pan.minDegrees, "");
  assert.throws(
    () => profileDefinition(withModel),
    (e: unknown) =>
      e instanceof FixtureFieldError && e.field === "pan-minDegrees",
  );
  assert.deepEqual(withModel.channels, draft.channels);
  const missing = structuredClone(draft);
  missing.channels = missing.channels.filter((c) => c.attribute !== "tilt");
  assert.throws(() => profileDefinition(missing), /水平和垂直/);
});
test("开关物理模型不改粗细地址和初值，旧模型重复启用保持反向及精度", () => {
  const draft = confirmedDraft();
  draft.positioning!.pan.reversed = true;
  draft.positioning!.tilt.maxDegrees = "100.5";
  assert.deepEqual(withMotion(draft, true), draft);
  assert.deepEqual(withPositionModel(draft, true), draft);
  const unknown = withPositionModel(draft, false);
  assert.deepEqual(unknown.channels, draft.channels);
  assert.equal(unknown.positioning, null);
  assert.equal(withPositionModel(profileDraft(), true).positioning, undefined);
});
