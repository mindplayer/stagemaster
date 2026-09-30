import { test } from "node:test";
import assert from "node:assert/strict";
import type { AudioTimeline } from "../src/audio-types.ts";
import {
  lightingSegments,
  constrainBoundaryTime,
} from "../src/components/audio/lighting-segments.ts";

const track: AudioTimeline = {
  asset: {
    digest: "test",
    fileName: "test.wav",
    extension: "wav",
    durationMs: 12000,
  },
  inMs: 2000,
  outMs: 10000,
  markers: [
    { id: "beat", name: "纯标记", timeMs: 500, sceneId: null },
    { id: "a", name: "起势", timeMs: 1000, sceneId: "scene-a" },
    { id: "beat2", name: "纯标记", timeMs: 2000, sceneId: null },
    { id: "b", name: "收束", timeMs: 4000, sceneId: "scene-b" },
  ],
};
test("灯光区间使用裁切相对时间，纯节奏点不打断场景", () => {
  assert.deepEqual(lightingSegments(track), [
    { markerId: null, sceneId: null, start: 0, end: 1000 },
    { markerId: "a", sceneId: "scene-a", start: 1000, end: 4000 },
    { markerId: "b", sceneId: "scene-b", start: 4000, end: 8000 },
  ]);
});
test("没有绑定时整曲使用默认值，起点绑定不产生零长默认段", () => {
  assert.deepEqual(lightingSegments({ ...track, markers: [] }), [
    { markerId: null, sceneId: null, start: 0, end: 8000 },
  ]);
  assert.equal(
    lightingSegments({
      ...track,
      markers: [{ ...track.markers[1], timeMs: 0 }],
    })[0].markerId,
    "a",
  );
});
test("拖动中重新排序投影，不改变持久标记原数组", () => {
  const unsorted = { ...track, markers: [...track.markers].reverse() };
  assert.deepEqual(lightingSegments(unsorted), lightingSegments(track));
  assert.equal(unsorted.markers[0].id, "b");
});
test("边界不能跨越相邻场景，末端保留一毫秒范围", () => {
  assert.equal(constrainBoundaryTime(track, "a", 9999), 3999);
  assert.equal(constrainBoundaryTime(track, "b", -99), 1001);
  assert.equal(constrainBoundaryTime(track, "a", -99), 0);
  assert.equal(constrainBoundaryTime(track, "b", 9999), 7999);
});
test("吸附到纯标记也不产生同时间碰撞，并保持毫秒精度", () => {
  assert.equal(constrainBoundaryTime(track, "a", 2000), 2001);
  assert.equal(constrainBoundaryTime(track, "b", 2000), 1999);
  assert.equal(constrainBoundaryTime(track, "a", 1234.56), 1235);
});
test("密集卡点有界寻找空位，无效来源拒绝，非法坐标保持原边界", () => {
  const crowded = {
    ...track,
    markers: [
      ...track.markers,
      ...Array.from({ length: 20 }, (_, i) => ({
        id: `m${i}`,
        name: "拍",
        sceneId: null,
        timeMs: 2010 + i,
      })),
    ],
  };
  const result = constrainBoundaryTime(crowded, "a", 2015);
  assert.equal(
    crowded.markers.some((m) => m.timeMs === result && m.id !== "a"),
    false,
  );
  assert.equal(constrainBoundaryTime(track, "a", NaN), 1000);
  assert.throws(() => constrainBoundaryTime(track, "missing", 0), /已删除/);
  assert.throws(() => constrainBoundaryTime(track, "beat", 0), /未绑定/);
});
