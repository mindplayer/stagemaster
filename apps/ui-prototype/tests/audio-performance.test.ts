import test from "node:test";
import assert from "node:assert/strict";
import type { AudioPosition, AudioTimeline } from "../src/audio-types.ts";
import {
  audioMediaKey,
  audioLoopExitCommand,
  audioLoopRuntime,
} from "../src/audio-performance-tools.ts";
import { audioDisplayTime } from "../src/audio-loop-tools.ts";

function track(): AudioTimeline {
  return {
    asset: {
      digest: "a".repeat(64),
      extension: "wav",
      fileName: "演出.wav",
      durationMs: 5000,
    },
    inMs: 0,
    outMs: 5000,
    markers: [],
    loopRegions: [
      {
        id: "off",
        name: "停用",
        startMs: 0,
        endMs: 100,
        plays: { kind: "count", count: 2 },
        enabled: false,
        locked: false,
      },
      {
        id: "one",
        name: "等待演员",
        startMs: 1000,
        endMs: 2000,
        plays: { kind: "untilExit" },
        enabled: true,
        locked: false,
      },
    ],
  };
}
function position(): AudioPosition {
  return {
    playing: true,
    volumePercent: 100,
    positionMs: 1990,
    durationMs: 5000,
    problem: null,
    performance: {
      instance: "9007199254741001",
      region: 0,
      pass: "18446744073709551615",
      exitRequested: false,
      pendingExit: null,
      controlProblem: null,
      ended: false,
      snapshotPending: false,
      boundaryMs: 2000,
      cachedBytes: 1000,
    },
  };
}
test("演出音源身份忽略命名、锁定及停用区段，启用语义和工程切换需要重载", () => {
  const a = track(),
    b = track(),
    key = audioMediaKey(a, "工程1");
  b.loopRegions![1].name = "候场";
  b.loopRegions![1].locked = true;
  b.loopRegions![0].endMs = 200;
  assert.equal(audioMediaKey(b, "工程1"), key);
  assert.notEqual(audioMediaKey(a, "工程2"), key);
  for (const alter of [
    (t: AudioTimeline) => {
      t.loopRegions![1].id = "new";
    },
    (t: AudioTimeline) => {
      t.loopRegions![1].startMs = 1100;
    },
    (t: AudioTimeline) => {
      t.loopRegions![1].endMs = 2500;
    },
    (t: AudioTimeline) => {
      t.loopRegions![1].enabled = false;
    },
    (t: AudioTimeline) => {
      t.loopRegions![1].plays = { kind: "count", count: 8 };
    },
    (t: AudioTimeline) => {
      t.inMs = 10;
    },
  ]) {
    const changed = track();
    alter(changed);
    assert.notEqual(audioMediaKey(changed, "工程1"), key);
  }
  assert.equal(
    audioMediaKey({ ...a, loopRegions: undefined }),
    audioMediaKey({ ...a, loopRegions: [] }),
  );
});
test("边界退出绑定原生实例、启用区段和完整 u64 遍数；待确认意图支持取消", () => {
  const p = position(),
    t = track();
  assert.equal(audioLoopRuntime(t, p)?.region.id, "one");
  assert.deepEqual(audioLoopExitCommand(t, p), {
    kind: "exitLoop",
    instance: "9007199254741001",
    regionId: "one",
    pass: "18446744073709551615",
    requested: true,
  });
  p.performance!.pendingExit = {
    region: 0,
    pass: p.performance!.pass!,
    requested: true,
  };
  assert.equal(audioLoopExitCommand(t, p)?.kind, "exitLoop");
  assert.deepEqual(audioLoopExitCommand(t, p), {
    kind: "exitLoop",
    instance: "9007199254741001",
    regionId: "one",
    pass: "18446744073709551615",
    requested: false,
  });
  p.performance!.pendingExit.pass = "1";
  assert.equal(audioLoopRuntime(t, p)?.exiting, false);
  p.performance!.exitRequested = true;
  p.performance!.pendingExit = {
    region: 0,
    pass: p.performance!.pass!,
    requested: false,
  };
  assert.equal(audioLoopRuntime(t, p)?.exiting, false);
});
test("停止、结束、快照冲突、故障或不对应的范围不能发送退出", () => {
  for (const alter of [
    (p: AudioPosition) => {
      p.performance!.instance = null;
    },
    (p: AudioPosition) => {
      p.performance!.snapshotPending = true;
    },
    (p: AudioPosition) => {
      p.performance!.ended = true;
    },
    (p: AudioPosition) => {
      p.performance!.region = 8;
    },
    (p: AudioPosition) => {
      p.performance!.pass = null;
    },
    (p: AudioPosition) => {
      p.positionMs = 2000;
    },
    (p: AudioPosition) => {
      p.problem = "音频欠载";
    },
  ]) {
    const p = position();
    alter(p);
    assert.equal(audioLoopExitCommand(track(), p), null);
  }
});
test("界面插值停在原生边界，不推算圈数或提前跳到下一个区段", () => {
  const p = position();
  for (const delta of [20, 120, 99999])
    assert.equal(audioDisplayTime(p, delta), 2000);
  assert.equal(audioDisplayTime(p, -10), 1990);
  p.performance!.snapshotPending = true;
  assert.equal(audioDisplayTime(p, 120), 1990);
  p.performance!.snapshotPending = false;
  p.playing = false;
  assert.equal(audioDisplayTime(p, 120), 1990);
  p.playing = true;
  p.positionMs = 200;
  p.performance!.region = null;
  p.performance!.boundaryMs = 1000;
  assert.equal(audioDisplayTime(p, 2000), 320);
});
