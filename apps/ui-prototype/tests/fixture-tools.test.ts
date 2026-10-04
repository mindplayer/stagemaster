import test from "node:test";
import "./fixture-axis-speed.test.ts";
import "./fixture-emitters.test.ts";
import assert from "node:assert/strict";
import {
  profileDraft,
  profileDefinition,
  profileMatches,
  patchPlan,
  availablePatch,
  compatibleProfile,
  FixtureFieldError,
} from "../src/fixture-tools.ts";
import type { FixtureView, ProjectView } from "../src/application-host.ts";
function fixture(
  id: string,
  address: number,
  width = 4,
  domainId = "a",
): FixtureView {
  return {
    id,
    name: id,
    profileId: "p",
    profileName: "RGB",
    domainId,
    domainName: domainId,
    universe: 1,
    address,
    footprint: width,
    attributes: ["dimmer", "red", "green", "blue"].map((key) => ({
      key,
      label: key,
      defaultValue: 0,
    })),
  };
}
const project = {
  fixtures: [
    fixture("first", 1),
    fixture("second", 5),
    fixture("obstacle", 100),
  ],
} as ProjectView;
test("灯具模式草稿保留所有默认值精度、任意粗细顺序与元信息", () => {
  const d = profileDraft();
  d.channels[0] = {
    ...d.channels[0],
    coarse: "8",
    fine: "1",
    bits: "16",
    percent: "50",
  };
  d.footprint = "8";
  const p = profileDefinition(d);
  assert.equal(p.channels[0].defaultValue, 32768);
  assert.equal(p.channels[0].fine, 1);
  for (const n of [
    0, 1, 127, 128, 255, 256, 257, 4660, 32767, 32768, 65534, 65535,
  ]) {
    p.channels[0].defaultValue = n;
    assert.deepEqual(profileDefinition(profileDraft(p)), p);
  }
  assert.equal(
    profileMatches({ ...p, manufacturer: " 测试厂家 " }, "测试"),
    true,
  );
});
test("非法映射给出具体错误字段，拒绝空值、重复、越界及半套 RGB", () => {
  for (const [patch, field] of [
    [{ coarse: "" }, "coarse"],
    [{ coarse: "2" }, "coarse"],
    [{ coarse: "5" }, "coarse"],
    [{ bits: "16", fine: "1" }, "fine"],
    [{ percent: "-1" }, "percent"],
    [{ percent: "101" }, "percent"],
    [{ percent: "" }, "percent"],
  ] as const) {
    const d = profileDraft();
    Object.assign(d.channels[0], patch);
    assert.throws(
      () => profileDefinition(d),
      (e) => e instanceof FixtureFieldError && e.field.endsWith(field),
    );
  }
  const d = profileDraft();
  d.channels.pop();
  assert.throws(() => profileDefinition(d), /RGB/);
});
test("8 位映射不提交隐藏的细调草稿，复制不修改来源", () => {
  const d = profileDraft();
  d.channels[0].fine = "999";
  const before = structuredClone(d);
  assert.equal(profileDefinition(d).channels[0].fine, null);
  assert.deepEqual(d, before);
});
test("批量配适按灯序排布，可覆盖所选灯原地址但不能覆盖第三者", () => {
  const layout = { universe: 1, address: 1, gap: 0 };
  const before = structuredClone(project);
  assert.deepEqual(
    patchPlan(project, ["second", "first"], layout).map((r) => [
      r.fixture.id,
      r.address,
      r.end,
    ]),
    [
      ["second", 1, 4],
      ["first", 5, 8],
    ],
  );
  assert.throws(
    () => patchPlan(project, ["first", "second"], { ...layout, address: 96 }),
    /obstacle/,
  );
  assert.equal(availablePatch(project, ["first", "second"], 1, 0), 1);
  assert.deepEqual(project, before);
});
test("精确 512 边界、间隔和扩宽模式一起计算，不影响别的输出域", () => {
  const p = profileDefinition(profileDraft());
  p.footprint = 6;
  assert.deepEqual(
    patchPlan(
      project,
      ["first", "second"],
      { universe: 1, address: 501, gap: 0 },
      p,
    ).map((r) => r.end),
    [506, 512],
  );
  assert.throws(
    () =>
      patchPlan(
        project,
        ["first", "second"],
        { universe: 1, address: 501, gap: 1 },
        p,
      ),
    /512/,
  );
  assert.throws(() =>
    patchPlan(project, ["first"], { universe: 1, address: NaN, gap: 0 }),
  );
  const blocked = {
    fixtures: [fixture("first", 1), fixture("full", 1, 512, "b")],
  } as ProjectView;
  assert.equal(availablePatch(blocked, ["first"], 1, 0), 1);
  blocked.fixtures[1].domainId = "a";
  assert.equal(availablePatch(blocked, ["first"], 1, 0), null);
});
test("空、重复、过期选择拒绝；替换必须保留相同属性集合", () => {
  for (const ids of [[], ["first", "first"], ["missing"]])
    assert.throws(() =>
      patchPlan(project, ids, { universe: 1, address: 1, gap: 0 }),
    );
  const p = profileDefinition(profileDraft());
  assert.equal(compatibleProfile(project.fixtures, p), true);
  p.channels.pop();
  assert.equal(compatibleProfile(project.fixtures, p), false);
});
import "./fixture-emitter-functions.test.ts";
