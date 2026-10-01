import test from "node:test";
import assert from "node:assert/strict";
import type { StepView } from "../src/sequence-types";
import {
  groupScriptDraft,
  groupScriptCommand,
  GroupScriptError,
} from "../src/sequence-group-script.ts";
const steps: StepView[] = [
  {
    id: "a",
    number: "1",
    name: "一",
    sceneId: "s",
    delayMs: 0,
    fadeMs: 1000,
    waitMs: null,
    script: { section: "第一幕", trigger: "第一句", notes: "" },
  },
  {
    id: "b",
    number: "2",
    name: "二",
    sceneId: "s",
    delayMs: 0,
    fadeMs: 1000,
    waitMs: null,
    script: { section: "第一幕", trigger: "第二句", notes: "" },
  },
];
test("共同值、混合与未填写区别明确，默认所有字段保留且不生成命令", () => {
  const draft = groupScriptDraft(steps);
  assert.deepEqual(draft.fields.section, {
    mode: "keep",
    value: "第一幕",
    mixed: false,
  });
  assert.deepEqual(draft.fields.trigger, {
    mode: "keep",
    value: "",
    mixed: true,
  });
  assert.deepEqual(draft.fields.notes, {
    mode: "keep",
    value: "",
    mixed: false,
  });
  assert.equal(groupScriptCommand("seq", draft), null);
  assert.equal(
    groupScriptDraft([
      { ...steps[0], script: undefined },
      { ...steps[1], script: undefined },
    ]).fields.notes.mixed,
    false,
  );
});
test("按字段生成稀疏修改，清空必须显式，固定目标不受之后的选择修改影响", () => {
  const draft = groupScriptDraft(steps);
  draft.fields.section = {
    mode: "replace",
    value: "第二幕\n终场",
    mixed: false,
  };
  draft.fields.notes = { mode: "clear", value: "ignored", mixed: false };
  const command = groupScriptCommand("seq", draft);
  assert.deepEqual(command, {
    kind: "editSteps",
    id: "seq",
    stepIds: ["a", "b"],
    operation: {
      kind: "script",
      patch: { section: "第二幕\n终场", notes: "" },
    },
  });
  draft.ids.reverse();
  assert.deepEqual(command?.kind === "editSteps" && command.stepIds, [
    "a",
    "b",
  ]);
  assert.equal(steps[0].script?.section, "第一幕");
});
test("混合值空白不能意外清空、无效字段定位，Unicode 按字符计数", () => {
  const draft = groupScriptDraft(steps);
  draft.fields.trigger.mode = "replace";
  assert.throws(
    () => groupScriptCommand("seq", draft),
    (e) =>
      e instanceof GroupScriptError &&
      e.field === "trigger" &&
      /清空/.test(e.message),
  );
  draft.fields.trigger = { mode: "keep", value: "\0", mixed: true };
  draft.fields.section = {
    mode: "replace",
    value: "🎭".repeat(80),
    mixed: false,
  };
  assert.ok(groupScriptCommand("seq", draft));
  draft.fields.section.value += "🎭";
  assert.throws(
    () => groupScriptCommand("seq", draft),
    (e) => e instanceof GroupScriptError && e.field === "section",
  );
  draft.fields.section = { mode: "clear", value: "\0", mixed: false };
  assert.ok(groupScriptCommand("seq", draft));
  for (const ids of [
    [],
    ["a", "a"],
    Array.from({ length: 1025 }, (_, i) => String(i)),
  ])
    assert.throws(
      () => groupScriptCommand("seq", { ...draft, ids }),
      /重新选择/,
    );
});
