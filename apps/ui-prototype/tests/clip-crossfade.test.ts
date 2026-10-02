import test from "node:test";
import assert from "node:assert/strict";
import type { AudioLightingClip, AudioTimeline } from "../src/audio-types.ts";
import { clipDraft, collectClipDraft } from "../src/components/audio/audio-clip-draft.ts";
import { entryFadePreview, clipRestoreStart } from "../src/components/audio/clip-fade-tools.ts";
import { moveLightingClip } from "../src/components/audio/clip-motion.ts";
import { collectGroupFade } from "../src/components/audio/clip-group-fade.ts";
import { sceneUsages } from "../src/components/workbench/scene-usage.ts";
const clip: AudioLightingClip = {
  id: "clip",
  name: "动态片段",
  sceneId: "target",
  startMs: 1000,
  endMs: 2000,
  fadeMs: 300,
  locked: false,
  fadeMode: "dynamic",
  effectOffsetMs: 200,
  entryCrossfade: {
    durationMs: 500,
    offsetMs: 200,
    source: {
      sceneId: "source",
      effectOffsetMs: 100,
      elapsedMs: 600
    }
  }
};
const track: AudioTimeline = {
  asset: {
    digest: "a".repeat(64),
    fileName: "曲.wav",
    extension: "wav",
    durationMs: 5000
  },
  inMs: 0,
  outMs: 5000,
  markers: [],
  lightingClips: [clip]
};
test("动态片段草稿保留来源，裁切反馈不伪造持久源时钟", () => {
  const draft = {
    ...clipDraft(clip),
    preserveProgress: true,
    start: "1.1",
    end: "1.25"
  };
  const command = collectClipDraft(draft, track);
  assert.equal(command.kind, "trimLightingClip");
  if (command.kind !== "trimLightingClip") throw new Error("command");
  assert.deepEqual(command.clip.entryCrossfade, clip.entryCrossfade);
  assert.equal(command.clip.effectOffsetMs, 200);
  assert.deepEqual(entryFadePreview(clip, command.clip, true), {
    durationMs: 500,
    offsetMs: 300,
    visibleMs: 150
  });
  assert.deepEqual(clip, track.lightingClips![0]);
  const changed = collectClipDraft({
    ...clipDraft(clip),
    fadeMode: "snapshot"
  }, track);
  assert.equal(changed.kind, "putLightingClip");
  if (changed.kind === "putLightingClip") {
    assert.equal(changed.clip.fadeMode, "snapshot");
    assert.deepEqual(changed.clip.entryCrossfade, clip.entryCrossfade); // core validates and clears history atomically
    assert.equal(entryFadePreview(clip, changed.clip, false), null);
  }
});
test("源零点与源末端约束反馈到裁切输入和拖动", () => {
  assert.equal(clipRestoreStart(clip), 800);
  assert.equal(moveLightingClip(track, clip, "start", -10000).startMs, 800);
  assert.equal(moveLightingClip(track, clip, "end", -10000).endMs, 1001);
  assert.throws(() => collectClipDraft({
    ...clipDraft(clip),
    preserveProgress: true,
    start: "0.799"
  }, track));
  const nearEnd = {
    ...clip,
    entryCrossfade: {
      ...clip.entryCrossfade!,
      source: {
        ...clip.entryCrossfade!.source,
        elapsedMs: 3598000
      }
    }
  };
  const moved = moveLightingClip({
    ...track,
    lightingClips: [nearEnd]
  }, nearEnd, "end", 10000);
  assert.equal(moved.endMs, 2900);
  assert.throws(() => entryFadePreview(nearEnd, {
    ...nearEnd,
    endMs: 2901
  }, true), /来源范围/);
});
test("完整内部截取不允许同时改变过渡方式，复制仍只发送稳定身份", () => {
  assert.throws(() => collectClipDraft({
    ...clipDraft(clip),
    preserveProgress: true,
    preserveEntry: true,
    fadeMode: "snapshot"
  }, track), /原场景和渐变/);
  assert.deepEqual(collectClipDraft({
    ...clipDraft(clip),
    copy: true,
    start: "3"
  }, track), {
    kind: "copyLightingClip",
    id: "clip",
    startMs: 3000
  });
});
test("整组渐变方式可保持或显式更换，锁定成员整体拒绝", () => {
  assert.deepEqual(collectGroupFade({
    kind: "clipGroupFade",
    ids: [clip.id],
    fade: "0.2",
    fadeMode: "dynamic"
  }, track), {
    kind: "editLightingClips",
    ids: [clip.id],
    action: {
      kind: "fade",
      fadeMs: 200,
      fadeMode: "dynamic"
    }
  });
  assert.throws(() => collectGroupFade({
    kind: "clipGroupFade",
    ids: [clip.id],
    fade: "0.2",
    fadeMode: "snapshot"
  }, {
    ...track,
    lightingClips: [{
      ...clip,
      locked: true
    }]
  }), /锁定/);
});
test("交叉来源在场景引用目录单独列出，包括停用片段", () => {
  const usages = sceneUsages({
    sequences: [],
    audio: {
      ...track,
      lightingClips: [{
        ...clip,
        enabled: false
      }]
    }
  }, "source");
  assert.equal(usages.length, 1);
  assert.match(usages[0].detail, /交叉来源/);
  assert.match(usages[0].detail, /已停用/);
  assert.deepEqual(usages[0].target, {
    kind: "clip",
    id: clip.id
  });
});
