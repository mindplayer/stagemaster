import assert from "node:assert/strict";
import test from "node:test";
import {
  markerMotion,
  type MarkerGrip,
} from "../src/components/audio/marker-motion.ts";
import { timelinePoint } from "../src/components/audio/timeline-edge-scroll.ts";
import type { AudioTimeline } from "../src/audio-types.ts";

const track: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "验收.wav",
    extension: "wav",
    durationMs: 60000,
  },
  inMs: 0,
  outMs: 60000,
  markers: [
    { id: "a", name: "开场", timeMs: 1000, sceneId: "scene", fadeMs: 500 },
    { id: "b", name: "节奏", timeMs: 1050, sceneId: null },
    { id: "c", name: "收束", timeMs: 40000, sceneId: "scene", fadeMs: 1000 },
  ],
};
const grip: MarkerGrip = {
  marker: track.markers[0],
  original: 1000,
  anchor: 1020,
  x: 202,
  moved: false,
  boundary: false,
};
test("a marker click and sub-threshold jitter never snap or edit its time", () => {
  for (const x of [200, 202, 204]) {
    assert.deepEqual(markerMotion(grip, 1020, x, track, true, 80), {
      moved: false,
      time: 1000,
    });
  }
  assert.equal(markerMotion(grip, 1050, 205, track, true, 80).time, 1050);
  assert.equal(markerMotion(grip, 1050, 205, track, false, 80).time, 1030);
});
test("the original grab offset survives forward scrolling and reversal", () => {
  const d = { ...grip, moved: true };
  const view = { start: 0, end: 6000, width: 600 };
  const time = (start: number, x: number) =>
    markerMotion(
      d,
      timelinePoint(x, 100, { ...view, start, end: start + 6000 }),
      x,
      track,
      false,
      80,
    ).time;
  assert.equal(time(12000, 690), 17880);
  assert.equal(time(0, 202), 1000);
  assert.equal(time(54000, 800), 59980);
  assert.equal(time(0, -50), 0);
  assert.equal(track.markers[0].timeMs, 1000);
});
test("legacy boundaries retain neighboring fades and exclude occupied milliseconds", () => {
  const boundary: MarkerGrip = {
    ...grip,
    marker: track.markers[2],
    original: 40000,
    anchor: 40000,
    moved: true,
    boundary: true,
  };
  assert.equal(markerMotion(boundary, -100, 300, track, false, 80).time, 1500);
  assert.equal(
    markerMotion(boundary, 70000, 300, track, false, 80).time,
    59000,
  );
  const first = { ...grip, boundary: true, moved: true };
  assert.equal(markerMotion(first, 1050, 300, track, true, 80).time, 1051);
});
test("empty-waveform seeking uses absolute time with document bounds", () => {
  const seek = { ...grip, marker: undefined, boundary: false };
  assert.equal(markerMotion(seek, 5000, 202, track, false, 80).time, 5000);
  assert.equal(markerMotion(seek, 100000, 202, track, false, 80).time, 59999);
  assert.equal(markerMotion(seek, -1000, 202, track, false, 80).time, 0);
});
