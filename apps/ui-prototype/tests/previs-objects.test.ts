import { test } from "node:test";
import assert from "node:assert/strict";
import { readPrevisMessage } from "../src/previs-messages.ts";
import { readPrevisTargets, viewportTargets } from "../src/previs-objects.ts";

const targets = [
  { kind: "placement", id: "same" },
  { kind: "construction", id: "same" },
];
const proposal = {
  kind: "objectTranslation",
  requestId: "a".repeat(32),
  generation: 7,
  version: "9007199254740993",
  targets,
  deltaMeters: { x: "1.25", y: "0", z: "-0.25" },
};
const read = (v: unknown) => readPrevisMessage(JSON.stringify(v));

test("三维对象按类型区分身份，完整混合选择与有界位移保留精确版本", () => {
  assert.deepEqual(read({ kind: "selectionTargets", targets }), {
    kind: "selectionTargets",
    targets,
  });
  assert.deepEqual(read(proposal), proposal);
  assert.deepEqual(readPrevisTargets([]), []);
  const all = Array.from({ length: 1024 }, (_, i) => ({
    kind: "construction",
    id: `c${i}`,
  }));
  assert.deepEqual(read({ kind: "selectionTargets", targets: all }), {
    kind: "selectionTargets",
    targets: all,
  });
  assert.ok(read({ ...proposal, targets: all.slice(0, 256) }));
  assert.equal(read({ ...proposal, targets: all.slice(0, 257) }), null);
  assert.equal(
    read({
      kind: "selectionTargets",
      targets: [...all, { kind: "placement", id: "extra" }],
    }),
    null,
  );
});
test("对象提案严格拒绝重复身份、未知类型、额外字段与非法位移", () => {
  for (const targets of [
    [{ kind: "space", id: "x" }],
    [{ kind: "construction", id: "x", extra: true }],
    [{ kind: "placement", id: "" }],
    [{ kind: "construction", id: 1 }],
    [null],
    [{ kind: "construction", id: "x".repeat(257) }],
    [
      { kind: "placement", id: "x" },
      { kind: "placement", id: "x" },
    ],
  ]) {
    assert.equal(read({ kind: "selectionTargets", targets }), null);
    assert.equal(read({ ...proposal, targets }), null);
  }
  for (const patch of [
    { targets: [] },
    { extra: true },
    { version: "01" },
    { generation: -1 },
    { deltaMeters: { x: "200000.000001", y: "0", z: "0" } },
    { deltaMeters: { x: "0", y: "0", z: "NaN" } },
  ])
    assert.equal(read({ ...proposal, ...patch }), null);
});
test("不支持的空间混选不会被截成灯具子集；对象过滤状态拒绝伪布尔", () => {
  assert.deepEqual(
    viewportTargets([
      { kind: "space", id: "room" },
      { kind: "placement", id: "lamp" },
    ]),
    [],
  );
  const state = {
    kind: "state",
    status: "",
    selection: "",
    workLight: "",
    move: false,
    interactionVersion: 4,
  };
  for (const allObjects of [true, false]) {
    const value = read({ ...state, allObjects });
    assert.equal(value?.kind, "state");
    if (value?.kind === "state") assert.equal(value.allObjects, allObjects);
  }
  for (const allObjects of [1, "false", [], null])
    assert.equal(read({ ...state, allObjects }), null);
});
