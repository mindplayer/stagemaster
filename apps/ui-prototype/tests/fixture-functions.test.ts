import test from "node:test";
import assert from "node:assert/strict";
import {
  profileDraft,
  profileDefinition,
  withMotion,
  compatibleProfile,
  FixtureFieldError,
} from "../src/fixture-tools.ts";
import {
  addFunctionChannel,
  withLinearFamily,
  draftInitial,
} from "../src/fixture-function-draft.ts";
import { initialFunction } from "../src/fixture-function-types.ts";
import { commonAttributes, parameterCommands } from "../src/editor-tools.ts";
import { functionState } from "../src/function-parameter-tools.ts";
import { sceneValueLabel } from "../src/library-tools.ts";
import { packageTargetReason } from "../src/installation-tools.ts";
import type { FixtureView, SceneView } from "../src/application-host";
function definition() {
  const draft = addFunctionChannel(
    withLinearFamily(profileDraft(), "dimmer"),
    "shutter",
  );
  draft.channels.at(-1)!.functions = [
    {
      key: "closed",
      name: "关闭",
      mode: "slot",
      dmxFrom: "0",
      dmxTo: "15",
      dmxDefault: "0",
    },
    {
      key: "strobe",
      name: "频闪",
      mode: "range",
      dmxFrom: "32",
      dmxTo: "239",
      dmxDefault: "64",
    },
  ];
  draft.channels.at(-1)!.defaultFunction = {
    functionKey: "strobe",
    position: "12345",
  };
  return profileDefinition(draft);
}
function fixture(id = "one"): FixtureView {
  const p = definition(),
    c = p.channels[1];
  return {
    id,
    profileId: "profile",
    name: id,
    profileName: p.name,
    domainId: "domain",
    domainName: "域",
    address: 1,
    universe: 1,
    footprint: p.footprint,
    attributes: [
      { key: "dimmer", label: "亮度", defaultValue: 0 },
      {
        key: "shutter",
        label: "快门与频闪",
        defaultValue: 0,
        function: {
          functions: c.functions!,
          default: { functionKey: "strobe", position: 12345 },
          fine: false,
        },
      },
    ],
  };
}
test("预设明细使用类型化功能身份，不把色盘档位显示成归一化百分比", () => {
  const value: SceneView["values"][number] = {
    fixtureId: "one",
    attribute: "shutter",
    mode: "set",
    value: 34952,
    functionValue: { functionKey: "strobe", position: 32768 },
  };
  assert.equal(sceneValueLabel(value, [fixture()]), "频闪 · 50.00%");
  assert.equal(
    sceneValueLabel(
      { ...value, functionValue: { functionKey: "closed", position: 0 } },
      [fixture()],
    ),
    "关闭",
  );
  assert.equal(sceneValueLabel(value, []), "功能定义不可用");
  assert.equal(
    sceneValueLabel({ ...value, mode: "release" }, [fixture()]),
    "释放",
  );
  assert.equal(
    sceneValueLabel({ ...value, functionValue: null, value: 65535 }, [
      fixture(),
    ]),
    "100.00%",
  );
});
test("功能档案保留非代表默认位置、任意粗细地址，复制和组合切换不丢区间", () => {
  const p = definition();
  assert.deepEqual(profileDefinition(profileDraft(p)), p);
  const d = withMotion(profileDraft(p), true);
  d.channels[1].bits = "16";
  d.channels[1].fine = "9";
  d.footprint = "9";
  const next = withLinearFamily(d, "rgbd");
  assert.deepEqual(
    next.channels.find((c) => c.attribute === "shutter"),
    d.channels[1],
  );
  assert.deepEqual(next.positioning, d.positioning);
  const parsed = profileDefinition(next);
  assert.deepEqual(
    parsed.channels.find((c) => c.attribute === "shutter")!.defaultValue,
    p.channels[1].defaultValue,
  );
  assert.deepEqual(profileDefinition(profileDraft(parsed)), parsed);
  assert.equal(addFunctionChannel(next, "shutter"), next);
});
test("功能表错误定位到具体区间；不把隐藏细调或已删除默认选择静默转换", () => {
  for (const [mutate, field] of [
    [
      (d: ReturnType<typeof profileDraft>) => {
        d.channels[1].functions![1].dmxFrom = "15";
      },
      "function-1-dmxFrom",
    ],
    [
      (d: ReturnType<typeof profileDraft>) => {
        d.channels[1].functions![1].dmxDefault = "255";
      },
      "function-1-dmxDefault",
    ],
    [
      (d: ReturnType<typeof profileDraft>) => {
        d.channels[1].functions![1].dmxTo = "999";
      },
      "function-1-dmxTo",
    ],
    [
      (d: ReturnType<typeof profileDraft>) => {
        d.channels[1].functions![1].name = "";
      },
      "function-1-name",
    ],
    [
      (d: ReturnType<typeof profileDraft>) => {
        d.channels[1].functions!.pop();
      },
      "default-function",
    ],
    [
      (d: ReturnType<typeof profileDraft>) => {
        d.channels[1].defaultFunction = {
          functionKey: "closed",
          position: "1",
        };
      },
      "default-position",
    ],
  ] as const) {
    const d = profileDraft(definition());
    mutate(d);
    assert.throws(
      () => profileDefinition(d),
      (e) => e instanceof FixtureFieldError && e.field.endsWith(field),
    );
  }
});
test("功能切换使用代表值；所有代表值在 8 位区间内往返一致", () => {
  for (let v = 32; v <= 239; v++) {
    const f = {
      key: "s",
      name: "s",
      mode: "range" as const,
      dmxFrom: 32,
      dmxTo: 239,
      dmxDefault: v,
    };
    const selected = initialFunction(f);
    assert.equal(Math.round((selected.position * 207) / 65535) + 32, v);
    assert.equal(
      draftInitial({ ...f, dmxFrom: "32", dmxTo: "239", dmxDefault: String(v) })
        .position,
      String(selected.position),
    );
  }
});
test("批量选择只开放语义一致的功能；地址变化允许，功能差异禁止安全换灯", () => {
  const a = fixture(),
    b = fixture("two"),
    p = definition();
  assert.equal(commonAttributes([a, b]).length, 2);
  p.channels[1].coarse = 12;
  assert.equal(compatibleProfile([a, b], p), true);
  b.attributes[1].function!.functions[1].dmxDefault++;
  assert.equal(commonAttributes([a, b]).length, 1);
  assert.equal(compatibleProfile([a, b], p), false);
  assert.throws(
    () =>
      parameterCommands("scene", [a, b], {
        shutter: { function: { functionKey: "closed", position: 0 } },
      }),
    /选择范围/,
  );
});
test("类型化场景编辑保留选择与预设来源差异，不用编码后的值代替功能", () => {
  const a = fixture(),
    b = fixture("two");
  const scene: SceneView = {
    id: "scene",
    name: "场景",
    effects: [],
    values: [
      {
        fixtureId: a.id,
        attribute: "shutter",
        mode: "preset",
        value: 123,
        presetId: "p",
        presetName: "频闪",
        functionValue: { functionKey: "strobe", position: 12345 },
      },
    ],
  };
  assert.equal(functionState(scene, [a, b], "shutter").mixed, true);
  const commands = parameterCommands(scene.id, [a, b], {
    shutter: {
      function: { functionKey: "strobe", position: 0 },
      percent: "50",
    },
    dimmer: "80",
  });
  assert.deepEqual(commands[0], {
    op: "setSceneFunctionValue",
    sceneId: "scene",
    fixtureId: a.id,
    attribute: "shutter",
    selection: { functionKey: "strobe", position: 32768 },
  });
  assert.equal(commands.length, 4);
  for (const percent of ["", "-1", "101", "NaN"])
    assert.throws(() =>
      parameterCommands("scene", [a], {
        shutter: { function: { functionKey: "strobe", position: 0 }, percent },
      }),
    );
  assert.throws(() => parameterCommands("scene", [a], { shutter: 65535 }));
  assert.equal(
    parameterCommands("scene", [a], { shutter: { mode: "remove" } })[0].op,
    "setSceneValue",
  );
});
test("下发入口按旧目标默认能力拒绝新功能包", () => {
  assert.equal(packageTargetReason(1, 1), null);
  assert.match(packageTargetReason(2)!, /固件/);
  assert.equal(packageTargetReason(2, 2), null);
});
