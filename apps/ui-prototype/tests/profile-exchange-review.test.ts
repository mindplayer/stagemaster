import test from "node:test";
import assert from "node:assert/strict";
import { compatibleProfile } from "../src/fixture-tools.ts";
import { profileExchangeReview } from "../src/profile-exchange-review.ts";
import type { FixtureView, ProjectView } from "../src/application-host.ts";
import type { ProfileView } from "../src/fixture-types.ts";
import type { FunctionDefinition } from "../src/fixture-function-types.ts";
const slots: FunctionDefinition[] = [
  {
    key: "one",
    name: "第一档",
    mode: "slot",
    dmxFrom: 0,
    dmxTo: 9,
    dmxDefault: 4,
  },
  {
    key: "two",
    name: "第二档",
    mode: "slot",
    dmxFrom: 10,
    dmxTo: 19,
    dmxDefault: 14,
  },
  {
    key: "auto",
    name: "自动",
    mode: "range",
    dmxFrom: 140,
    dmxTo: 255,
    dmxDefault: 140,
  },
];
function setup() {
  const source: ProfileView = {
    id: "old",
    revision: "1",
    authorable: true,
    name: "原模式",
    manufacturer: "测试",
    model: "测试",
    mode: "色盘",
    footprint: 2,
    channels: [
      { attribute: "dimmer", coarse: 1, fine: null, defaultValue: 0 },
      {
        attribute: "color-wheel",
        coarse: 2,
        fine: null,
        defaultValue: { functionKey: "one", position: 0 },
        functions: structuredClone(slots),
      },
    ],
  };
  const target = structuredClone(source);
  target.id = "new";
  target.name = "新模式";
  target.channels[1].functions![1].dmxDefault = 15;
  const fixture: FixtureView = {
    id: "a",
    profileId: source.id,
    profileName: source.name,
    name: "灯甲",
    footprint: 2,
    domainId: "d",
    domainName: "灯光",
    attributes: [
      { key: "dimmer", label: "亮度", defaultValue: 0 },
      {
        key: "color-wheel",
        label: "色盘",
        defaultValue: 0,
        function: {
          functions: structuredClone(slots),
          fine: false,
          default: { functionKey: "one", position: 0 },
        },
      },
    ],
  };
  const project = { profiles: [source, target] } as ProjectView;
  return { source, target, fixture, project };
}
test("固定档位需要明确选择；连续区域、身份和其他功能映射始终受保护", () => {
  const { fixture, target } = setup();
  assert.equal(compatibleProfile([fixture], target), false);
  assert.equal(compatibleProfile([fixture], target, true), true);
  for (const change of [
    (f: FunctionDefinition[]) => {
      f[2].dmxDefault++;
    },
    (f: FunctionDefinition[]) => {
      f[1].key = "different";
    },
    (f: FunctionDefinition[]) => {
      f[1].mode = "range";
    },
    (f: FunctionDefinition[]) => {
      f.pop();
    },
  ]) {
    const t = structuredClone(target);
    change(t.channels[1].functions!);
    assert.equal(compatibleProfile([fixture], t, true), false);
  }
  target.channels[1].attribute = "gobo-wheel";
  fixture.attributes[1].key = "gobo-wheel";
  assert.equal(compatibleProfile([fixture], target, true), false);
});
test("逐原模式展示所选灯及前后值，外观差异不误报控制改变", () => {
  const { fixture, source, target, project } = setup();
  target.channels[1].functions![0].appearance = { kind: "open" };
  const other = structuredClone(fixture);
  other.id = "b";
  other.name = "灯乙";
  const before = structuredClone(project);
  const r = profileExchangeReview(project, [fixture, other], target);
  assert.equal(r.needsRemap, true);
  assert.deepEqual(
    r.groups[0].fixtures.map((f) => f.id),
    ["a", "b"],
  );
  assert.equal(r.groups[0].changes[0].remap, false);
  const changed = r.groups[0].changes[1];
  assert.equal(changed.before.dmxDefault, 14);
  assert.equal(changed.after.dmxDefault, 15);
  assert.equal(changed.remap, true);
  assert.deepEqual(project, before);
  assert.equal(
    profileExchangeReview(project, [fixture], source).needsRemap,
    false,
  );
});
test("审阅身份绑定选择、顺序、模式及源目标修订，空选择不产生假差异", () => {
  const { fixture, source, target, project } = setup();
  const signature = () =>
    profileExchangeReview(project, [fixture], target).signature;
  const first = signature();
  source.revision = "2";
  assert.notEqual(signature(), first);
  const second = signature();
  target.revision = "2";
  assert.notEqual(signature(), second);
  const third = signature();
  fixture.id = "b";
  assert.notEqual(signature(), third);
  assert.deepEqual(profileExchangeReview(project, [], target).groups, []);
});
