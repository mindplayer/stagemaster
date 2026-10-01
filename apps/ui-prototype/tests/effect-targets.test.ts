import { test } from "node:test";
import assert from "node:assert/strict";
import type { FixtureView } from "../src/application-host.ts";
import {
  createEffect,
  effectCommands,
  reuseEffect,
} from "../src/effect-tools.ts";
import {
  arrangeEffectFixtures,
  effectTargetIssues,
} from "../src/effect-targets.ts";
const f = (id: string, address: number | null, name = id): FixtureView => ({
  id,
  name,
  profileId: "p",
  profileName: "RGB",
  domainName: "灯光",
  domainId: "d",
  footprint: 4,
  universe: address === null ? null : 1,
  address,
  attributes: ["dimmer", "red", "green", "blue"].map((key) => ({
    key,
    label: key,
    defaultValue: 0,
  })),
});
const fixtures = [
  f("a", 20, "灯10"),
  f("b", 5, "灯2"),
  f("c", null, "灯1"),
  f("d", 5, "灯2"),
];
test("灯序整组变换保持身份、不改输入和相位标志，名称自然排序、相同键稳定", () => {
  const ids = ["a", "c", "d", "b"];
  const before = structuredClone(ids);
  assert.deepEqual(arrangeEffectFixtures(ids, fixtures, "name"), [
    "c",
    "d",
    "b",
    "a",
  ]);
  assert.deepEqual(arrangeEffectFixtures(ids, fixtures, "reverse"), [
    "b",
    "d",
    "c",
    "a",
  ]);
  assert.deepEqual(arrangeEffectFixtures(ids, fixtures, "oddFirst"), [
    "a",
    "d",
    "c",
    "b",
  ]);
  assert.deepEqual(arrangeEffectFixtures(ids, fixtures, "patch"), [
    "d",
    "b",
    "a",
    "c",
  ]);
  assert.deepEqual(ids, before);
  const source = createEffect("chase", "source", ids);
  source.reverse = true;
  const copy = reuseEffect(
    source,
    "copy",
    arrangeEffectFixtures(ids, fixtures, "reverse"),
  );
  assert.equal(copy.reverse, true);
  assert.deepEqual(source.fixtureIds, before);
  assert.deepEqual(effectCommands("s", copy, fixtures, false)[0], {
    op: "effect",
    command: { kind: "put", sceneId: "s", effect: copy },
  });
});
test("配适按控制域、输出路、地址分组，未配适与缺失保持稳定且不丢弃", () => {
  const second = { ...f("second", 1), universe: 2 };
  const other = { ...f("other", 1), domainId: "z" };
  const unpatched = f("unpatched", null);
  assert.deepEqual(
    arrangeEffectFixtures(
      ["lost1", "second", "unpatched", "c", "a", "other", "lost2"],
      [...fixtures, second, other, unpatched],
      "patch",
    ),
    ["a", "second", "other", "unpatched", "c", "lost1", "lost2"],
  );
  for (const mode of ["reverse", "oddFirst", "name", "patch"] as const) {
    assert.deepEqual(arrangeEffectFixtures([], fixtures, mode), []);
    assert.deepEqual(arrangeEffectFixtures(["a"], fixtures, mode), ["a"]);
  }
});
test("目标报告列出混合能力、已删除与重复，不静默筛掉灯具", () => {
  const dim = {
    ...f("dim", 100),
    attributes: [{ key: "dimmer", label: "亮度", defaultValue: 0 }],
  };
  const color = createEffect("color", "color", ["a"]);
  assert.deepEqual(effectTargetIssues(["a"], fixtures, color.channels), []);
  const issues = effectTargetIssues(
    ["a", "dim", "missing", "a"],
    [...fixtures, dim],
    color.channels,
  );
  assert.deepEqual(
    issues.map((i) => i.id),
    ["dim", "missing", "a"],
  );
  assert.match(issues[0].reason, /红、绿、蓝/);
  assert.match(issues[1].reason, /删除/);
  assert.match(issues[2].reason, /重复/);
  assert.match(
    effectTargetIssues([], fixtures, color.channels)[0].reason,
    /至少/,
  );
  assert.throws(
    () =>
      effectCommands(
        "s",
        { ...color, fixtureIds: ["dim"] },
        [...fixtures, dim],
        false,
      ),
    /dim.*缺少红/,
  );
});
test("位置效果还要求运动模型，模型与属性检查可同时报告", () => {
  const motion = createEffect("circle", "circle", ["a"]);
  assert.match(
    effectTargetIssues(["a"], fixtures, motion.channels)[0].reason,
    /水平、垂直.*两轴/,
  );
  const moving: FixtureView = {
    ...f("m", 1),
    attributes: ["pan", "tilt"].map((key) => ({
      key,
      label: key,
      defaultValue: 0,
    })),
    positioning: {
      kind: "intersectingOrthogonal",
      pan: { minDegrees: "-270", maxDegrees: "270", reversed: false },
      tilt: { minDegrees: "-135", maxDegrees: "135", reversed: false },
    },
  };
  assert.deepEqual(effectTargetIssues(["m"], [moving], motion.channels), []);
  assert.match(
    effectTargetIssues(
      ["m"],
      [{ ...moving, positioning: null }],
      motion.channels,
    )[0].reason,
    /两轴/,
  );
  assert.match(
    effectTargetIssues(
      ["a"],
      fixtures,
      createEffect("breathe", "b", ["a"]).channels,
      true,
    )[0].reason,
    /两轴/,
  );
});
