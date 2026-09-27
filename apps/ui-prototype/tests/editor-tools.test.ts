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
  groups: [],
  presets: [],
  sequences: [],
  stage: {attachments:[],spaces:[],constructions:[],placements:[]},
};
const scene: SceneView = {
  effects: [],
  id: "s",
  name: "s",
  values: [
    {
      fixtureId: "one",
      attribute: "red",
      mode: "literal",
      value: 12345,
      presetName: null,
      presetId: null,
    },
    {
      fixtureId: "two",
      attribute: "red",
      mode: "preset",
      value: 12345,
      presetName: "红色",
      presetId: "p",
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
        presetId: null,
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

test("sequence times are parsed exactly and never silently rounded", async () => {
  const { secondsToMs, seconds } = await import("../src/sequence-tools.ts");
  for (const [input, expected] of [
    ["0", 0],
    ["1.005", 1005],
    [" 2.01 ", 2010],
    ["86400", 86400000],
  ] as const)
    assert.equal(secondsToMs(input, "渐变"), expected);
  for (const input of [
    "",
    "-1",
    "1e3",
    "0.0001",
    "1.",
    "Infinity",
    "86400.001",
    "9007199254740993",
  ])
    assert.throws(() => secondsToMs(input, "渐变"), /渐变/);
  assert.equal(seconds(1005), "1.005");
});
test("monitor color does not confuse RGB level with a separate dimmer", async () => {
  const { fixtureAppearance } = await import("../src/sequence-tools.ts");
  assert.deepEqual(
    fixtureAppearance([
      { key: "red", value: 65535 },
      { key: "green", value: 0 },
      { key: "blue", value: 0 },
    ]),
    { level: null, color: "rgb(255 0 0)" },
  );
  assert.equal(fixtureAppearance([{ key: "dimmer", value: 0 }]).level, 0);
});

test("sequence draft validation locates bad fields and groups metadata, timing and order atomically", async () => {
  const { sequenceDraft, sequenceCommands, SequenceInputError } = await import(
    "../src/sequence-tools.ts"
  );
  const a = {
    id: "a",
    name: "入场",
    number: "1",
    sceneId: "scene",
    delayMs: 0,
    fadeMs: 1000,
    waitMs: null,
  };
  const b = { ...a, id: "b", name: "转场", number: "2" };
  const sequence = {
    id: "seq",
    name: "晚场",
    tracking: "inherited" as const,
    repeat: "once" as const,
    steps: [a, b],
  };
  const draft = sequenceDraft(sequence, b);
  assert.deepEqual(sequenceCommands(draft, sequence, b), []);
  for (const [field, value] of [
    ["number", "1.0"],
    ["position", "3"],
    ["fade", "1.0001"],
    ["sequenceName", " "],
  ] as const)
    assert.throws(
      () => sequenceCommands({ ...draft, [field]: value }, sequence, b),
      (error: unknown) =>
        error instanceof SequenceInputError && error.field === field,
    );
  const commands = sequenceCommands(
    { ...draft, sequenceName: "新节目", fade: "1.005", position: "1" },
    sequence,
    b,
  );
  assert.equal(commands.length, 3);
  assert.deepEqual(commands.at(-1), {
    op: "sequence",
    command: { kind: "moveStep", id: "seq", stepId: "b", index: 0 },
  });
});

test("channel monitoring pages retain absolute addresses and reject invalid ranges", async () => {
  const { channelWindow } = await import("../src/sequence-tools.ts");
  const slots = Array.from({ length: 512 }, (_, i) => i % 256);
  assert.equal(channelWindow(slots, "", 0).rows.length, 64);
  assert.equal(channelWindow(slots, "", 7).rows.at(-1)?.address, 512);
  const page = channelWindow(slots, "101–180", 1);
  assert.equal(page.rows[0].address, 165);
  assert.equal(page.rows.at(-1)?.address, 180);
  assert.equal(channelWindow(slots, "512", 99).rows[0].address, 512);
  for (const q of ["0", "3-2", "1-513", "abc"])
    assert.equal(channelWindow(slots, q, 0).valid, false);
});

test("group recall keeps stored order, appends uniquely and subtracts without reordering", async () => {
  const { recallGroup, moveMember } = await import("../src/library-tools.ts");
  assert.deepEqual(recallGroup(["c"], ["b", "a"], "replace"), ["b", "a"]);
  assert.deepEqual(recallGroup(["c", "b"], ["b", "a"], "add"), ["c", "b", "a"]);
  assert.deepEqual(recallGroup(["c", "b", "a"], ["b"], "subtract"), ["c", "a"]);
  assert.deepEqual(moveMember(["a", "b", "c"], 2, -1), ["a", "c", "b"]);
  assert.deepEqual(moveMember(["a", "b"], 0, -1), ["a", "b"]);
});
test("preset coverage excludes release and unrecorded values without treating zero as absent", async () => {
  const { matchingValues, presetCoverage } = await import(
    "../src/library-tools.ts"
  );
  const values = [
    ...scene.values,
    { ...scene.values[0], attribute: "green", value: 0 },
  ];
  assert.equal(matchingValues(values, ["one"], ["red", "green"]).length, 2);
  assert.equal(
    matchingValues(
      [{ ...values[0], mode: "release", value: null }],
      ["one"],
      ["red"],
    ).length,
    0,
  );
  assert.deepEqual(
    presetCoverage(
      { id: "p", name: "p", values, usedByScenes: [], usedBySequences: [] },
      ["one", "missing"],
      ["red"],
    ),
    { fixtures: 1, attributes: 1 },
  );
});
test("same-name presets with equal numbers remain distinct references", () => {
  const values = scene.values.map((v, i) => ({
    ...v,
    mode: "preset",
    presetName: "红色",
    presetId: `preset-${i}`,
  }));
  assert.equal(
    attributeState({ ...scene, values }, fixtures, "red").mixed,
    true,
  );
});
