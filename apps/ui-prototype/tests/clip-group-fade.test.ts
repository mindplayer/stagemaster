import test from "node:test";
import assert from "node:assert/strict";
import {
  groupFadeDraft,
  collectGroupFade,
} from "../src/components/audio/clip-group-fade.ts";
import {
  collectAudioDraft,
  AudioDraftError,
} from "../src/components/audio/audio-inspector-draft.ts";
import type { AudioTimeline } from "../src/audio-types";
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
  lightingClips: [
    {
      id: "a",
      name: "长段",
      sceneId: "s",
      startMs: 0,
      endMs: 2000,
      fadeMs: 200,
      locked: false,
    },
    {
      id: "b",
      name: "短段",
      sceneId: "s",
      startMs: 3000,
      endMs: 3500,
      fadeMs: 0,
      locked: false,
      enabled: false,
      effectOffsetMs: 150,
    },
  ],
};
test("混合值留空、固定身份，明确 0／边界精确值生成仅渐变命令", () => {
  const ids = ["b", "a"];
  const draft = groupFadeDraft(track, ids);
  ids.pop();
  assert.deepEqual(draft.ids, ["b", "a"]);
  assert.equal(draft.fade, "");
  assert.throws(() => collectGroupFade(draft, track), AudioDraftError);
  for (const [text, ms] of [
    ["0", 0],
    ["0.500", 500],
    ["0.333", 333],
  ] as const)
    assert.deepEqual(collectAudioDraft({ ...draft, fade: text }, track), {
      kind: "editLightingClips",
      ids: ["b", "a"],
      action: { kind: "fade", fadeMs: ms },
    });
  assert.equal(groupFadeDraft(track, ["a"]).fade, "0.200");
});
test("后段更短／锁定／消失及坏输入全部拒绝且定位渐变字段", () => {
  const draft = { ...groupFadeDraft(track, ["a", "b"]), fade: "0.501" };
  assert.throws(
    () => collectGroupFade(draft, track),
    (error) =>
      error instanceof AudioDraftError &&
      error.field === "clipGroupFade" &&
      error.message.includes("短段"),
  );
  for (const fade of ["-1", "NaN", "", " ", "Infinity"])
    assert.throws(() => collectGroupFade({ ...draft, fade }, track));
  for (const ids of [[], ["a", "a"], ["missing"], Array(513).fill("a")])
    assert.throws(() => groupFadeDraft(track, ids));
  const locked = {
    ...track,
    lightingClips: track.lightingClips!.map((c) => ({
      ...c,
      locked: c.id === "b",
    })),
  };
  assert.throws(
    () => collectGroupFade({ ...draft, fade: "0" }, locked),
    /短段.*锁定/,
  );
  assert.throws(
    () =>
      collectGroupFade(
        { ...draft, fade: "0" },
        { ...track, lightingClips: [] },
      ),
    /不存在/,
  );
});
