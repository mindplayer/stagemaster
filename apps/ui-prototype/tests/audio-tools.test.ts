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
