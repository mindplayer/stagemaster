import test from "node:test";
import assert from "node:assert/strict";
import {
  profileDraft,
  profileDefinition,
  compatibleProfile,
} from "../src/fixture-tools.ts";
import {
  addFunctionChannel,
  withLinearFamily,
  newFunction,
} from "../src/fixture-function-draft.ts";
import { addSlotBatch, planSlotBatch } from "../src/fixture-slot-batch.ts";
import { commonAttributes } from "../src/editor-tools.ts";
import type { FixtureView } from "../src/application-host.ts";
function wheel() {
  return addFunctionChannel(
    withLinearFamily(profileDraft(), "dimmer"),
    "color-wheel",
  );
}
const batch = { start: "0", width: "10", count: "14", name: "颜色档位" };
test("14 档批量创建保留控制身份、原草稿与自动区域，代表值可落在各档内部", () => {
  const d = wheel(),
    c = d.channels[1],
    old = structuredClone(c);
  const next = addSlotBatch(c, batch, "channel-1");
  assert.deepEqual(c, old);
  assert.equal(next.functions?.length, 14);
  assert.equal(next.functions?.[0].key, c.functions?.[0].key);
  assert.deepEqual(next.defaultFunction, c.defaultFunction);
  assert.deepEqual(
    next.functions?.map((f) => [f.dmxFrom, f.dmxTo, f.dmxDefault]),
    Array.from({ length: 14 }, (_, i) => [
      `${i * 10}`,
      `${i * 10 + 9}`,
      `${i * 10 + 4}`,
    ]),
  );
  const automatic = {
    ...newFunction(140),
    key: "automatic",
    name: "自动换色",
    mode: "range" as const,
    dmxTo: "255",
  };
  c.functions = [automatic];
  c.defaultFunction = { functionKey: "automatic", position: "12345" };
  const withAutomatic = addSlotBatch(c, batch, "channel-1");
  assert.deepEqual(withAutomatic.functions![0], automatic);
  assert.deepEqual(withAutomatic.defaultFunction, c.defaultFunction);
  assert.equal(withAutomatic.functions!.length, 15);
});
test("批量创建拒绝覆盖已编辑行、重叠、越界、超容量与小数；16 位支持全范围", () => {
  const c = wheel().channels[1];
  for (const patch of [
    { count: "26" },
    { count: "65" },
    { width: "0" },
    { count: "1.5" },
    { start: "-1" },
    { name: "" },
  ])
    assert.throws(() => planSlotBatch(c, { ...batch, ...patch }, "ch"));
  c.functions![0].name = "已确认通光";
  Object.assign(c.functions![0], { dmxFrom: "0", dmxTo: "0", dmxDefault: "0" });
  assert.throws(() => planSlotBatch(c, batch, "ch"), /重叠/);
  c.functions = Array.from({ length: 64 }, (_, i) => ({
    ...newFunction(i),
    name: `档位 ${i}`,
  }));
  assert.throws(
    () => planSlotBatch(c, { ...batch, start: "100", count: "1" }, "ch"),
    /64/,
  );
  c.functions = [];
  c.bits = "16";
  const plan = planSlotBatch(c, { ...batch, count: "1", width: "65536" }, "ch");
  assert.deepEqual(plan.slots[0], {
    name: "颜色档位 1",
    from: 0,
    to: 65535,
    representative: 32767,
  });
});
test("通光、单色和半色往返；未知保持缺省，不发明白色", () => {
  const d = wheel();
  d.channels[1] = addSlotBatch(d.channels[1], batch, "ch");
  const fs = d.channels[1].functions!;
  fs[0].appearance = { kind: "open" };
  fs[1].appearance = { kind: "color", colors: ["#FF0000"] };
  fs[2].appearance = { kind: "color", colors: ["#00ff00", "#ffffff"] };
  const p = profileDefinition(d);
  assert.deepEqual(profileDefinition(profileDraft(p)), p);
  assert.equal(p.channels[1].functions![3].appearance, undefined);
  fs[2].appearance.colors[1] = "red";
  assert.throws(() => profileDefinition(d), /#RRGGBB/);
  fs[2].appearance = undefined;
  fs[1].mode = "range";
  assert.throws(() => profileDefinition(d), /固定档位/);
});
test("变体可显式替换，但不同外观不能作为同一种颜色批量编辑；改代表值仍拒绝", () => {
  const d = wheel();
  d.channels[1] = addSlotBatch(d.channels[1], batch, "ch");
  const p = profileDefinition(d);
  const fixture: FixtureView = {
    id: "one",
    profileId: "p",
    name: "灯",
    profileName: "原模式",
    domainId: "d",
    domainName: "灯光",
    footprint: 2,
    attributes: [
      { key: "dimmer", label: "亮度", defaultValue: 0 },
      {
        key: "color-wheel",
        label: "色盘",
        defaultValue: 0,
        function: {
          functions: structuredClone(p.channels[1].functions!),
          default: {
            functionKey: p.channels[1].functions![0].key,
            position: 0,
          },
          fine: false,
        },
      },
    ],
  };
  p.channels[1].functions![0].name = "实测为红";
  p.channels[1].functions![0].appearance = {
    kind: "color",
    colors: ["#FF0000"],
  };
  p.channels[1].functions!.reverse();
  assert.equal(compatibleProfile([fixture], p), true);
  const other = structuredClone(fixture);
  other.id = "two";
  other.attributes[1].function!.functions[0].appearance = {
    kind: "color",
    colors: ["#00FF00"],
  };
  assert.deepEqual(
    commonAttributes([fixture, other]).map((a) => a.key),
    ["dimmer"],
  );
  p.channels[1].functions![0].dmxDefault++;
  assert.equal(compatibleProfile([fixture], p), false);
});
