import { test } from "node:test";
import assert from "node:assert/strict";
import { readPrevisMessage } from "../src/previs-messages.ts";
const proposal = {
  kind: "placement",
  requestId: "a".repeat(32),
  generation: 7,
  version: "9007199254740993",
  placement: {
    fixtureId: "fixture-1",
    spaceId: null,
    positionMeters: { x: "-0.123456", y: "2", z: "3" },
    rotationDegreesXYZ: { x: "25", y: "0", z: "0" },
  },
};
test("renderer proposals preserve exact revision and signed decimal coordinates", () => {
  assert.deepEqual(readPrevisMessage(JSON.stringify(proposal)), proposal);
  assert.deepEqual(readPrevisMessage('{"kind":"selection","fixtureId":""}'), {
    kind: "selection",
    fixtureId: "",
  });
});
test("renderer proposals reject malformed, oversized and lossy payloads", () => {
  for (const patch of [
    { generation: 2 ** 32 },
    { generation: 2.5 },
    { version: "18446744073709551616" },
    { version: "1e3" },
    { requestId: "" },
    { placement: {} },
  ])
    assert.equal(
      readPrevisMessage(JSON.stringify({ ...proposal, ...patch })),
      null,
    );
  for (const x of ["NaN", "Infinity", "0.1234567", 2, "1e10"])
    assert.equal(
      readPrevisMessage(
        JSON.stringify({
          ...proposal,
          placement: {
            ...proposal.placement,
            positionMeters: { x, y: "0", z: "0" },
          },
        }),
      ),
      null,
    );
  assert.equal(readPrevisMessage(" ".repeat(8193)), null);
  assert.equal(readPrevisMessage("{"), null);
  assert.equal(readPrevisMessage('{"kind":"execute"}'), null);
});
test("剖视状态兼容旧渲染器，拒绝错误字段类型", () => {
  const state = {
    kind: "state",
    status: "场景预演",
    selection: "",
    workLight: "工作照明：开",
    move: false,
  };
  assert.deepEqual(readPrevisMessage(JSON.stringify(state)), {
    ...state,
    cutaway: false,
    interactionVersion: 1,
    vertical: false,
    tool: "horizontal",
    marqueeSupported: false,
    allObjects: false,
    marqueeMode: "replace",
    selectionThrough: false,
  });
  assert.deepEqual(
    readPrevisMessage(JSON.stringify({ ...state, cutaway: true })),
    {
      ...state,
      cutaway: true,
      interactionVersion: 1,
      vertical: false,
      tool: "horizontal",
      marqueeSupported: false,
      allObjects: false,
      marqueeMode: "replace",
      selectionThrough: false,
    },
  );
  assert.equal(
    readPrevisMessage(JSON.stringify({ ...state, cutaway: "false" })),
    null,
  );
});

