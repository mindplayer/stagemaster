import { test } from "node:test";
import assert from "node:assert/strict";
import type {
  FixtureView,
  ProjectView,
  SceneView,
} from "../src/application-host.ts";
import {
  availableAddress,
  commonAttributes,
  attributeState,
  parameterCommands,
  selectRange,
  uniqueName,
} from "../src/editor-tools.ts";
const fixture = (id: string, address: number, domainId = "a"): FixtureView => ({
  id,
  name: id,
  profileName: "RGB",
  domainName: domainId,
  domainId,
  footprint: 4,
  address,
  universe: 1,
  attributes: ["dimmer", "red", "green", "blue"].map((key) => ({
    key,
    label: key,
    defaultValue: 0,
  })),
});
const fixtures = [fixture("one", 1), fixture("two", 9)];
const project: ProjectView = {
  id: "p",
  name: "p",
  description: "",
  profiles: [],
  domains: [],
  fixtures,
  scenes: [],
};
const scene: SceneView = {
  id: "s",
  name: "s",
  values: [
    {
      fixtureId: "one",
      attribute: "red",
      mode: "literal",
      value: 12345,
      presetName: null,
    },
    {
      fixtureId: "two",
      attribute: "red",
      mode: "preset",
      value: 12345,
      presetName: "红色",
    },
  ],
};
test("suggestion finds a contiguous block in the same domain and universe without wrapping", () => {
  assert.equal(availableAddress(project, "a", 1, 4), 5);
  assert.equal(availableAddress(project, "a", 1, 4, 2), 13);
  assert.equal(availableAddress(project, "b", 1, 4, 2), 1);
  assert.equal(availableAddress(project, "a", 2, 4, 2), 1);
  assert.equal(availableAddress(project, "a", 1, 4, 128), null);
});
test("mixed state distinguishes preset, literal, absent and release even at equal numeric values", () => {
  assert.equal(attributeState(scene, fixtures, "red").mixed, true);
  assert.equal(attributeState(scene, fixtures, "green").mode, "absent");
  const released = {
    ...scene,
    values: [
      {
        fixtureId: "one",
        attribute: "green",
        mode: "release",
        value: null,
        presetName: null,
      },
    ],
  };
  assert.equal(attributeState(released, fixtures, "green").mixed, true);
});
test("multi selection exposes intersection and generates only explicitly edited values", () => {
  const dimmer = {
    ...fixtures[1],
    attributes: fixtures[1].attributes.slice(0, 1),
  };
  assert.deepEqual(
    commonAttributes([fixtures[0], dimmer]).map((a) => a.key),
    ["dimmer"],
  );
  assert.throws(() =>
    parameterCommands("s", [fixtures[0], dimmer], { red: 100 }),
  );
  const commands = parameterCommands("s", fixtures, { red: "50" });
  assert.equal(commands.length, 2);
  assert.ok(
    commands.every(
      (c) =>
        c.op === "setSceneValue" && c.value === 32768 && c.attribute === "red",
    ),
  );
});
test("RGB is a single bounded command group, invalid text is never coerced to zero", () => {
  assert.equal(
    parameterCommands("s", fixtures, { red: 65535, green: 0, blue: 257 })
      .length,
    6,
  );
  for (const text of ["", " ", "101", "-1", "NaN"])
    assert.throws(() => parameterCommands("s", fixtures, { red: text }));
  assert.throws(() =>
    parameterCommands(
      "s",
      Array.from({ length: 86 }, (_, i) => fixture(String(i), 1)),
      { red: 1, green: 2, blue: 3 },
    ),
  );
});
test("range selection uses visible order and preserves additive selection without duplicates", () => {
  assert.deepEqual(
    selectRange(["a", "b", "c", "d"], ["d"], "c", "a", true, false),
    ["a", "b", "c"],
  );
  assert.deepEqual(
    selectRange(["a", "b", "c", "d"], ["a", "d"], "c", "a", true, true),
    ["a", "d", "b", "c"],
  );
  assert.deepEqual(selectRange(["a", "b"], ["a"], "a", "a", false, true), []);
  assert.deepEqual(selectRange(["b", "c"], ["a"], "c", "a", true, false), [
    "c",
  ]);
});
test("duplicate names fill available suffix without colliding after deletions", () => {
  assert.equal(uniqueName("场景", ["场景", "场景 3"]), "场景 2");
  assert.equal(uniqueName("场景", []), "场景");
});
