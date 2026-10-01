import test from "node:test";
import assert from "node:assert/strict";
import { moveClipGroup } from "../src/components/audio/clip-group-motion.ts";
import type { AudioTimeline } from "../src/audio-types.ts";
const track: AudioTimeline = {
  asset: {
    digest: "a".repeat(64),
    fileName: "曲.wav",
    extension: "wav",
    durationMs: 20000,
  },
  inMs: 0,
  outMs: 20000,
  markers: [],
  lightingClips: [
    {
      id: "a",
      name: "前段",
      sceneId: "s",
      startMs: 1000,
      endMs: 2000,
      fadeMs: 500,
      locked: false,
      effectOffsetMs: 777,
    },
    {
      id: "b",
      name: "间隙中的段",
      sceneId: "s",
      startMs: 2100,
      endMs: 2500,
      fadeMs: 0,
      locked: false,
    },
    {
      id: "c",
      name: "后段",
      sceneId: "s",
      startMs: 4000,
      endMs: 5000,
      fadeMs: 250,
      locked: false,
      enabled: false,
    },
  ],
};
test("整组保留间隔、效果源起点、渐变和停用，几何预览不改原工程", () => {
  const before = structuredClone(track);
  const proposal = moveClipGroup(track, ["c", "a"], 8000.4);
  assert.deepEqual(
    [proposal.delta, proposal.destination, proposal.problem],
    [8000, 9000, ""],
  );
  assert.deepEqual(proposal.ids, ["a", "c"]);
  assert.deepEqual(
    proposal.clips,
    [track.lightingClips![0], track.lightingClips![2]].map((c) => ({
      ...c,
      startMs: c.startMs + 8000,
      endMs: c.endMs + 8000,
    })),
  );
  assert.deepEqual(track, before);
  assert.equal(moveClipGroup(track, ["a", "c"], 0).problem, ""); // An unselected clip may occupy an internal gap.
});
test("整组范围限位与精确微调，内部间隔不压缩", () => {
  assert.equal(moveClipGroup(track, ["a", "c"], -10000).destination, 0);
  assert.equal(
    moveClipGroup(track, ["a", "c"], 90000).clips.at(-1)?.endMs,
    20000,
  );
  assert.equal(moveClipGroup(track, ["a", "c"], 10).destination, 1010);
  assert.equal(moveClipGroup(track, ["a", "c"], -10).destination, 990);
});
test("未选择片段碰撞、锁定与过时选择被明确拒绝", () => {
  assert.match(
    moveClipGroup(track, ["a", "c"], 500).problem,
    /间隙中的段.*重叠/,
  );
  const locked = structuredClone(track);
  locked.lightingClips![2].locked = true;
  assert.match(moveClipGroup(locked, ["a", "c"], 8000).problem, /后段.*锁定/);
  for (const ids of [[], ["missing"], ["a", "a"], ["a", "missing"]])
    assert.ok(moveClipGroup(track, ids, 0).problem);
  for (const invalid of [NaN, Infinity, -Infinity])
    assert.ok(moveClipGroup(track, ["a"], invalid).problem);
});
test("仅吸附整组首尾，碰撞吸附目标不遮蔽更远的可行候选", () => {
  const t = structuredClone(track);
  t.markers = [{ id: "m", name: "拍", timeMs: 9000, sceneId: null, fadeMs: 0 }];
  assert.equal(moveClipGroup(t, ["a", "c"], 8010, 20).destination, 9000);
  assert.equal(
    moveClipGroup(t, ["a", "c"], 3995, 20).clips.at(-1)?.endMs,
    9000,
  );
  assert.equal(moveClipGroup(t, ["a", "c"], 8010, 0).destination, 9010);
  t.lightingClips![1].startMs = 6100;
  t.lightingClips![1].endMs = 6500;
  t.lightingClips!.sort((a, b) => a.startMs - b.startMs);
  assert.equal(
    moveClipGroup(t, ["a", "c"], 1090, 20).clips.at(-1)?.endMs,
    6100,
  );
  t.markers[0].timeMs = 2110; // Closest start candidate would overlap at the group's tail.
  assert.equal(moveClipGroup(t, ["a", "c"], 1108, 20).delta, 1100);
});
test("512 个片段的大组移动保留每个片段并检查完整范围", () => {
  const t = structuredClone(track);
  t.asset.durationMs = t.outMs = 3600000;
  t.lightingClips = Array.from({ length: 512 }, (_, i) => ({
    ...track.lightingClips![0],
    id: String(i),
    startMs: i * 5000,
    endMs: i * 5000 + 1000,
  }));
  const ids = t.lightingClips.filter((_, i) => i % 2 === 0).map((c) => c.id);
  const proposal = moveClipGroup(t, ids, 1000, 8);
  assert.equal(proposal.problem, "");
  assert.equal(proposal.clips.length, 256);
  assert.equal(moveClipGroup(t, ids, 5000).problem.includes("重叠"), true);
});
