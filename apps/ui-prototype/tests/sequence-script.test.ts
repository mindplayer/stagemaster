import test from "node:test";
import assert from "node:assert/strict";
import {
  sequenceCommands,
  sequenceDraft,
  SequenceInputError,
} from "../src/sequence-tools.ts";
import { stepMatches } from "../src/sequence-script-tools.ts";
import type { SequenceView } from "../src/sequence-types";
const sequence: SequenceView = {
  id: "list",
  name: "戏剧",
  tracking: "inherited",
  repeat: "once",
  steps: [
    {
      id: "a",
      name: "开幕",
      number: "1",
      sceneId: "scene",
      delayMs: 0,
      fadeMs: 1000,
      waitMs: null,
      script: {
        section: "第一幕",
        trigger: "演员：现在开始。\n举手时执行",
        notes: "排练：等掌声",
      },
    },
    {
      id: "b",
      name: "谢幕",
      number: "2",
      sceneId: "scene",
      delayMs: 0,
      fadeMs: 1000,
      waitMs: null,
    },
  ],
};
test("提示与其他步骤修改同批提交，旧步骤编辑不会清空提示", () => {
  const step = sequence.steps[0],
    draft = sequenceDraft(sequence, step);
  assert.deepEqual(sequenceCommands(draft, sequence, step), []);
  const commands = sequenceCommands(
    {
      ...draft,
      name: "新开幕",
      position: "2",
      scriptTrigger: "灯亮后，演员：开始。",
    },
    sequence,
    step,
  );
  assert.equal(commands.length, 3);
  assert.deepEqual(
    commands.map((c) => (c.op === "sequence" ? c.command.kind : c.op)),
    ["updateStepScript", "updateStep", "moveStep"],
  );
  assert.equal(draft.scriptTrigger, step.script!.trigger);
});
test("清空提示显式用 null，新建与全空草稿没有多余写入", () => {
  const step = sequence.steps[0],
    draft = sequenceDraft(sequence, step);
  const commands = sequenceCommands(
    { ...draft, scriptSection: "", scriptTrigger: "", scriptNotes: "" },
    sequence,
    step,
  );
  assert.deepEqual(commands, [
    {
      op: "sequence",
      command: {
        kind: "updateStepScript",
        id: "list",
        stepId: "a",
        script: null,
      },
    },
  ]);
  const plain = sequence.steps[1];
  assert.deepEqual(
    sequenceCommands(sequenceDraft(sequence, plain), sequence, plain),
    [],
  );
});
test("中文多行、Unicode 长度与控制字符验证定位具体字段", () => {
  const step = sequence.steps[0],
    d = sequenceDraft(sequence, step);
  assert.doesNotThrow(() =>
    sequenceCommands({ ...d, scriptSection: "🎭".repeat(80) }, sequence, step),
  );
  for (const [field, value] of [
    ["scriptSection", "字".repeat(81)],
    ["scriptTrigger", "字".repeat(1025)],
    ["scriptNotes", "\u0085"],
  ] as const)
    assert.throws(
      () => sequenceCommands({ ...d, [field]: value }, sequence, step),
      (e) => e instanceof SequenceInputError && e.field === field,
    );
});
test("步骤检索覆盖幕场、台词与备注，筛选不修改步骤", () => {
  const step = sequence.steps[0];
  for (const query of ["  第一幕 ", "举手", "掌声", "SCENE", "1", "开幕"])
    assert.ok(stepMatches(step, "scene", query));
  assert.equal(stepMatches(step, "scene", "不存在"), false);
  assert.equal(step.script!.notes, "排练：等掌声");
});
