import test from "node:test";
import assert from "node:assert/strict";
import type { AudioTimeline } from "../src/audio-types.ts";
import type { AudioLoopRegion } from "../src/audio-performance-types.ts";
import { collectPerformanceLoop, newPerformanceLoop, performanceLoopDraft } from "../src/components/audio/performance-loop-draft.ts";
import { collectAudioDraft } from "../src/components/audio/audio-inspector-draft.ts";
import { AudioDraftError } from "../src/components/audio/audio-draft-error.ts";
import { audioMediaKey } from "../src/audio-performance-tools.ts";

const region = (id: string, startMs: number, endMs: number): AudioLoopRegion => ({
  id, name: id, startMs, endMs, plays: { kind: "count", count: 2 }, enabled: true, locked: false,
});
const track = (): AudioTimeline => ({
  asset: { digest: "ab".repeat(32), fileName: "演出.wav", extension: "wav", durationMs: 10000 },
  inMs: 0, outMs: 10000, markers: [], loopRegions: [region("a", 1000, 2000), region("b", 4000, 5000)],
});
const rejects = (work: () => unknown, field: string) => assert.throws(work,
  (e: unknown) => e instanceof AudioDraftError && e.field === field);

test("旧工程新建使用已有命令，草稿和验证不修改工程", () => {
  const t = track(); delete t.loopRegions;
  const before = structuredClone(t);
  const d = newPerformanceLoop(t, 1500);
  assert.deepEqual(collectPerformanceLoop(d, t), { kind: "loopRegions", command: {
    kind: "add", name: "循环区段 1", startMs: 1500, endMs: 2500, plays: { kind: "count", count: 2 },
  } });
  assert.deepEqual(t, before);
});
test("新建默认避开启用及停用区段并受下一端点限制", () => {
  const t = track(); t.loopRegions![0].enabled = false;
  assert.equal(newPerformanceLoop(t, 1500).start, "2.000");
  assert.equal(newPerformanceLoop(t, 3500).end, "4.000");
  rejects(() => newPerformanceLoop(t, 10000), "loopStart");
});
test("半开范围允许相邻，停用范围仍拒绝重叠", () => {
  const t = track(); t.loopRegions![1].enabled = false;
  const d = newPerformanceLoop(t, 2000); d.end = "4.000";
  assert.equal(collectPerformanceLoop(d, t).kind, "loopRegions");
  d.end = "4.001"; rejects(() => collectPerformanceLoop(d, t), "loopEnd");
  d.start = "1.999"; d.end = "3.000"; rejects(() => collectPerformanceLoop(d, t), "loopStart");
});
test("总次数包含一次，持续模式独立，不接受零或隐式数字格式", () => {
  const t = track(), d = performanceLoopDraft(t.loopRegions![0]);
  for (const count of ["1", "4294967295"]) {
    d.count = count;
    const c = collectPerformanceLoop(d, t);
    assert.ok(c.kind === "loopRegions" && c.command.kind === "put");
    assert.deepEqual(c.command.region.plays, { kind: "count", count: Number(count) });
  }
  for (const count of ["0", "-1", "1.5", "1e2", "4294967296", ""]) {
    d.count = count; rejects(() => collectPerformanceLoop(d, t), "loopCount");
  }
  d.mode = "untilExit";
  const c = collectPerformanceLoop(d, t);
  assert.ok(c.kind === "loopRegions" && c.command.kind === "put");
  assert.deepEqual(c.command.region.plays, { kind: "untilExit" });
});
test("输入失败定位字段并保留原草稿／工程", () => {
  const t = track(), d = performanceLoopDraft(t.loopRegions![0]);
  d.start = "-0.001"; const before = structuredClone(d);
  rejects(() => collectPerformanceLoop(d, t), "loopStart"); assert.deepEqual(d, before);
  d.start = "1.000"; d.end = "1.000"; rejects(() => collectPerformanceLoop(d, t), "loopEnd");
  d.end = "10.001"; rejects(() => collectPerformanceLoop(d, t), "loopEnd");
  d.name = " "; rejects(() => collectPerformanceLoop(d, t), "loopName");
  assert.equal(t.loopRegions![0].endMs, 2000);
});
test("锁定、删除和同时更新的旧目标拒绝，不能覆盖别人修改", () => {
  for (const change of [
    (t: AudioTimeline) => { t.loopRegions![0].locked = true; },
    (t: AudioTimeline) => { t.loopRegions!.shift(); },
    (t: AudioTimeline) => { t.loopRegions![0].name = "新名称"; },
    (t: AudioTimeline) => { t.loopRegions![0].enabled = false; },
  ]) {
    const t = track(), d = performanceLoopDraft(t.loopRegions![0]); change(t);
    const before = structuredClone(t);
    rejects(() => collectPerformanceLoop(d, t), "loopName"); assert.deepEqual(t, before);
  }
});
test("达到 128 段拒绝新建但允许编辑已有对象", () => {
  const t = track(); t.loopRegions = Array.from({ length: 128 }, (_, i) => region(String(i), i * 50, i * 50 + 20));
  rejects(() => collectPerformanceLoop(newPerformanceLoop(t, 7000), t), "loopName");
  const d = performanceLoopDraft(t.loopRegions[0]); d.name = "更名";
  assert.equal(collectPerformanceLoop(d, t).kind, "loopRegions");
});
test("重命名保留启停／锁定，媒体身份不变；次数变化使旧计划失效", () => {
  const t = track(); t.loopRegions![0].enabled = false;
  const d = performanceLoopDraft(t.loopRegions![0]); d.name = "改名";
  const c = collectPerformanceLoop(d, t);
  assert.ok(c.kind === "loopRegions" && c.command.kind === "put");
  assert.equal(c.command.region.enabled, false); assert.equal(c.command.region.locked, false);
  const after = structuredClone(t); after.loopRegions![0] = c.command.region;
  assert.equal(audioMediaKey(t), audioMediaKey(after));
  after.loopRegions![1].plays = { kind: "count", count: 3 };
  assert.notEqual(audioMediaKey(t), audioMediaKey(after));
});
test("裁切不能静默删掉越界循环，旧无区段工程仍可裁切", () => {
  const t = track();
  rejects(() => collectAudioDraft({ kind: "trim", start: "0", end: "4" }, t), "trimEnd");
  delete t.loopRegions;
  assert.deepEqual(collectAudioDraft({ kind: "trim", start: "0", end: "4" }, t), { kind: "trim", inMs: 0, outMs: 4000 });
});
