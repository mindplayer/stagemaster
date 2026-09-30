import { test } from "node:test";
import assert from "node:assert/strict";
import {
  createEffect,
  effectCommands,
  reorderEffect,
  supportsEffect,
} from "../src/effect-tools.ts";
import type { FixtureView } from "../src/application-host.ts";
const fixture = (id: string, keys: string[]): FixtureView => ({
  id,
  name: id,
  profileName: "测试",
  domainName: "灯光",
  domainId: "d",
  footprint: keys.length,
  universe: 1,
  address: 1,
  attributes: keys.map((key) => ({ key, label: key, defaultValue: 0 })),
});
const fixtures = [
  fixture("rgb", ["dimmer", "red", "green", "blue"]),
  fixture("dim", ["dimmer"]),
];
test("mixed fixture selections reject unsupported color effects without silently dropping members", () => {
  assert.equal(supportsEffect(fixtures, "color"), false);
  assert.equal(supportsEffect(fixtures, "breathe"), true);
  assert.equal(supportsEffect([], "breathe"), false);
  assert.throws(
    () =>
      effectCommands(
        "s",
        createEffect("color", "id", ["rgb", "dim"]),
        fixtures,
        true,
      ),
    /dim/,
  );
});
test("authoring order remains independent of current selection and moving one item preserves membership", () => {
  const ids = ["b", "a", "c"];
  const e = createEffect("chase", "id", ids);
  ids.reverse();
  assert.deepEqual(e.fixtureIds, ["b", "a", "c"]);
  assert.deepEqual(reorderEffect(e.fixtureIds, 0, 1), ["a", "b", "c"]);
  assert.deepEqual(reorderEffect(e.fixtureIds, 0, -1), e.fixtureIds);
  assert.deepEqual(e.fixtureIds, ["b", "a", "c"]);
});
test("color illumination is explicit and shares one transaction with the effect", () => {
  const e = createEffect("color", "id", ["rgb"]);
  const without = effectCommands("s", e, fixtures, false);
  assert.equal(without.length, 1);
  const withLight = effectCommands("s", e, fixtures, true);
  assert.equal(withLight.length, 2);
  assert.deepEqual(withLight[1], {
    op: "setSceneValue",
    sceneId: "s",
    fixtureId: "rgb",
    attribute: "dimmer",
    mode: "literal",
    value: 65535,
  });
  assert.throws(
    () => effectCommands("s", { ...e, fixtureIds: [] }, fixtures, false),
    /至少/,
  );
  assert.throws(
    () =>
      effectCommands(
        "s",
        { ...e, fixtureIds: ["rgb", "rgb"] },
        fixtures,
        false,
      ),
    /重复/,
  );
});

import { reuseEffect } from "../src/effect-tools.ts";
import {
  appendFrame,
  evenFrames,
  frameDrafts,
  readFrames,
  reorderFrames,
  toKeyframes,
  valuePercent,
} from "../src/keyframe-tools.ts";
test("keyframe authoring round-trips all u16 values through precise percent inputs", () => {
  for (let i = 0; i <= 65535; i++)
    assert.equal(Math.round((Number(valuePercent(i)) * 65535) / 100), i);
  const e = createEffect("multicolor", "id", ["rgb"]);
  assert.deepEqual(
    readFrames(frameDrafts(e.channels), ["red", "green", "blue"]),
    e.channels,
  );
});
test("converting basic shapes retains endpoints and transition intent", () => {
  for (const kind of ["breathe", "chase", "color"] as const) {
    const before = createEffect(kind, "id", ["rgb"]),
      next = toKeyframes(before);
    assert.equal(next.waveform, "keyframes");
    assert.equal(before.channels[0].keyframes, undefined);
    assert.equal(next.channels[0].keyframes![0].position, 0);
    assert.equal(
      next.channels[0].keyframes![0].value,
      kind === "chase" ? 65535 : 0,
    );
    assert.equal(
      next.channels[0].keyframes![1].position,
      kind === "chase" ? 2500 : 5000,
    );
  }
});
test("reordering keyframes moves values while preserving time slots, even spacing is explicit", () => {
  const frames = frameDrafts(
    createEffect("multicolor", "id", ["rgb"]).channels,
  );
  const moved = reorderFrames(frames, 0, 1);
  assert.deepEqual(
    moved.map((f) => f.position),
    frames.map((f) => f.position),
  );
  assert.deepEqual(moved[0].values, frames[1].values);
  const added = appendFrame(moved);
  assert.equal(added.length, 4);
  assert.deepEqual(
    evenFrames(added).map((f) => f.position),
    ["0", "25", "50", "75"],
  );
  assert.equal(frames.length, 3);
});
test("bad keyframe timing and values identify the failing field without sorting away intent", () => {
  const frames = frameDrafts(
    createEffect("multicolor", "id", ["rgb"]).channels,
  );
  frames[1].position = "90";
  assert.throws(
    () => readFrames(frames, ["red", "green", "blue"]),
    /第 3 帧时间/,
  );
  frames[1].position = "33.333";
  assert.throws(() => readFrames(frames, ["red"]), /第 2 帧时间/);
  frames[1].position = "33.33";
  frames[1].values.red = "";
  assert.throws(() => readFrames(frames, ["red"]), /第 2 帧红/);
});
test("effect reuse deep copies curves, keeps source intact and checks replacement fixture capability", () => {
  const source = createEffect("multicolor", "source", ["rgb"]),
    copy = reuseEffect(source, "copy", ["dim"]);
  assert.equal(copy.enabled, false);
  assert.deepEqual(copy.fixtureIds, ["dim"]);
  copy.channels[0].keyframes![0].value = 12345;
  assert.equal(source.channels[0].keyframes![0].value, 0);
  assert.throws(() => effectCommands("s", copy, fixtures, false), /dim/);
});

