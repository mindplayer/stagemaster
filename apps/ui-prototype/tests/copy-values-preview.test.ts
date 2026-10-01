import test from "node:test";
import assert from "node:assert/strict";
import type { FixtureView, SceneView } from "../src/application-host";
import {
  copyValuesPreview,
  copySubmissionIssue,
} from "../src/copy-values-preview.ts";

const attrs = ["dimmer", "red", "zoom"].map((key) => ({
  key,
  label: key,
  defaultValue: 0,
}));
const source: FixtureView = {
  id: "a",
  name: "来源",
  profileId: "p",
  profileName: "模式",
  domainId: "d",
  domainName: "灯光",
  universe: 1,
  address: 1,
  footprint: 3,
  attributes: attrs,
};
const target = { ...source, id: "b", name: "目标" };
const scene: SceneView = {
  id: "s",
  name: "场景",
  effects: [],
  values: [
    {
      fixtureId: "a",
      attribute: "dimmer",
      mode: "preset",
      value: 12345,
      presetId: "p",
      presetName: "亮度",
    },
    {
      fixtureId: "a",
      attribute: "red",
      mode: "release",
      value: 0,
      presetId: null,
      presetName: null,
    },
  ],
};
test("复制预检保留引用解析值，只复制记录项且不静默删除目标不支持的掩码", () => {
  const before = structuredClone(scene);
  const p = copyValuesPreview(
    scene,
    source,
    [source, target, { ...target, id: "c", attributes: attrs.slice(0, 1) }],
    ["dimmer", "red", "zoom", "dimmer"],
  );
  assert.deepEqual(p.keys, ["dimmer", "red", "zoom"]);
  assert.equal(p.values.length, 1);
  assert.equal(p.values[0].value, 12345);
  assert.deepEqual(p.skipped, ["red", "zoom"]);
  assert.equal(p.targets.length, 2);
  assert.deepEqual(p.targets[0].issues, []);
  assert.deepEqual(p.targets[1].issues, ["缺少红色", "缺少变焦"]);
  assert.equal(copySubmissionIssue(p, ["b"]), "");
  assert.match(copySubmissionIssue(p, ["b", "c"]), /1 台已选目标/);
  assert.match(copySubmissionIssue(p, ["a"]), /至少选择/);
  assert.deepEqual(scene, before);
});
test("复制预检保持空范围、无记录及缺失来源，并限制整批写入容量", () => {
  assert.match(
    copySubmissionIssue(copyValuesPreview(scene, source, [target], []), ["b"]),
    /请选择复制属性/,
  );
  assert.match(
    copySubmissionIssue(copyValuesPreview(scene, source, [target], ["zoom"]), [
      "b",
    ]),
    /没有已记录/,
  );
  assert.match(
    copySubmissionIssue(
      copyValuesPreview(scene, undefined, [target], ["dimmer"]),
      ["b"],
    ),
    /来源灯具/,
  );
  assert.match(
    copySubmissionIssue(
      copyValuesPreview(scene, source, [target], ["missing"]),
      ["b"],
    ),
    /来源缺少/,
  );
  const candidates = Array.from({ length: 10001 }, (_, i) => ({
    ...target,
    id: `t${i}`,
  }));
  const p = copyValuesPreview(scene, source, candidates, ["dimmer"]);
  assert.equal(
    copySubmissionIssue(
      p,
      candidates.slice(0, 10000).map((f) => f.id),
    ),
    "",
  );
  assert.match(
    copySubmissionIssue(
      p,
      candidates.map((f) => f.id),
    ),
    /10000/,
  );
});
test("功能值按目标定义匹配；不同物理通道仍可复制，缺项与非零离散位置明确阻止", () => {
  const range = {
    key: "color-wheel",
    label: "色盘",
    defaultValue: 0,
    function: {
      fine: false,
      default: { functionKey: "spin", position: 0 },
      functions: [
        {
          key: "spin",
          name: "旋转",
          mode: "range" as const,
          dmxFrom: 0,
          dmxTo: 255,
          dmxDefault: 0,
        },
      ],
    },
  };
  const src = { ...source, attributes: [range] };
  const s = {
    ...scene,
    values: [
      {
        ...scene.values[0],
        attribute: "color-wheel",
        functionValue: { functionKey: "spin", position: 30000 },
        value: 30000,
      },
    ],
  };
  const shifted = {
    ...target,
    attributes: [
      {
        ...range,
        function: {
          ...range.function,
          fine: true,
          functions: [
            {
              ...range.function.functions[0],
              dmxFrom: 400,
              dmxTo: 800,
              dmxDefault: 400,
            },
          ],
        },
      },
    ],
  };
  const slot = {
    ...target,
    id: "slot",
    attributes: [
      {
        ...range,
        function: {
          ...range.function,
          functions: [
            { ...range.function.functions[0], mode: "slot" as const },
          ],
        },
      },
    ],
  };
  const missing = {
    ...target,
    id: "missing",
    attributes: [{ ...range, function: { ...range.function, functions: [] } }],
  };
  const plain = {
    ...target,
    id: "plain",
    attributes: [{ key: "color-wheel", label: "色盘", defaultValue: 0 }],
  };
  const p = copyValuesPreview(
    s,
    src,
    [shifted, slot, missing, plain],
    ["color-wheel"],
  );
  assert.deepEqual(p.targets[0].issues, []);
  assert.match(p.targets[1].issues[0], /固定档位/);
  assert.match(p.targets[2].issues[0], /没有功能/);
  assert.match(p.targets[3].issues[0], /功能值不能写入连续/);
  const literal = { ...s, values: [{ ...s.values[0], functionValue: null }] };
  assert.match(
    copyValuesPreview(literal, src, [shifted], ["color-wheel"]).targets[0]
      .issues[0],
    /连续值不能写入功能/,
  );
});
