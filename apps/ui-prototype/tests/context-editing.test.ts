import { test } from "node:test";
import assert from "node:assert/strict";
import { runAudioCommands } from "../src/audio-command-sequence.ts";
import { curvePath, moveCurveFrame } from "../src/keyframe-geometry.ts";
import { readFrames, type FrameDraft } from "../src/keyframe-tools.ts";

const frames: FrameDraft[] = [
  { position: "0", transition: "linear", values: { dimmer: "0", red: "33" } },
  { position: "50", transition: "hold", values: { dimmer: "100", red: "66" } },
];
test("曲线保持跳变及末帧循环闭合，不画斜坡替代保持", () => {
  assert.equal(curvePath(frames, "dimmer"), "M 0 100 L 50 0 H 100 V 100");
  assert.match(
    curvePath([{ ...frames[0], transition: "smooth" }, frames[1]], "dimmer"),
    /^M 0 100 C 16\.666.+ 100 33\.333.+ 0 50 0 H 100 V 100$/,
  );
});
test("拖动首帧锁定起点，数值夹紧且保留其他通道和输入草稿", () => {
  const next = moveCurveFrame(frames, 0, "dimmer", 80, 150);
  assert.equal(next[0].position, "0");
  assert.deepEqual(next[0].values, { dimmer: "100", red: "33" });
  assert.equal(frames[0].values.dimmer, "0");
  readFrames(next, ["dimmer", "red"]);
});
test("拖动不得跨越相邻帧或循环边界，精度保持 0.01%", () => {
  assert.equal(
    moveCurveFrame(frames, 1, "dimmer", -50, 50)[1].position,
    "0.01",
  );
  assert.equal(
    moveCurveFrame(frames, 1, "dimmer", 101, -3)[1].position,
    "99.99",
  );
  const three = [...frames, { ...frames[1], position: "75" }];
  assert.equal(
    moveCurveFrame(three, 1, "dimmer", 100, 50)[1].position,
    "74.99",
  );
});
test("无效输入不能通过曲线被悄悄归零", () => {
  const invalid = [{ ...frames[0], values: { dimmer: "" } }, frames[1]];
  assert.throws(() => curvePath(invalid, "dimmer"));
  assert.throws(() => moveCurveFrame(invalid, 0, "dimmer", 0, 10));
  assert.throws(() => moveCurveFrame(frames, 1, "dimmer", NaN, 10));
});
test("定位成功后才播放，保持同一动作顺序", async () => {
  const sent: string[] = [];
  assert.equal(
    await runAudioCommands(
      [{ kind: "seek", positionMs: 1600 }, { kind: "play" }],
      () => true,
      async (c) => {
        sent.push(c.kind);
        return true;
      },
    ),
    true,
  );
  assert.deepEqual(sent, ["seek", "play"]);
});
test("定位失败不继续播放", async () => {
  const sent: string[] = [];
  assert.equal(
    await runAudioCommands(
      [{ kind: "seek", positionMs: 1600 }, { kind: "play" }],
      () => true,
      async (c) => {
        sent.push(c.kind);
        return false;
      },
    ),
    false,
  );
  assert.deepEqual(sent, ["seek"]);
});
test("工程变化后拒绝排队动作以及迟到定位后的播放", async () => {
  let current = false;
  const sent: string[] = [];
  const action = [
    { kind: "seek" as const, positionMs: 1600 },
    { kind: "play" as const },
  ];
  const send = async (c: { kind: string }) => {
    sent.push(c.kind);
    current = false;
    return true;
  };
  assert.equal(await runAudioCommands(action, () => current, send), false);
  assert.deepEqual(sent, []);
  current = true;
  assert.equal(await runAudioCommands(action, () => current, send), false);
  assert.deepEqual(sent, ["seek"]);
});
test("异常回执不可被当成定位成功", async () => {
  let count = 0;
  await assert.rejects(
    runAudioCommands(
      [{ kind: "seek", positionMs: 5 }, { kind: "play" }],
      () => true,
      async () => {
        count++;
        throw new Error("读取失败");
      },
    ),
    /读取失败/,
  );
  assert.equal(count, 1);
});