import {
  positionAxisDrafts,
  readPositionAxes,
} from "../src/position-effect-tools.ts";
test("position templates require physical models and preserve independent ordered authoring data", () => {
  const moving = fixture("moving", ["dimmer", "pan", "tilt"]);
  assert.equal(supportsEffect([moving], "circle"), false);
  moving.positioning = {
    kind: "intersectingOrthogonal",
    pan: { minDegrees: "-270", maxDegrees: "270", reversed: true },
    tilt: { minDegrees: "-135", maxDegrees: "135", reversed: false },
  };
  assert.equal(supportsEffect([moving], "circle"), true);
  assert.equal(supportsEffect([moving, fixtures[0]], "panSweep"), false);
  const ids = ["moving"];
  const circle = createEffect("circle", "motion", ids);
  ids.push("rgb");
  assert.deepEqual(circle.fixtureIds, ["moving"]);
  assert.deepEqual(
    circle.channels.map((c) => c.phaseDegrees),
    [0, 90],
  );
  const operations = effectCommands("s", circle, [moving], false);
  assert.equal(operations.length, 1); // Motion never changes static position or illumination.
  assert.throws(() => toKeyframes(circle), /运动属性/);
  const copy = reuseEffect(circle, "copy");
  copy.channels[0].amplitudeDegrees = "1";
  assert.equal(circle.channels[0].amplitudeDegrees, "15");
  assert.equal(copy.enabled, false);
  for (const kind of ["panSweep", "tiltSweep", "circle"] as const) {
    const channels = createEffect(kind, "m", ["moving"]).channels;
    assert.deepEqual(readPositionAxes(positionAxisDrafts(channels)), channels);
  }
});
test("motion angle input canonicalizes decimals and rejects invalid axes without changing drafts", () => {
  const axes = positionAxisDrafts(
    createEffect("circle", "m", ["moving"]).channels,
  );
  axes[0].amplitude = "30.000000";
  axes[0].offset = "-0.000000";
  axes[1].amplitude = "0.000001";
  assert.equal(readPositionAxes(axes)[0].amplitudeDegrees, "30");
  assert.equal(readPositionAxes(axes)[0].offsetDegrees, "0");
  assert.equal(readPositionAxes(axes)[1].amplitudeDegrees, "0.000001");
  assert.equal(axes[0].amplitude, "30.000000");
  for (const bad of ["", "-1", "3601", "NaN", "1e2", "0.0000001"]) {
    assert.throws(
      () => readPositionAxes([{ ...axes[0], amplitude: bad }]),
      /水平幅度/,
    );
  }
  assert.throws(
    () => readPositionAxes([{ ...axes[0], phase: "360" }]),
    /水平相位/,
  );
  assert.throws(() => readPositionAxes([]), /运动轴/);
  assert.throws(() => readPositionAxes([axes[0], axes[0]]), /运动轴/);
});
