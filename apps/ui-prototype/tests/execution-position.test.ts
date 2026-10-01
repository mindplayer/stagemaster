import { test } from "node:test";
import assert from "node:assert/strict";
import type { PreviewSnapshot } from "../src/sequence-types.ts";
import {
  executionPhase,
  executionPosition,
} from "../src/components/workbench/execution-position.ts";

const loaded: NonNullable<PreviewSnapshot["loaded"]> = {
  sequenceId: "list",
  sceneId: null,
  name: "演出",
  sourceRevision: "1",
  status: "idle",
  stepId: null,
  elapsedMs: 0,
  ratePercent: 100,
  delayMs: 500,
  fadeMs: 1000,
  waitMs: null,
  stale: false,
  canNext: true,
  bufferBytes: 0,
  effectBufferBytes: 0,
  steps: [
    { id: "a", name: "序幕", number: "1" },
    { id: "b", name: "对白", number: "2" },
  ],
  output: { universe: 1, slots: [], fixtures: [] },
};
test("未载入和单场景预演不被当成执行列表", () => {
  assert.equal(executionPosition(null).sequenceId, null);
  assert.equal(executionPosition({ ...loaded, sceneId: "scene" }).nextId, null);
});
test("载入就绪只显示下一步骤，选择并未成为正在执行", () => {
  assert.deepEqual(executionPosition(loaded), {
    sequenceId: "list",
    currentId: null,
    nextId: "a",
    status: "idle",
    stale: false,
  });
});
test("运行、暂停和旧版本从载入快照确定当前与下一步", () => {
  const p = executionPosition({
    ...loaded,
    status: "paused",
    stepId: "a",
    stale: true,
  });
  assert.equal(p.currentId, "a");
  assert.equal(p.nextId, "b");
  assert.equal(p.status, "paused");
  assert.equal(p.stale, true);
});
test("末尾是否回到首步由核心 canNext 决定，不猜测编辑列表重复模式", () => {
  assert.equal(
    executionPosition({ ...loaded, stepId: "b", canNext: false }).nextId,
    null,
  );
  assert.equal(
    executionPosition({ ...loaded, stepId: "b", canNext: true }).nextId,
    "a",
  );
  assert.equal(
    executionPosition({ ...loaded, stepId: "missing" }).nextId,
    null,
  );
});
test("延时和渐变分别显示当前阶段进度而非虚构全场进度", () => {
  assert.deepEqual(executionPhase({ ...loaded, stepId: "a", elapsedMs: 250 }), {
    label: "延时",
    elapsed: 250,
    total: 500,
    progress: 0.5,
  });
  assert.deepEqual(executionPhase({ ...loaded, stepId: "a", elapsedMs: 750 }), {
    label: "渐变",
    elapsed: 250,
    total: 1000,
    progress: 0.25,
  });
});
test("人工等待没有倒计时，自动等待从渐变完成后计时", () => {
  assert.deepEqual(
    executionPhase({ ...loaded, stepId: "a", elapsedMs: 4500 }),
    {
      label: "等待人工推进",
      elapsed: 3000,
      total: 0,
      progress: 1,
    },
  );
  assert.deepEqual(
    executionPhase({ ...loaded, stepId: "a", elapsedMs: 2000, waitMs: 1000 }),
    {
      label: "自动等待",
      elapsed: 500,
      total: 1000,
      progress: 0.5,
    },
  );
});
