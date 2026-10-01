import {
  clipsInRange,
  currentClipIds,
} from "../src/components/audio/clip-selection.ts";
import test from "node:test";
import assert from "node:assert/strict";
import type { AudioTimeline, AudioLightingClip } from "../src/audio-types.ts";
import {
  clipGroupCommand,
  clipGroupSelection,
  filteredClips,
  toggleClipRange,
} from "../src/components/audio/clip-group-tools.ts";
const clips: AudioLightingClip[] = [0, 1, 2].map((i) => ({
  id: `c${i}`,
  name: `片段 ${i}`,
  sceneId: "s",
  startMs: i * 2000,
  endMs: i * 2000 + 1000,
  fadeMs: 0,
  locked: i === 2,
}));
const track: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "曲.wav",
    extension: "wav",
    durationMs: 10000,
  },
  inMs: 0,
  outMs: 10000,
  markers: [],
  lightingClips: clips,
};
test("片段组选择按时间排序、保留隐藏项并识别锁定", () => {
  const selection = clipGroupSelection(
    clips,
    ["c2", "missing", "c0"],
    [clips[0]],
  );
  assert.deepEqual(
    selection.items.map((c) => c.id),
    ["c0", "c2"],
  );
  assert.deepEqual(
    [selection.first, selection.last, selection.hidden, selection.locked],
    [0, 5000, 1, 1],
  );
  assert.deepEqual(filteredClips(clips, [], " 片段 1 "), [clips[1]]);
});
test("范围选择只跨可见片段，保留筛选外项，失效锚点退为单选", () => {
  assert.deepEqual(toggleClipRange([], clips, "c2", "c0", true), [
    "c0",
    "c1",
    "c2",
  ]);
  assert.deepEqual(
    toggleClipRange(["c1"], [clips[0], clips[2]], "c2", "c0", true),
    ["c1", "c0", "c2"],
  );
  assert.deepEqual(toggleClipRange(["c1"], [clips[2]], "c2", "c0", true), [
    "c1",
    "c2",
  ]);
  assert.deepEqual(toggleClipRange(["c1"], clips, "c1", null, false), []);
});
test("组操作毫秒精确、删除不依赖时间输入，核心负责碰撞与新身份", () => {
  const before = structuredClone(track);
  assert.deepEqual(clipGroupCommand(track, ["c2", "c0"], "copy", "6.125"), {
    kind: "editLightingClips",
    ids: ["c2", "c0"],
    action: { kind: "copy", destinationMs: 6125 },
  });
  assert.deepEqual(clipGroupCommand(track, ["c0"], "remove", "无效"), {
    kind: "editLightingClips",
    ids: ["c0"],
    action: { kind: "remove" },
  });
  for (const value of ["-1", "NaN", "1.0001", ""])
    assert.throws(() => clipGroupCommand(track, ["c0"], "move", value));
  for (const ids of [[], ["missing"], ["c0", "c0"], Array(513).fill("c0")])
    assert.throws(() => clipGroupCommand(track, ids, "remove", ""));
  assert.throws(() =>
    clipGroupCommand(
      { ...track, lightingClips: undefined },
      ["c0"],
      "remove",
      "",
    ),
  );
  assert.deepEqual(track, before);
});

test("片段启停筛选保持隐藏选择，统一启停不依赖目标时间", () => {
  const t = structuredClone(track);
  t.lightingClips![1].enabled = false;
  assert.deepEqual(
    filteredClips(t.lightingClips!, [], "", "disabled").map((c) => c.id),
    ["c1"],
  );
  assert.deepEqual(
    filteredClips(t.lightingClips!, [], "", "enabled").map((c) => c.id),
    ["c0", "c2"],
  );
  const selection = clipGroupSelection(
    t.lightingClips!,
    ["c0", "c1"],
    [t.lightingClips![1]],
  );
  assert.equal(selection.inactive, 1);
  assert.equal(selection.hidden, 1);
  assert.deepEqual(clipGroupCommand(t, ["c0", "c1"], "disable", "无效"), {
    kind: "editLightingClips",
    ids: ["c0", "c1"],
    action: { kind: "enabled", enabled: false },
  });
});

test("时间线框选双向且仅按区间交集，不因锁定停用而遗漏", () => {
  const before = structuredClone(clips);
  const source = clips.map((c) => ({ ...c, enabled: false }));
  assert.deepEqual(clipsInRange(source, 500, 4001), ["c0", "c1", "c2"]);
  assert.deepEqual(clipsInRange(source, 4001, 500), ["c0", "c1", "c2"]);
  assert.deepEqual(clipsInRange(source, 1000, 2000), []);
  assert.deepEqual(clipsInRange(source, 1000, 4000), ["c1"]);
  assert.deepEqual(clipsInRange(source, 0, 0), []);
  assert.deepEqual(clipsInRange(source, NaN, 5000), []);
  assert.deepEqual(clipsInRange(source, 0, Infinity), []);
  assert.deepEqual(clips, before);
});
test("共享选择规范化去重且过滤撤销后失效项，按时间顺序呈现", () => {
  assert.deepEqual(currentClipIds(clips, ["c2", "missing", "c0", "c2"]), [
    "c0",
    "c2",
  ]);
  assert.deepEqual(currentClipIds([], ["c2"]), []);
  const range = clipsInRange(clips, 2000, 5000);
  assert.deepEqual(currentClipIds(clips, ["c0", "c2", ...range]), [
    "c0",
    "c1",
    "c2",
  ]);
});
