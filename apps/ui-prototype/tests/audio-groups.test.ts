import test from "node:test";
import assert from "node:assert/strict";
import {
  filteredAudioMarkers,
  markerSelection,
  markerGroupCommand,
} from "../src/audio-group-tools.ts";
import type { AudioTimeline } from "../src/audio-types";
const track: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "音乐.wav",
    extension: "wav",
    durationMs: 10000,
  },
  inMs: 0,
  outMs: 10000,
  markers: [
    { id: "a", name: "开场", timeMs: 1000, sceneId: "scene" },
    { id: "b", name: "拍点", timeMs: 2000, sceneId: null },
    { id: "c", name: "副歌", timeMs: 3000, sceneId: "scene" },
  ],
};
test("筛选不会悄悄清除隐藏选择，实际集合按工程时序排列", () => {
  const visible = filteredAudioMarkers(track, [], "  拍点  ");
  const selection = markerSelection(track, ["c", "b", "missing"], visible);
  assert.deepEqual(
    selection.markers.map((m) => m.id),
    ["b", "c"],
  );
  assert.equal(selection.hidden, 1);
  assert.equal(selection.first, 2000);
  assert.equal(selection.last, 3000);
  assert.equal(
    filteredAudioMarkers(
      track,
      [{ id: "scene", name: "BLUE", values: [], effects: [] }],
      "blue",
    ).length,
    2,
  );
});
test("成组操作保留显式目标，删除不依赖未应用时间输入", () => {
  assert.deepEqual(markerGroupCommand(track, ["b", "a"], "move", "4.005"), {
    kind: "editMarkers",
    ids: ["b", "a"],
    action: { kind: "move", destinationMs: 4005 },
  });
  assert.deepEqual(markerGroupCommand(track, ["a"], "remove", "不是秒数"), {
    kind: "editMarkers",
    ids: ["a"],
    action: { kind: "remove" },
  });
  assert.equal(track.markers[0].timeMs, 1000);
});
test("未知、重复、空选择与无效精度必须拒绝", () => {
  for (const ids of [[], ["a", "a"], ["missing"]])
    assert.throws(() => markerGroupCommand(track, ids, "copy", "1"));
  for (const value of ["", "-1", "1.0001", "10", "Infinity"])
    assert.throws(() => markerGroupCommand(track, ["a"], "copy", value));
});
