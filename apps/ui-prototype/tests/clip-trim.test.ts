import test from "node:test";
import assert from "node:assert/strict";
import type { AudioLightingClip, AudioTimeline } from "../src/audio-types.ts";
import {
  clipDraft,
  collectClipDraft,
} from "../src/components/audio/audio-clip-draft.ts";
import { AudioDraftError } from "../src/components/audio/audio-draft-error.ts";
import {
  clipMotionCommand,
  trimmedEffectOffset,
} from "../src/components/audio/clip-trim-tools.ts";
import { moveLightingClip } from "../src/components/audio/clip-motion.ts";
const c: AudioLightingClip = {
  id: "a",
  name: "灯光",
  sceneId: "s",
  startMs: 2000,
  endMs: 5000,
  fadeMs: 500,
  locked: false,
  effectOffsetMs: 1000,
};
const t: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "曲.wav",
    extension: "wav",
    durationMs: 10000,
  },
  inMs: 0,
  outMs: 10000,
  markers: [],
  lightingClips: [c],
};
test("裁切精确草稿预检新源时间但命令携带原偏移，由核心计算", () => {
  const d = {
    ...clipDraft(c),
    preserveProgress: true,
    start: "3.371",
    end: "4.999",
    name: "局部",
  };
  const cmd = collectClipDraft(d, t);
  assert.equal(cmd.kind, "trimLightingClip");
  if (cmd.kind === "trimLightingClip") {
    assert.equal(cmd.clip.effectOffsetMs, 1000);
    assert.equal(trimmedEffectOffset(c, cmd.clip.startMs), 2371);
    assert.equal(cmd.clip.name, "局部");
  }
  assert.equal(
    collectClipDraft({ ...d, preserveProgress: false }, t).kind,
    "putLightingClip",
  );
  assert.equal(
    collectClipDraft({ ...d, copy: true, start: "7" }, t).kind,
    "copyLightingClip",
  );
  assert.equal(c.startMs, 2000);
});
test("负源时间、过长源范围与锁定按精确字段拒绝", () => {
  assert.throws(
    () =>
      collectClipDraft(
        { ...clipDraft(c), preserveProgress: true, start: "0.999" },
        t,
      ),
    (e: unknown) => e instanceof AudioDraftError && e.field === "clipStart",
  );
  const limit = { ...c, effectOffsetMs: 3597000 };
  const bound = { ...t, lightingClips: [limit] };
  assert.throws(
    () =>
      collectClipDraft(
        {
          ...clipDraft(limit),
          preserveProgress: true,
          start: "3",
          end: "5.001",
        },
        bound,
      ),
    (e: unknown) => e instanceof AudioDraftError && e.field === "clipEnd",
  );
  assert.throws(
    () =>
      collectClipDraft(
        { ...clipDraft(c), preserveProgress: true },
        { ...t, lightingClips: [{ ...c, locked: true }] },
      ),
    /锁定/,
  );
  const preserved = collectClipDraft(
    { ...clipDraft(c), preserveProgress: true, start: "3", end: "5" },
    t,
  );
  assert.equal(preserved.kind, "trimLightingClip");
});
test("端点拖动／吸附有源零点和一小时上限，移动仍可重新安排", () => {
  const atStart = moveLightingClip(t, c, "start", -5000, true, 100);
  assert.equal(atStart.startMs, 1000);
  assert.equal(atStart.effectOffsetMs, 1000);
  assert.equal(trimmedEffectOffset(c, atStart.startMs), 0);
  assert.equal(moveLightingClip(t, c, "move", -5000).startMs, 0);
  const nearLimit = { ...c, effectOffsetMs: 3596500 };
  assert.equal(moveLightingClip(t, nearLimit, "end", 9999).endMs, 5500);
  const short = moveLightingClip(t, c, "start", 9999);
  assert.equal(short.startMs, 4500);
  assert.equal(trimmedEffectOffset(c, short.startMs), 3500);
  assert.deepEqual(moveLightingClip(t, c, "start", NaN), c);
});
test("手势命令区分整体移动与前后裁切，不信任 UI 偏移变换", () => {
  for (const mode of ["start", "end"] as const)
    assert.deepEqual(clipMotionCommand(c, mode), {
      kind: "trimLightingClip",
      clip: c,
    });
  assert.deepEqual(clipMotionCommand(c, "move"), {
    kind: "putLightingClip",
    clip: c,
  });
});
