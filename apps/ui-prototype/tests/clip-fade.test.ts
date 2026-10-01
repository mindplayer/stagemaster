import test from "node:test";
import assert from "node:assert/strict";
import type { AudioLightingClip, AudioTimeline } from "../src/audio-types.ts";
import {
  clipDraft,
  collectClipDraft,
} from "../src/components/audio/audio-clip-draft.ts";
import {
  entryFadePreview,
  clipRestoreStart,
} from "../src/components/audio/clip-fade-tools.ts";
import { moveLightingClip } from "../src/components/audio/clip-motion.ts";
import { clipMotionCommand } from "../src/components/audio/clip-trim-tools.ts";
const source: AudioLightingClip = {
  id: "clip",
  name: "渐变",
  sceneId: "scene",
  startMs: 1000,
  endMs: 2000,
  fadeMs: 500,
  locked: false,
  effectOffsetMs: 333,
};
function track(clip = source): AudioTimeline {
  return {
    asset: {
      digest: "a".repeat(64),
      fileName: "曲.wav",
      extension: "wav",
      durationMs: 10000,
    },
    inMs: 0,
    outMs: 10000,
    markers: [],
    lightingClips: [clip],
  };
}
test("内部截取允许短于原渐变的范围并把旧源状态交给核心", () => {
  const draft = {
    ...clipDraft(source),
    preserveProgress: true,
    preserveEntry: true,
    start: "1.123",
    end: "1.234",
  };
  const command = collectClipDraft(draft, track());
  assert.equal(command.kind, "sliceLightingClip");
  if (command.kind !== "sliceLightingClip") throw new Error("wrong command");
  assert.equal(command.clip.fadeMs, 500);
  assert.equal(command.clip.effectOffsetMs, 333);
  assert.equal(command.clip.entryFade, undefined);
  assert.deepEqual(entryFadePreview(source, command.clip, true, true), {
    durationMs: 500,
    offsetMs: 123,
    visibleMs: 111,
  });
  assert.throws(
    () => collectClipDraft({ ...draft, preserveEntry: false }, track()),
    /渐变/,
  );
  assert.throws(
    () => collectClipDraft({ ...draft, start: "0.9" }, track()),
    /原渐变零点/,
  );
  assert.throws(
    () => collectClipDraft({ ...draft, sceneId: "different" }, track()),
    /原场景/,
  );
});
test("保留渐变的复制与普通改名传递原快照，不把反馈值写进命令", () => {
  const clip = {
    ...source,
    fadeMs: 299,
    entryFade: {
      durationMs: 500,
      offsetMs: 201,
      from: [{ fixtureId: "fixture", attribute: "dimmer", value: 34567 }],
    },
  };
  const command = collectClipDraft(
    { ...clipDraft(clip), name: "改名", start: "4", end: "5" },
    track(clip),
  );
  assert.equal(command.kind, "putLightingClip");
  if (command.kind !== "putLightingClip") throw new Error("wrong command");
  assert.deepEqual(command.clip.entryFade, clip.entryFade);
  assert.equal(command.clip.fadeMs, 299);
  assert.deepEqual(
    collectClipDraft(
      { ...clipDraft(clip), copy: true, start: "4" },
      track(clip),
    ),
    { kind: "copyLightingClip", id: "clip", startMs: 4000 },
  );
  assert.equal(entryFadePreview(clip, { ...clip, fadeMs: 300 }, false), null);
});
test("保留渐变两端裁切可进入渐变内部，同时限制两个源零点", () => {
  const clip = {
    ...source,
    entryFade: { durationMs: 700, offsetMs: 200, from: [] },
  };
  const timeline = track(clip);
  assert.equal(clipRestoreStart(clip), 800);
  const next = moveLightingClip(timeline, clip, "start", -1000);
  assert.equal(next.startMs, 800);
  assert.deepEqual(clipMotionCommand(next, "start"), {
    kind: "trimLightingClip",
    clip: next,
  });
  assert.ok(next.entryFade);
  assert.equal(next.entryFade.offsetMs, 200);
  assert.equal(moveLightingClip(timeline, clip, "end", -900).endMs, 1100);
  assert.equal(
    entryFadePreview(clip, { ...clip, endMs: 1100 }, true)?.visibleMs,
    100,
  );
  assert.equal(moveLightingClip(timeline, clip, "start", 999).startMs, 1999);
  assert.throws(
    () => entryFadePreview(clip, { ...clip, startMs: 799 }, true),
    /零点/,
  );
});
test("渐变与效果时间不同步时，较小源范围决定末端约束", () => {
  const clip = {
    ...source,
    fadeMs: 0,
    entryFade: { durationMs: 500, offsetMs: 3599000, from: [] },
  };
  assert.equal(moveLightingClip(track(clip), clip, "end", 4000).endMs, 2000);
  assert.throws(
    () => entryFadePreview(clip, { ...clip, endMs: 2001 }, false),
    /3600/,
  );
});
