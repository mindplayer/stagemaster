import test from "node:test";
import assert from "node:assert/strict";
import {
  audioDisplayTime,
  audioLoopRange,
  AudioLoopError,
  markerLoopRange,
} from "../src/audio-loop-tools.ts";
import type { AudioTimeline } from "../src/audio-types.ts";
test("loop range validates millisecond precision and focuses the offending boundary", () => {
  assert.deepEqual(audioLoopRange(" 1.234 ", "1.334", 2000), {
    startMs: 1234,
    endMs: 1334,
  });
  for (const [a, b, field] of [
    ["-1", "2", "start"],
    ["1", "2.0001", "end"],
    ["1", "1.099", "end"],
    ["1", "0", "end"],
    ["0", "60.001", "end"],
    ["0", "65", "end"],
  ]) {
    assert.throws(
      () => audioLoopRange(a, b, 64000),
      (e: unknown) => e instanceof AudioLoopError && e.field === field,
    );
  }
  assert.deepEqual(audioLoopRange("0", "60", 60000), {
    startMs: 0,
    endMs: 60000,
  });
});
test("selected lighting range ends at the next light change and ignores beat-only markers", () => {
  const track = {
    inMs: 500,
    outMs: 10000,
    markers: [
      { id: "b", timeMs: 2000, sceneId: null },
      { id: "c", timeMs: 3000, sceneId: "2" },
      { id: "a", timeMs: 1000, sceneId: "1" },
    ],
  } as AudioTimeline;
  assert.deepEqual(markerLoopRange(track, "a"), { startMs: 1000, endMs: 3000 });
  assert.deepEqual(markerLoopRange(track, "c"), { startMs: 3000, endMs: 9500 });
  assert.equal(markerLoopRange(track, "b"), null);
  assert.equal(markerLoopRange(track, "missing"), null);
});
test("waveform interpolation wraps visually without advancing a paused native position", () => {
  const p = {
    playing: true,
    positionMs: 1980,
    durationMs: 10000,
    loopRange: { startMs: 1000, endMs: 2000 },
    volumePercent: 100,
    problem: null,
  };
  assert.equal(audioDisplayTime(p, 10), 1990);
  assert.equal(audioDisplayTime(p, 20), 1000);
  assert.equal(audioDisplayTime(p, 1000), 1100);
  assert.equal(audioDisplayTime({ ...p, playing: false }, 1000), 1980);
  assert.equal(audioDisplayTime({ ...p, loopRange: null }, 1000), 2100);
  assert.equal(p.positionMs, 1980);
});
