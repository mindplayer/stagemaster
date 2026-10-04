import test from "node:test";
import assert from "node:assert/strict";
import {
  profileDraft,
  profileDefinition,
  FixtureFieldError,
} from "../src/fixture-tools.ts";
import {
  addFunctionChannel,
  withLinearFamily,
} from "../src/fixture-function-draft.ts";
import {
  addProgramChannel,
  newProgramFunction,
  programKind,
} from "../src/fixture-program.ts";
import { programSelectionAllowed } from "../src/fixture-program-rules.ts";
import { parameterCategory } from "../src/parameter-categories.ts";
import { presetScopeOptions } from "../src/preset-attribute-scopes.ts";
import { commonAttributes, parameterCommands } from "../src/editor-tools.ts";
import { sceneValueLabel } from "../src/library-tools.ts";
import type { FixtureView } from "../src/application-host";

function draft() {
  const d = addProgramChannel(withLinearFamily(profileDraft(), "dimmer"));
  const c = d.channels.at(-1)!;
  c.functions = [
    {
      ...newProgramFunction("external"),
      dmxFrom: "0",
      dmxTo: "59",
      dmxDefault: "0",
    },
  ];
  for (const [kind, first, last] of [
    ["auto", 60, 159],
    ["sound", 160, 255],
  ] as const)
    for (let n = 0; n < 4; n++)
      c.functions.push({
        ...newProgramFunction(kind),
        key: `${kind}.${3 - n}`,
        name: `${kind === "auto" ? "自动" : "声控"} ${3 - n}`,
        dmxFrom: String(first + 25 * n),
        dmxTo: String(n === 3 ? last : first + 25 * n + 24),
        dmxDefault: String(first + 25 * n),
      });
  return d;
}
function fixture(id: string): FixtureView {
  const def = profileDefinition(draft()),
    c = def.channels.at(-1)!;
  return {
    id,
    profileId: "profile",
    name: id,
    profileName: def.name,
    domainId: "domain",
    domainName: "域",
    universe: 1,
    address: id === "a" ? 17 : 28,
    footprint: def.footprint,
    attributes: [
      {
        key: "fixture-program",
        label: "内置程序",
        defaultValue: 0,
        function: {
          functions: c.functions!,
          default: { functionKey: "external", position: 0 },
          fine: false,
        },
      },
    ],
  };
}
test("新建程序默认只声明外部控制，原生区间为空且普通功能入口不能绕过专用规则", () => {
  const initial = profileDraft(),
    before = structuredClone(initial),
    d = addProgramChannel(initial),
    c = d.channels.at(-1)!;
  assert.deepEqual(initial, before);
  assert.equal(addProgramChannel(d), d);
  assert.equal(addFunctionChannel(initial, "fixture-program"), initial);
  assert.deepEqual(c.defaultFunction, {
    functionKey: "external",
    position: "0",
  });
  assert.deepEqual(
    [
      c.functions![0].dmxFrom,
      c.functions![0].dmxTo,
      c.functions![0].dmxDefault,
    ],
    ["", "", ""],
  );
  assert.throws(
    () => profileDefinition(d),
    (e) => e instanceof FixtureFieldError && e.field.endsWith("dmxFrom"),
  );
  for (const kind of ["auto", "sound"] as const) {
    const f = newProgramFunction(kind);
    assert.ok(f.key.startsWith(`${kind}.`));
    assert.equal(f.mode, "slot");
    assert.equal(f.name, "");
  }
});
test("已知九档精确往返，复制及基础组合切换保持程序身份与独立控制分类", () => {
  const d = draft(),
    def = profileDefinition(d);
  assert.equal(def.channels.at(-1)!.functions!.length, 9);
  assert.deepEqual(profileDefinition(profileDraft(def)), def);
  const rgb = withLinearFamily(d, "rgbd");
  assert.deepEqual(
    rgb.channels.find((c) => c.attribute === "fixture-program"),
    d.channels.at(-1),
  );
  assert.equal(parameterCategory("fixture-program"), "control");
  const scopes = presetScopeOptions(
    ["dimmer", "pan", "tilt", "pan-tilt-speed", "fixture-program"].map(
      (key) => ({ key }),
    ),
  );
  assert.deepEqual(scopes.find((s) => s.id === "program")!.keys, [
    "fixture-program",
  ]);
  assert.deepEqual(scopes.find((s) => s.id === "position")!.keys, [
    "pan",
    "tilt",
  ]);
  assert.deepEqual(scopes.find((s) => s.id === "light")!.keys, ["dimmer"]);
  assert.equal(programKind("reset"), "未知程序");
});
test("非法程序草稿定位具体字段，不静默默认自走或转换复位", () => {
  for (const [mutate, suffix] of [
    [
      (d: ReturnType<typeof draft>) => {
        d.channels.at(-1)!.defaultFunction!.functionKey = "auto.3";
      },
      "default-function",
    ],
    [
      (d: ReturnType<typeof draft>) => {
        d.channels.at(-1)!.functions![1].mode = "range";
      },
      "function-1-mode",
    ],
    [
      (d: ReturnType<typeof draft>) => {
        d.channels.at(-1)!.functions![1].key = "reset";
      },
      "function-1-name",
    ],
    [
      (d: ReturnType<typeof draft>) => {
        d.channels.at(-1)!.functions!.shift();
        d.channels.at(-1)!.defaultFunction!.functionKey = "auto.3";
      },
      "function-add",
    ],
    [
      (d: ReturnType<typeof draft>) => {
        d.channels.at(-1)!.defaultFunction!.position = "1";
      },
      "default-position",
    ],
    [
      (d: ReturnType<typeof draft>) => {
        d.channels.at(-1)!.functions![1].dmxFrom = "59";
      },
      "function-1-dmxFrom",
    ],
  ] as const) {
    const d = draft();
    mutate(d);
    assert.throws(
      () => profileDefinition(d),
      (e) => e instanceof FixtureFieldError && e.field.endsWith(suffix),
    );
  }
});
test("两灯程序选择生成类型化批次，百分比与不相容功能不混入共同能力", () => {
  const a = fixture("a"),
    b = fixture("b"),
    fixtures = [a, b];
  assert.equal(commonAttributes(fixtures).length, 1);
  const commands = parameterCommands("scene", fixtures, {
    "fixture-program": { function: { functionKey: "external", position: 0 } },
  });
  assert.equal(commands.length, 2);
  for (const c of commands) assert.equal(c.op, "setSceneFunctionValue");
  assert.throws(() =>
    parameterCommands("scene", fixtures, { "fixture-program": 32768 }),
  );
  b.attributes[0].function!.functions[1].dmxDefault++;
  assert.equal(commonAttributes(fixtures).length, 0);
  assert.throws(() =>
    parameterCommands("scene", fixtures, {
      "fixture-program": { function: { functionKey: "external", position: 0 } },
    }),
  );
  assert.equal(
    sceneValueLabel(
      {
        fixtureId: "a",
        attribute: "fixture-program",
        mode: "set",
        value: 15420,
        functionValue: { functionKey: "external", position: 0 },
      },
      [a],
    ),
    "外部通道控制",
  );
});

test("声控和内置自走仅作禁用资料，命令入口不接受任何自主档位", () => {
  const fixtures = [fixture("a"), fixture("b")];
  assert.equal(programSelectionAllowed("external"), true);
  for (const kind of ["auto", "sound"] as const)
    for (let number = 0; number < 4; number++) {
      const key = `${kind}.${number}`;
      assert.equal(programSelectionAllowed(key), false);
      assert.throws(
        () =>
          parameterCommands("scene", fixtures, {
            "fixture-program": { function: { functionKey: key, position: 0 } },
          }),
        /已屏蔽/,
      );
    }
});
