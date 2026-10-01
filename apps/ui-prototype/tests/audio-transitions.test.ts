import test from "node:test";
import assert from "node:assert/strict";
import type { AudioTimeline } from "../src/audio-types.ts";
import { validateMarker } from "../src/audio-tools.ts";
import {
  audioFadeLimit,
  validateAudioTransitions,
} from "../src/audio-transition-tools.ts";
import {
  collectAudioDraft,
  markerDraft,
  AudioDraftError,
} from "../src/components/audio/audio-inspector-draft.ts";
import { constrainBoundaryTime } from "../src/components/audio/lighting-segments.ts";
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
    markers: [
      { id: "a", name: "入场", timeMs: 1000, sceneId: "one", fadeMs: 2000 },
      { id: "beat", name: "拍", timeMs: 2000, sceneId: null },
      { id: "b", name: "转场", timeMs: 4000, sceneId: "two", fadeMs: 1000 },
    ],
  };
}
test("渐变上限由灯光段落确定，纯节奏点不截断；零渐变允许纯标记", () => {
  const t = track();
  assert.equal(audioFadeLimit(t, t.markers[0]), 3000);
  assert.doesNotThrow(() => validateAudioTransitions(t));
  assert.throws(
    () => validateMarker({ ...t.markers[1], fadeMs: 1 }, t),
    /须绑定场景/,
  );
  assert.throws(
    () => validateMarker({ ...t.markers[0], fadeMs: 3001 }, t),
    /最多 3.000/,
  );
});
test("边界拖动保留前段渐变和本段时长，不能默默缩短", () => {
  const t = track();
  assert.equal(constrainBoundaryTime(t, "b", 0), 3000);
  assert.equal(constrainBoundaryTime(t, "a", 3999), 1999); // 2000 is occupied by a beat
  assert.equal(constrainBoundaryTime(t, "b", 9999), 9000);
  assert.throws(
    () => validateMarker({ ...t.markers[2], timeMs: 2500 }, t),
    /入场/,
  );
});
test("精确草稿区分时间与渐变错误，取消不修改原对象，移除绑定清除渐变", () => {
  const t = track();
  const original = structuredClone(t);
  const d = markerDraft(t.markers[2]);
  assert.equal(d.kind, "marker");
  if (d.kind !== "marker") throw Error();
  assert.throws(
    () => collectAudioDraft({ ...d, time: "2.500" }, t),
    (e: unknown) => e instanceof AudioDraftError && e.field === "markerTime",
  );
  assert.throws(
    () => collectAudioDraft({ ...d, fade: "7" }, t),
    (e: unknown) => e instanceof AudioDraftError && e.field === "markerFade",
  );
  assert.deepEqual(t, original);
  const result = collectAudioDraft({ ...d, fade: "0.325" }, t);
  assert.equal(result.kind, "putMarker");
  if (result.kind !== "putMarker") throw Error();
  assert.equal(result.marker.fadeMs, 325);
  const cleared = collectAudioDraft({ ...d, sceneId: "" }, t);
  if (cleared.kind !== "putMarker") throw Error();
  assert.equal(cleared.marker.fadeMs, undefined);
});
test("裁切拒绝截断末段渐变；旧草稿保存不添加渐变字段", () => {
  const t = track();
  assert.throws(
    () => collectAudioDraft({ kind: "trim", start: "0", end: "4.5" }, t),
    /渐变/,
  );
  const m = { ...t.markers[2], fadeMs: 0 };
  const d = markerDraft(m);
  const result = collectAudioDraft(d, t);
  if (result.kind !== "putMarker") throw Error();
  assert.equal(result.marker.fadeMs, undefined);
});
