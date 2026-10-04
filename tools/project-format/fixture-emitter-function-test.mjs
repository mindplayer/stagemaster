import test from "node:test";
import assert from "node:assert/strict";
import { auditProject, loadExamples } from "./check.mjs";
function document() {
  const p = structuredClone(
    loadExamples().find((p) => p.project?.name === "单路灯光示例"),
  );
  const profile = p.lighting.profiles[0];
  profile.emitters = [
    { key: "pattern", name: "图案光源" },
    { key: "wash", name: "染色光源" },
  ];
  profile.attributes = ["emitter.pattern.dimmer", "emitter.wash.dimmer"].map(
    (key) => ({
      key,
      mix: "htp",
      valueType: { kind: "normalized" },
      default: { kind: "normalized", value: 65535 },
    }),
  );
  profile.channels = profile.attributes.map((a, i) => ({
    attribute: a.key,
    encoding: "u8",
    offsets: [i],
  }));
  for (const owner of ["pattern", "wash"]) {
    const key = `emitter.${owner}.shutter`;
    profile.attributes.push({
      key,
      mix: "ltp",
      valueType: { kind: "function" },
      default: { kind: "function", functionKey: "open", position: 0 },
    });
    profile.channels.push({
      attribute: key,
      encoding: "u8",
      offsets: [profile.channels.length],
      functions: [
        {
          key: "open",
          name: "开光",
          mode: "slot",
          dmxFrom: 0,
          dmxTo: 15,
          dmxDefault: 0,
        },
        {
          key: "strobe",
          name: "受控频闪",
          mode: "range",
          dmxFrom: 16,
          dmxTo: 255,
          dmxDefault: 16,
        },
      ],
    });
  }
  profile.footprint = 4;
  p.lighting.scenes = [];
  p.lighting.presets = [];
  p.lighting.sequences = [];
  p.entryPoints = [];
  p.requires.push(
    { key: "lighting.fixture-emitters", version: 1 },
    { key: "lighting.fixture-functions", version: 1 },
    { key: "lighting.fixture-emitter-functions", version: 1 },
  );
  return p;
}
test("独立快门功能有归属和明确能力", () =>
  assert.doesNotThrow(() => auditProject(document())));
for (const [name, mutate] of [
  ["缺能力", (p) => p.requires.pop()],
  ["未知能力版本", (p) => (p.requires.at(-1).version = 2)],
  [
    "声控",
    (p) => (p.lighting.profiles[0].channels[2].functions[1].key = "sound-0"),
  ],
  [
    "自走",
    (p) => (p.lighting.profiles[0].channels[2].functions[1].key = "auto-0"),
  ],
  [
    "复位",
    (p) => (p.lighting.profiles[0].channels[2].functions[1].key = "reset"),
  ],
  [
    "伪装固定频闪",
    (p) => (p.lighting.profiles[0].channels[2].functions[1].mode = "slot"),
  ],
  [
    "普通数值",
    (p) =>
      (p.lighting.profiles[0].attributes[2].default = {
        kind: "normalized",
        value: 0,
      }),
  ],
  ["错误混合", (p) => (p.lighting.profiles[0].attributes[2].mix = "htp")],
  [
    "无连续光源",
    (p) => {
      p.lighting.profiles[0].attributes.splice(0, 2);
      p.lighting.profiles[0].channels.splice(0, 2);
    },
  ],
])
  test(`独立光源功能拒绝${name}`, () => {
    const p = document();
    mutate(p);
    assert.throws(() => auditProject(p));
  });
