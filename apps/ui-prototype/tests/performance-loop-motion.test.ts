import test from "node:test";
import assert from "node:assert/strict";
import type { AudioTimeline } from "../src/audio-types.ts";
import type { AudioLoopRegion } from "../src/audio-performance-types.ts";
import {
  loopMotionCommand,
  moveLoopGroup,
  moveLoopRegion,
} from "../src/components/audio/performance-loop-motion.ts";
import {
  guardLoopGroup,
  loopGroupCommand,
  loopGroupSelection,
} from "../src/components/audio/performance-loop-group.ts";
import { selectionViewRange } from "../src/components/audio/selection-view.ts";
import { AudioDraftError } from "../src/components/audio/audio-draft-error.ts";
const region = (
  id: string,
  startMs: number,
  endMs: number,
): AudioLoopRegion => ({
  id,
  name: id,
  startMs,
  endMs,
  plays: { kind: "count", count: 1 },
  enabled: true,
  locked: false,
});
const track = (): AudioTimeline => ({
  asset: {
    digest: "ab".repeat(32),
    fileName: "演出.wav",
    extension: "wav",
    durationMs: 10000,
  },
  inMs: 0,
  outMs: 10000,
  markers: [{ id: "m", name: "卡点", timeMs: 6500, sceneId: null }],
  loopRegions: [region("a", 1000, 2000), region("b", 4000, 5000)],
});
test("等待草稿处理之后重新核对整组快照，不把旧预览应用到新区段", () => {
  const t = track(),
    original = structuredClone(t.loopRegions!);
  guardLoopGroup(t, original);
  t.loopRegions![1].startMs++;
  assert.throws(
    () => guardLoopGroup(t, original),
    (e) => e instanceof AudioDraftError && e.field === "loopDestination",
  );
  t.loopRegions!.pop();
  assert.throws(() => guardLoopGroup(t, original), /变化/);
});
test("单段移动保持长度、次数及标志，停用邻居仍限制范围", () => {
  const t = track();
  t.loopRegions![1].enabled = false;
  const r = t.loopRegions![0],
    before = structuredClone(t);
  assert.deepEqual(moveLoopRegion(t, r, "move", 10000), {
    ...r,
    startMs: 3000,
    endMs: 4000,
  });
  assert.equal(moveLoopRegion(t, r, "move", -10000).startMs, 0);
  assert.deepEqual(t, before);
});
test("边界保留至少一毫秒，不能覆盖邻居；相邻半开边界允许", () => {
  const t = track(),
    r = t.loopRegions![0];
  assert.equal(moveLoopRegion(t, r, "start", 10000).startMs, 1999);
  assert.equal(moveLoopRegion(t, r, "end", -10000).endMs, 1001);
  assert.equal(moveLoopRegion(t, r, "end", 10000).endMs, 4000);
});
test("靠近才吸附，关闭或超阈值不跳到远处；整数毫秒", () => {
  const t = track(),
    r = t.loopRegions![1];
  assert.equal(moveLoopRegion(t, r, "end", 1495, 8).endMs, 6500);
  assert.equal(moveLoopRegion(t, r, "end", 1495).endMs, 6495);
  assert.equal(moveLoopRegion(t, r, "end", 1480, 8).endMs, 6480);
  assert.equal(moveLoopRegion(t, r, "start", 1.4).startMs, 4001);
});
test("锁定与非有限位移保持原值", () => {
  const t = track(),
    r = t.loopRegions![0];
  assert.strictEqual(moveLoopRegion(t, r, "move", NaN), r);
  r.locked = true;
  assert.strictEqual(moveLoopRegion(t, r, "end", 100), r);
});
test("组移动保留空隙与标志，边界统一夹住，不修改来源", () => {
  const t = track(),
    before = structuredClone(t),
    m = moveLoopGroup(t, ["b", "a"], 10000);
  assert.equal(m.destination, 6000);
  assert.equal(m.delta, 5000);
  assert.equal(m.problem, "");
  assert.deepEqual(
    m.items.map((r) => [r.startMs, r.endMs]),
    [
      [6000, 7000],
      [9000, 10000],
    ],
  );
  assert.deepEqual(t, before);
});
test("整组预览不越过未选区段，重复／失效选择拒绝，锁定项整组拒绝", () => {
  const t = track();
  assert.match(moveLoopGroup(t, ["a"], 2500).problem, /重叠/);
  assert.equal(moveLoopGroup(t, ["a"], 2000).problem, "");
  for (const ids of [[], ["a", "a"], ["a", "lost"]])
    assert.notEqual(moveLoopGroup(t, ids, 100).problem, "");
  t.loopRegions![1].locked = true;
  assert.equal(moveLoopGroup(t, ["a", "b"], 100).delta, 0);
});
test("组吸附遵循阈值且不选择重叠候选", () => {
  const t = track();
  assert.equal(moveLoopGroup(t, ["a"], 1995, 8).destination, 3000);
  assert.equal(moveLoopGroup(t, ["a"], 1980, 8).destination, 2980);
  assert.notEqual(moveLoopGroup(t, ["a"], 2995, 8).problem, "");
});
test("移动用现有组命令，边界用 Put，源变化拒绝，不覆盖次数", () => {
  const t = track(),
    original = structuredClone(t.loopRegions![0]),
    next = { ...original, startMs: 1100 };
  assert.deepEqual(loopMotionCommand(t, original, next, "move"), {
    kind: "loopRegions",
    command: {
      kind: "edit",
      ids: ["a"],
      action: { kind: "move", destinationMs: 1100 },
    },
  });
  const c = loopMotionCommand(t, original, next, "start");
  assert.ok(c.kind === "loopRegions" && c.command.kind === "put");
  assert.deepEqual(c.command.region.plays, original.plays);
  t.loopRegions![0].plays = { kind: "untilExit" };
  assert.throws(() => loopMotionCommand(t, original, next, "start"), /变化/);
});
test("组命令排序且精确解析目标，错误定位输入；锁定复制允许，改动不允许", () => {
  const t = track();
  assert.deepEqual(loopGroupCommand(t, ["b", "a"], "copy", "5.001"), {
    kind: "loopRegions",
    command: {
      kind: "edit",
      ids: ["a", "b"],
      action: { kind: "copy", destinationMs: 5001 },
    },
  });
  assert.throws(
    () => loopGroupCommand(t, ["a"], "move", "-1"),
    (e) => e instanceof AudioDraftError && e.field === "loopDestination",
  );
  assert.throws(() => loopGroupCommand(t, ["a", "a"], "copy", "1"));
  t.loopRegions![1].locked = true;
  assert.throws(() => loopGroupCommand(t, ["a", "b"], "remove"), /整组/);
  assert.equal(
    loopGroupCommand(t, ["a", "b"], "copy", "5").kind,
    "loopRegions",
  );
  assert.equal(loopGroupCommand(t, ["a", "b"], "unlock").kind, "loopRegions");
});
test("筛选不丢隐藏选择，组范围和单段范围可适配视口，不改播放状态", () => {
  const t = track(),
    regions = t.loopRegions!,
    group = loopGroupSelection(regions, ["a", "b"], [regions[0]]);
  assert.equal(group.hidden, 1);
  assert.equal(group.items.length, 2);
  assert.deepEqual(selectionViewRange(t, "a"), {
    startMs: 1000,
    endMs: 2000,
    label: "a",
  });
  assert.deepEqual(
    selectionViewRange(t, "", undefined, undefined, {
      active: true,
      ids: ["a", "b"],
    }),
    { startMs: 1000, endMs: 5000, label: "2 个循环区段" },
  );
  assert.equal(
    selectionViewRange(t, "a", undefined, undefined, { active: true, ids: [] }),
    null,
  );
});
