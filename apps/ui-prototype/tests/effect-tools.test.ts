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