test("三维整组选灯允许清空和全场，拒绝重复、超限与未知字段", () => {
  for (const fixtureIds of [
    [],
    ["a"],
    Array.from({ length: 1024 }, (_, i) => `fixture-${i}`),
  ]) {
    const value = { kind: "selectionGroup", fixtureIds };
    assert.deepEqual(readPrevisMessage(JSON.stringify(value)), value);
  }
  for (const fixtureIds of [
    ["a", "a"],
    [""],
    [12],
    Array.from({ length: 1025 }, (_, i) => `${i}`),
  ])
    assert.equal(
      readPrevisMessage(JSON.stringify({ kind: "selectionGroup", fixtureIds })),
      null,
    );
  assert.equal(
    readPrevisMessage(
      JSON.stringify({
        kind: "selectionGroup",
        fixtureIds: ["a"],
        execute: true,
      }),
    ),
    null,
  );
  assert.equal(
    readPrevisMessage(
      JSON.stringify({
        kind: "selection",
        fixtureId: "a",
        extra: "x".repeat(65536),
      }),
    ),
    null,
  );
});
test("整组位移保留精确版本与相对高差，严格校验身份和位移边界", () => {
  const translation = {
    kind: "translation",
    requestId: "b".repeat(32),
    generation: 7,
    version: "9007199254740993",
    fixtureIds: ["a", "b"],
    deltaMeters: { x: "1.25", y: "0", z: "-0.625" },
  };
  assert.deepEqual(readPrevisMessage(JSON.stringify(translation)), translation);
  const boundary = {
    ...translation,
    fixtureIds: Array.from({ length: 256 }, (_, i) => `${i}`),
    deltaMeters: { x: "200000", y: "-200000", z: "0.000001" },
  };
  assert.deepEqual(readPrevisMessage(JSON.stringify(boundary)), boundary);
  for (const patch of [
    { fixtureIds: [] },
    { fixtureIds: ["a", "a"] },
    { fixtureIds: [...boundary.fixtureIds, "extra"] },
    { generation: -1 },
    { version: "01" },
    { requestId: "X".repeat(32) },
    { extra: true },
  ])
    assert.equal(
      readPrevisMessage(JSON.stringify({ ...translation, ...patch })),
      null,
    );
  for (const x of [
    "200000.000001",
    "-200001",
    "Infinity",
    "NaN",
    "1e2",
    "0.0000001",
    1,
  ])
    assert.equal(
      readPrevisMessage(
        JSON.stringify({ ...translation, deltaMeters: { x, y: "0", z: "0" } }),
      ),
      null,
    );
});
test("渲染器显式提供移动交互版本和方向，错误类型不开放移动", () => {
  const state = {
    kind: "state",
    status: "场地预演",
    selection: "已选 2 台",
    workLight: "工作照明：开",
    move: true,
    cutaway: true,
    interactionVersion: 2,
    vertical: true,
    tool: "vertical",
  };
  assert.deepEqual(readPrevisMessage(JSON.stringify(state)), {
    ...state,
    marqueeSupported: false,
    allObjects: false,
    marqueeMode: "replace",
    selectionThrough: false,
  });
  for (const patch of [
    { interactionVersion: "2" },
    { interactionVersion: 2.5 },
    { vertical: "true" },
    { tool: ["rotate"] },
    { tool: "unknown" },
  ])
    assert.equal(
      readPrevisMessage(JSON.stringify({ ...state, ...patch })),
      null,
    );
});

test("整组变换严格校验版本、角度、比例和目标", () => {
  const transform = {
    kind: "transform",
    requestId: "a".repeat(32),
    generation: 1,
    version: "9007199254740993",
    fixtureIds: ["a", "b"],
    yawDegrees: "-90.125",
    spacingScale: "1.25",
  };
  assert.deepEqual(readPrevisMessage(JSON.stringify(transform)), transform);
  for (const patch of [
    { yawDegrees: "361" },
    { yawDegrees: "NaN" },
    { yawDegrees: "1e2" },
    { spacingScale: "0" },
    { spacingScale: "-1" },
    { spacingScale: "100.000001" },
    { spacingScale: "1.0000001" },
    { fixtureIds: [] },
    { fixtureIds: ["a", "a"] },
    { extra: true },
  ])
    assert.equal(
      readPrevisMessage(JSON.stringify({ ...transform, ...patch })),
      null,
    );
});

test("框选能力和穿透状态明确兼容旧版本并拒绝伪布尔", () => {
  const state = {
    kind: "state",
    status: "就绪",
    selection: "",
    workLight: "开",
    move: false,
    interactionVersion: 3,
  };
  for (const selectionThrough of [true, false]) {
    const value = readPrevisMessage(
      JSON.stringify({ ...state, marqueeSupported: true, selectionThrough }),
    );
    assert.equal(value?.kind, "state");
    if (value?.kind === "state") {
      assert.equal(value.marqueeSupported, true);
      assert.equal(value.selectionThrough, selectionThrough);
      assert.equal(value.interactionVersion, 3);
    }
  }
  for (const field of ["marqueeSupported", "selectionThrough"])
    for (const value of ["false", 1, null, [], {}])
      assert.equal(
        readPrevisMessage(JSON.stringify({ ...state, [field]: value })),
        null,
      );
});

test("框选方式只接受明确枚举，兼容旧状态的替换选择", () => {
  const state = {
    kind: "state",
    status: "",
    selection: "",
    workLight: "",
    move: false,
    marqueeSupported: true,
  };
  for (const marqueeMode of ["replace", "add", "remove"]) {
    const result = readPrevisMessage(JSON.stringify({ ...state, marqueeMode }));
    assert.equal(result?.kind, "state");
    if (result?.kind === "state") assert.equal(result.marqueeMode, marqueeMode);
  }
  for (const marqueeMode of ["toggle", ["add"], null, 1])
    assert.equal(
      readPrevisMessage(JSON.stringify({ ...state, marqueeMode })),
      null,
    );
});
