import { fixtureMatches } from "../src/editor-tools.ts";
import test from "node:test";
import assert from "node:assert/strict";
import type { FixtureView } from "../src/application-host.ts";
import {
  fixtureSymbol,
  fixtureLegend,
  fixtureAddress,
  fixturePlanLabel,
} from "../src/fixture-plan-display.ts";
function fixture(id: string, keys: string[]): FixtureView {
  return {
    id,
    name: `灯具 ${id}`,
    profileName: "不按名称识别的摇头灯",
    profileId: "profile",
    domainId: "domain",
    domainName: "灯光",
    footprint: 1,
    universe: 1,
    address: 7,
    attributes: keys.map((key) => ({ key, label: key, defaultValue: 0 })),
  };
}
test("改名、属性次序和局部颜色定义不误判双轴与完整混色", () => {
  const dim = fixture("1", ["dimmer"]),
    rgb = fixture("2", ["blue", "dimmer", "red", "green"]);
  assert.equal(fixtureSymbol(dim).moving, false);
  assert.equal(fixtureSymbol(dim).rgb, false);
  assert.equal(fixtureSymbol(rgb).rgb, true);
  assert.equal(
    fixtureSymbol({ ...rgb, name: "x" } as FixtureView).moving,
    false,
  );
  assert.equal(
    fixtureSymbol(fixture("3", ["pan", "red", "blue"])).moving,
    false,
  );
  assert.equal(fixtureSymbol(fixture("3", ["pan", "red", "blue"])).rgb, false);
  const mover = fixture("4", [
    "pan",
    "tilt",
    "dimmer",
    "red",
    "green",
    "blue",
    "color-wheel",
  ]);
  assert.deepEqual(
    fixtureSymbol(mover),
    fixtureSymbol({ ...mover, attributes: [...mover.attributes].reverse() }),
  );
  assert.equal(fixtureSymbol(mover).label, "双轴灯具 · 混色 · 色盘");
  assert.equal(fixtureSymbol(undefined).label, "通用灯具");
});
test("缺配适不伪造地址且名字和地址标注可独立切换", () => {
  const f = fixture("1", ["dimmer"]);
  assert.equal(fixtureAddress(f), "1.007");
  assert.equal(fixtureAddress({ ...f, universe: null }), "未配适");
  assert.equal(fixtureAddress({ ...f, address: null }), "未配适");
  assert.equal(fixturePlanLabel(f, "none"), "");
  assert.equal(fixturePlanLabel(f, "name"), "灯具 1");
  assert.equal(fixturePlanLabel(f, "address"), "1.007");
  assert.equal(fixturePlanLabel(undefined, "address"), "未配适");
});
test("图例仅统计当前可见灯位，隐藏与重复身份不影响工程或有序选择", () => {
  const fixtures = [
    fixture("1", ["dimmer"]),
    fixture("2", ["pan", "tilt", "dimmer"]),
    fixture("3", ["pan", "tilt", "dimmer"]),
  ];
  const ids = ["3", "2", "3", "missing"],
    before = structuredClone({ fixtures, ids });
  const legend = fixtureLegend(fixtures, ids);
  assert.equal(legend.length, 1);
  assert.equal(legend[0]!.count, 2);
  assert.equal(legend[0]!.symbol.label, "双轴灯具");
  assert.deepEqual({ fixtures, ids }, before);
  assert.equal(fixtureLegend(fixtures, []).length, 0);
});

test("显示的补零配适地址也能直接搜索，并保留旧短地址输入", () => {
  const f = fixture("1", ["dimmer"]);
  assert.equal(fixtureMatches(f, "1.007"), true);
  assert.equal(fixtureMatches(f, "1.7"), true);
  assert.equal(fixtureMatches(f, "1.008"), false);
  assert.equal(fixtureMatches({ ...f, address: null }, "未配适"), true);
});
