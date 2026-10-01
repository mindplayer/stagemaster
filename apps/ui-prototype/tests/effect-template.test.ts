import test from "node:test";
import assert from "node:assert/strict";
import {
  canExportEffectTemplate,
  effectTemplateContext,
} from "../src/effect-template-tools.ts";
import {
  createEffect,
  reuseEffect,
  effectCommands,
} from "../src/effect-tools.ts";
import type { EffectTemplateSource } from "../src/effect-template-types.ts";
test("模板导出仅接受完整单一亮度基础曲线，其他类型不悄悄删字段", () => {
  const e = createEffect("breathe", "a", ["f"]);
  assert.equal(canExportEffectTemplate(e), true);
  assert.equal(canExportEffectTemplate({ ...e, enabled: false }), true);
  for (const key of ["color", "multicolor", "worldLine", "panSweep"] as const) {
    assert.equal(canExportEffectTemplate(createEffect(key, "a", ["f"])), false);
  }
  assert.equal(
    canExportEffectTemplate({
      ...e,
      channels: [...e.channels, { attribute: "red", low: 0, high: 100 }],
    }),
    false,
  );
});
test("灯序和可见上下文分别改变绑定意图，效果副本保留独立来源快照", () => {
  assert.notEqual(
    effectTemplateContext("scene", ["a", "b"], true),
    effectTemplateContext("scene", ["b", "a"], true),
  );
  assert.notEqual(
    effectTemplateContext("scene", ["a"], true),
    effectTemplateContext("other", ["a"], true),
  );
  const e = createEffect("breathe", "a", ["f"]);
  const source: EffectTemplateSource = {
    sha256: "a".repeat(64),
    template: {
      format: "stagemaster-effect-template",
      formatVersion: 1,
      templateId: "t",
      revision: "r",
      definition: {
        name: "来源",
        recipe: {
          kind: "intensity-wave",
          waveform: "smooth",
          low: 0,
          high: 65535,
          dutyPercent: 50,
        },
        timing: {
          periodMs: 2000,
          phaseDegrees: 0,
          spreadDegrees: 0,
          reverseOrder: false,
        },
      },
    },
  };
  e.templateSource = source;
  const copy = reuseEffect(e, "b", ["g"]);
  assert.deepEqual(copy.templateSource, source);
  assert.notEqual(copy.templateSource, source);
  copy.periodMs = 3000;
  const commands = effectCommands(
    "scene",
    copy,
    [
      {
        id: "g",
        name: "灯",
        attributes: [{ key: "dimmer", label: "亮度", defaultValue: 0 }],
      } as Parameters<typeof effectCommands>[2][number],
    ],
    false,
  );
  assert.equal(commands[0].op, "effect");
  if (commands[0].op === "effect" && commands[0].command.kind === "put")
    assert.deepEqual(commands[0].command.effect.templateSource, source);
});
