import assert from "node:assert/strict";
import test from "node:test";
import {
  sourceProgress,
  progressSeconds,
  type ExecutionSourceState,
} from "../src/execution-source-progress.ts";
import type { ExecutionSource } from "../src/execution-types.ts";
const source: ExecutionSource = {
  id: "list",
  name: "演出",
  priority: 0,
  selection: { kind: "sequence", id: "sequence" },
  steps: [
    { id: "a", name: "开场", number: "1" },
    { id: "b", name: "主段", number: "2" },
  ],
};
function state(): ExecutionSourceState {
  return {
    id: "list",
    level: 65535,
    status: "Running",
    step: "a",
    progress: {
      phase: "fade",
      elapsedMs: "1500",
      phaseElapsedMs: "500",
      phaseDurationMs: "2000",
      nextStep: "b",
      nextWrap: false,
    },
  };
}
test("snapshot labels expose current phase and next step without extrapolation", () => {
  const s = state();
  const original = structuredClone(s);
  assert.deepEqual(sourceProgress(source, s), {
    phase: "fade",
    label: "渐变",
    elapsed: "1.5 秒",
    remaining: "1.5 秒",
    percent: 25,
    next: "2 · 主段",
  });
  assert.deepEqual(s, original);
  s.status = "Paused";
  assert.equal(sourceProgress(source, s)?.label, "已暂停 · 渐变");
  assert.equal(sourceProgress(source, s)?.percent, 25);
});
test("manual final holds have no fake deadline and explicit loops name the first step", () => {
  const s = state();
  s.step = "b";
  Object.assign(s.progress!, {
    phase: "hold",
    phaseElapsedMs: "500",
    phaseDurationMs: null,
    nextStep: null,
  });
  assert.equal(sourceProgress(source, s)?.label, "末步保持");
  assert.equal(sourceProgress(source, s)?.percent, null);
  Object.assign(s.progress!, { nextStep: "a", nextWrap: true });
  assert.equal(sourceProgress(source, s)?.next, "1 · 开场（回到首步）");
  assert.equal(sourceProgress(source, s)?.label, "等待执行下一步");
});
test("legacy missing or invalid host fields never become zero-time progress", () => {
  assert.equal(sourceProgress(source), null);
  const s = state();
  delete s.progress;
  assert.equal(sourceProgress(source, s), null);
  for (const patch of [
    { elapsedMs: "-1" },
    { elapsedMs: "18446744073709551616" },
    { phaseElapsedMs: "5000" },
    { phaseDurationMs: "0" },
    { phaseDurationMs: "500" },
    { nextStep: "gone" },
    { nextWrap: true },
    { phase: "idle" },
  ]) {
    const s = state();
    Object.assign(s.progress!, patch);
    assert.equal(sourceProgress(source, s), null);
  }
});
test("large hold times retain u64 precision and scene holds do not request a next step", () => {
  assert.equal(
    progressSeconds(18446744073709551615n),
    "18446744073709551.6 秒",
  );
  const s = state();
  Object.assign(s.progress!, {
    phase: "hold",
    elapsedMs: "18446744073709551615",
    phaseElapsedMs: "18446744073709551615",
    phaseDurationMs: null,
    nextStep: null,
  });
  assert.equal(
    sourceProgress({ ...source, selection: { kind: "scene", id: "scene" } }, s)
      ?.label,
    "保持输出",
  );
});
