import test from "node:test";
import assert from "node:assert/strict";
import {
  selectionViewRange,
  fitSelectionView,
} from "../src/components/audio/selection-view.ts";
import { usageProject } from "./scene-usage-fixture.ts";
test("适应所选按当前工程时间合并全部有效片段，空组不回退旧单选", () => {
  const track = usageProject().audio!;
  track.lightingClips!.push({
    ...track.lightingClips![0],
    id: "last",
    startMs: 90000,
    endMs: 97000,
  });
  assert.deepEqual(
    selectionViewRange(track, "clip", {
      active: true,
      ids: ["missing", "last", "last", "clip"],
    }),
    { startMs: 2000, endMs: 97000, label: "2 个灯光片段" },
  );
  assert.equal(
    selectionViewRange(track, "clip", { active: true, ids: [] }),
    null,
  );
  assert.equal(selectionViewRange(track, "missing"), null);
  assert.equal(selectionViewRange(track, "clip")?.startMs, 2000); // not inMs + start
  assert.deepEqual(selectionViewRange(track, "beat"), {
    startMs: 2000,
    endMs: 2000,
    label: "节拍",
  });
  delete track.lightingClips;
  track.markers.push({
    id: "next",
    name: "换场",
    timeMs: 5000,
    sceneId: "other",
  });
  assert.deepEqual(selectionViewRange(track, "mark"), {
    startMs: 1000,
    endMs: 5000,
    label: "第一拍",
  });
  assert.equal(selectionViewRange(track, "next")?.endMs, 98000);
});
test("视图拟合覆盖两端、限制放大、点目标保留上下文，严格拒绝无效范围", () => {
  for (const duration of [100, 30000, 3600000])
    for (const width of [320, 800, 1920]) {
      for (const range of [
        { startMs: 0, endMs: duration },
        { startMs: 0, endMs: 1 },
        { startMs: duration - 1, endMs: duration },
        { startMs: duration / 2, endMs: duration / 2 },
      ]) {
        const fit = fitSelectionView(duration, width, range)!;
        assert.ok(
          fit.start >= 0 &&
            fit.end <= duration &&
            fit.start <= range.startMs &&
            fit.end >= range.endMs,
        );
        assert.ok(
          fit.pixelsPerSecond <=
            Math.max(400, (width / duration) * 1000) + 1e-8,
        );
        assert.ok(fit.ratio >= 1 && Number.isFinite(fit.scrollPixels));
      }
    }
  const point = fitSelectionView(30000, 800, { startMs: 10000, endMs: 10000 })!;
  assert.equal(point.end - point.start, 3000);
  for (const range of [
    { startMs: NaN, endMs: 2 },
    { startMs: -1, endMs: 1 },
    { startMs: 2, endMs: 1 },
    { startMs: 0, endMs: 30001 },
  ])
    assert.equal(fitSelectionView(30000, 800, range), null);
  assert.equal(fitSelectionView(0, 800, { startMs: 0, endMs: 0 }), null);
  assert.equal(fitSelectionView(30000, 0, { startMs: 0, endMs: 1 }), null);
});
