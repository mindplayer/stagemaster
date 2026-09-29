import test from "node:test";
import assert from "node:assert/strict";
import {
  audioMilliseconds,
  audioTime,
  snapAudioTime,
  validateMarker,
} from "../src/audio-tools.ts";
import type { AudioTimeline } from "../src/audio-types.ts";
const track: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "音乐.wav",
    extension: "wav",
    durationMs: 12000,
  },
  inMs: 2000,
  outMs: 10000,
  markers: [{ id: "a", name: "第一拍", timeMs: 1000, sceneId: null }],
};
test("精确毫秒输入拒绝隐式舍入和无效数值", () => {
  assert.equal(audioMilliseconds("1.023", "时间"), 1023);
  for (const s of ["1.0001", "-1", "NaN", "1e3", "3601", ""])
    assert.throws(() => audioMilliseconds(s, "时间"));
  assert.equal(audioTime(61023), "01:01.023");
});
test("吸附可关闭并排除自己；碰撞必须由编辑层明确拒绝", () => {
  assert.equal(snapAudioTime(980, track, true), 1000);
  assert.equal(snapAudioTime(980, track, false), 980);
  assert.equal(snapAudioTime(980, track, true, "a"), 980);
  assert.throws(() =>
    validateMarker(
      { id: "b", name: "第二拍", timeMs: 1000, sceneId: null },
      track,
    ),
  );
  assert.throws(() =>
    validateMarker(
      { id: "a", name: "第一拍", timeMs: 8000, sceneId: null },
      track,
    ),
  );
});

test("无效时间不能进入卡点草稿", () => {
  for (const timeMs of [NaN, Infinity, 1.2])
    assert.throws(() =>
      validateMarker({ id: "new", name: "卡点", timeMs, sceneId: null }, track),
    );
});

import {
  clipEnvelope,
  viewTime,
  zoomAround,
} from "../src/components/audio/waveform-data.ts";
test("waveform clipping preserves signed channels and rejects malformed or excessive envelopes", () => {
  const wave = {
    durationMs: 25,
    bucketMs: 10,
    channels: [
      [-0.1, 0.2, -0.3, 0.4, -0.5, 0.6],
      [-0.7, 0.8, -0.2, 0.3, -0.1, 0.2],
    ],
  };
  const data = clipEnvelope(wave, 10, 25);
  assert.equal(data.length, 2);
  assert.equal(data[0].length, 4);
  assert.ok(Math.abs(data[0][0] + 0.3) < 0.00001);
  assert.ok(Math.abs(data[1][0] + 0.2) < 0.00001);
  assert.throws(() => clipEnvelope(wave, 25, 25));
  assert.throws(() => clipEnvelope({ ...wave, channels: [[NaN, 1]] }, 0, 10));
  assert.throws(() => clipEnvelope({ ...wave, channels: [[-1, 2]] }, 0, 10));
  assert.throws(() => clipEnvelope({ ...wave, channels: [[]] }, 0, 10));
  assert.throws(() =>
    clipEnvelope({ ...wave, channels: [new Array(720002).fill(0)] }, 0, 10),
  );
});
test("zoom and pointer calculations keep the musical location under the pointer", () => {
  assert.equal(
    viewTime(250, 50, { start: 10000, end: 20000, width: 1000 }),
    12000,
  );
  assert.equal(zoomAround(12000, 200, 200), 2200);
  assert.equal(zoomAround(0, 200, 200), 0);
});
