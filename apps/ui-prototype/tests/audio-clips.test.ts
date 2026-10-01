import {
  clipSplitLimits,
  splitClipCommand,
} from "../src/components/audio/clip-split-tools.ts";
import test from "node:test";
import assert from "node:assert/strict";
import type { AudioTimeline, AudioLightingClip } from "../src/audio-types.ts";
import {
  collectAudioDraft,
  AudioDraftError,
} from "../src/components/audio/audio-inspector-draft.ts";
import {
  clipDraft,
  nextClipGap,
} from "../src/components/audio/audio-clip-draft.ts";
import { moveLightingClip } from "../src/components/audio/clip-motion.ts";
import { markerLoopRange } from "../src/audio-loop-tools.ts";
import { validateMarker } from "../src/audio-tools.ts";
const clip: AudioLightingClip = {
  id: "clip",
  name: "起势",
  sceneId: "scene",
  startMs: 1000,
  endMs: 3000,
  fadeMs: 500,
  locked: false,
};
function track(): AudioTimeline {
  return {
    asset: {
      digest: "ab".repeat(32),
      fileName: "曲.wav",
      extension: "wav",
      durationMs: 10000,
    },
    inMs: 0,
    outMs: 10000,
    markers: [{ id: "beat", name: "拍", sceneId: null, timeMs: 1750 }],
    lightingClips: [
      { ...clip },
      { ...clip, id: "next", startMs: 5000, endMs: 7000 },
    ],
  };
}
test("片段草稿按字段拒绝重叠、越界和超长渐变，不改相邻内容", () => {
  const t = track(),
    before = structuredClone(t),
    draft = clipDraft(clip);
  for (const [patch, field] of [
    [{ start: "5" }, "clipEnd"],
    [{ end: "6" }, "clipEnd"],
    [{ fade: "3" }, "clipFade"],
    [{ start: "-1" }, "clipStart"],
    [{ name: " " }, "clipName"],
  ] as const)
    assert.throws(
      () => collectAudioDraft({ ...draft, ...patch }, t),
      (e: unknown) => e instanceof AudioDraftError && e.field === field,
    );
  const command = collectAudioDraft({ ...draft, start: "0", end: "2" }, t);
  assert.equal(command.kind, "putLightingClip");
  if (command.kind === "putLightingClip")
    assert.deepEqual([command.clip.startMs, command.clip.endMs], [0, 2000]);
  assert.deepEqual(t, before);
});
test("复制保留时长，锁定来源可复制而修改被拒绝，副本身份交给核心", () => {
  const t = track();
  t.lightingClips![0].locked = true;
  const draft = clipDraft(t.lightingClips![0]);
  assert.throws(() => collectAudioDraft({ ...draft, name: "改名" }, t), /锁定/);
  assert.deepEqual(collectAudioDraft({ ...draft, copy: true, start: "8" }, t), {
    kind: "copyLightingClip",
    id: "clip",
    startMs: 8000,
  });
  assert.throws(
    () => collectAudioDraft({ ...draft, copy: true, start: "9" }, t),
    (e: unknown) => e instanceof AudioDraftError && e.field === "clipStart",
  );
  assert.throws(
    () => collectAudioDraft({ ...draft, copy: true, start: "2" }, t),
    /重叠/,
  );
});
test("空隙分配、音乐裁切和局部试听取真实片段边界", () => {
  const t = track();
  assert.deepEqual(nextClipGap(t, 1500), { startMs: 3000, endMs: 5000 });
  assert.equal(nextClipGap(t, 10000), null);
  assert.deepEqual(markerLoopRange(t, "clip"), { startMs: 1000, endMs: 3000 });
  assert.equal(markerLoopRange(t, "beat"), null);
  assert.throws(
    () => collectAudioDraft({ kind: "trim", start: "0", end: "6" }, t),
    /片段超出/,
  );
  assert.throws(
    () => validateMarker({ ...t.markers[0], sceneId: "scene" }, t),
    /只能作节奏/,
  );
});
test("拖动保持长度、端点约束和原始对象，吸附同时支持前后沿", () => {
  const t = track(),
    before = structuredClone(t);
  let c = moveLightingClip(t, clip, "move", 5000);
  assert.deepEqual([c.startMs, c.endMs], [3000, 5000]);
  c = moveLightingClip(t, clip, "start", 9999);
  assert.equal(c.startMs, 2500);
  c = moveLightingClip(t, clip, "end", -9999);
  assert.equal(c.endMs, 1500);
  c = moveLightingClip(t, clip, "move", 740, true, 20);
  assert.deepEqual([c.startMs, c.endMs], [1750, 3750]);
  assert.deepEqual(
    moveLightingClip(t, { ...clip, locked: true }, "move", 500),
    { ...clip, locked: true },
  );
  assert.deepEqual(t, before);
});

test("编辑停用片段的名称、时间与渐变不会隐式恢复", () => {
  const t = track();
  t.lightingClips![0].enabled = false;
  const command = collectAudioDraft(
    { ...clipDraft(t.lightingClips![0]), name: "保留停用" },
    t,
  );
  assert.equal(command.kind, "putLightingClip");
  if (command.kind === "putLightingClip")
    assert.equal(command.clip.enabled, false);
});

test("分割位置精确到毫秒，渐变内允许而片段边界拒绝", () => {
  assert.deepEqual(clipSplitLimits(clip), { min: 1001, max: 2999 });
  assert.deepEqual(splitClipCommand(clip, "1.733"), {
    kind: "splitLightingClip",
    id: clip.id,
    timeMs: 1733,
  });
  for (const value of ["1", "3", "-1", "1.0001", ""])
    assert.throws(() => splitClipCommand(clip, value));
  assert.equal(splitClipCommand(clip, "1.233").kind, "splitLightingClip");
  assert.throws(() => splitClipCommand({ ...clip, locked: true }, "2"), /锁定/);
  assert.ok(
    clipSplitLimits({ ...clip, fadeMs: 2000 }).min <
      clipSplitLimits({ ...clip, fadeMs: 2000 }).max,
  );
});
test("改名与普通移动保留效果源偏移，过长源范围错误定位到结束", () => {
  const t = track();
  t.lightingClips![0].effectOffsetMs = 333;
  const c = t.lightingClips![0];
  const command = collectAudioDraft({ ...clipDraft(c), name: "保持源时间" }, t);
  assert.equal(command.kind, "putLightingClip");
  if (command.kind === "putLightingClip")
    assert.equal(command.clip.effectOffsetMs, 333);
  assert.equal(moveLightingClip(t, c, "move", 100).effectOffsetMs, 333);
  t.lightingClips![0].effectOffsetMs = 3599999;
  assert.throws(
    () => collectAudioDraft(clipDraft(c), t),
    (e: unknown) => e instanceof AudioDraftError && e.field === "clipEnd",
  );
});
