import assert from "node:assert/strict";
import test from "node:test";
import {
  edgeScrollPixels,
  sameTimelineScale,
  timelinePoint,
} from "../src/components/audio/timeline-edge-scroll.ts";
const bounds = { left: 100, right: 700, top: 200, bottom: 260 };
const view = { start: 10000, end: 16000, width: 600 };
const pointer = { clientX: 700, clientY: 230, moved: true };
test("edge navigation needs a drag, stays in its lane, and uses a gradual speed", () => {
  const dx = (x: number, y = 230, moved = true) =>
    edgeScrollPixels(
      { clientX: x, clientY: y, moved },
      bounds,
      view,
      60000,
      20,
    );
  assert.equal(dx(700, 230, false), 0);
  assert.equal(dx(400), 0);
  assert.equal(dx(700, 285), 0);
  assert.equal(dx(100, 175), 0);
  assert.ok(dx(680) > 0 && dx(680) < dx(700));
  assert.equal(dx(800), dx(700));
  assert.equal(dx(100), -dx(700));
});
test("scrolling is bounded by content, elapsed time and actual speed", () => {
  assert.equal(
    edgeScrollPixels(
      pointer,
      bounds,
      { ...view, start: 54000, end: 60000 },
      60000,
      20,
    ),
    0,
  );
  assert.equal(
    edgeScrollPixels(
      { ...pointer, clientX: 100 },
      bounds,
      { ...view, start: 0, end: 6000 },
      60000,
      20,
    ),
    0,
  );
  assert.equal(
    edgeScrollPixels(
      pointer,
      bounds,
      { ...view, start: 53999, end: 59999 },
      60000,
      20,
    ),
    0.1,
  );
  assert.equal(
    edgeScrollPixels(
      { ...pointer, clientX: 400 },
      bounds,
      { ...view, start: 0, end: 60000.1 },
      60000,
      20,
    ),
    0,
  );
  assert.equal(
    edgeScrollPixels(pointer, bounds, view, 60000, 10000),
    edgeScrollPixels(pointer, bounds, view, 60000, 50),
  );
  assert.equal(edgeScrollPixels(pointer, bounds, view, 60000, -2), 0);
  const run = (hz: number) =>
    Array.from({ length: hz }, () =>
      edgeScrollPixels(pointer, bounds, view, 60000, 1000 / hz),
    ).reduce((a, b) => a + b, 0);
  assert.ok(Math.abs(run(60) - run(120)) < 1e-6);
});
test("content grip accumulates scrolling and reverses without drift; zoom and resize invalidate it", () => {
  const anchor = timelinePoint(350, bounds.left, view);
  const moved = { ...view, start: 14000, end: 20000 };
  assert.equal(timelinePoint(350, bounds.left, moved) - anchor, 4000);
  assert.equal(timelinePoint(350, bounds.left, view) - anchor, 0);
  assert.equal(timelinePoint(900, bounds.left, moved), 20000);
  assert.equal(timelinePoint(0, bounds.left, moved), 14000);
  assert.equal(sameTimelineScale(view, moved), true);
  assert.equal(sameTimelineScale(view, { ...moved, end: 20000.0000001 }), true);
  assert.equal(sameTimelineScale(view, { ...moved, end: 21000 }), false);
  assert.equal(sameTimelineScale(view, { ...view, width: 500 }), false);
});
