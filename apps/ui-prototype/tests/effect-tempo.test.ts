import { test } from "node:test";
import assert from "node:assert/strict";
import {
  EffectTapTempo,
  effectPeriodMs,
  tempoPeriodMs,
  scaledEffectPeriod,
} from "../src/effect-tempo.ts";
test("按拍换算只生成既有毫秒周期，舍入明确且按拍数缩放", () => {
  assert.equal(tempoPeriodMs("120", 1), 500);
  assert.equal(tempoPeriodMs("120", 4), 2000);
  assert.equal(tempoPeriodMs("120", 0.25), 125);
  assert.equal(tempoPeriodMs(" 120.5 ", 4), 1992);
  assert.equal(tempoPeriodMs("30", 16), 32000);
  assert.equal(tempoPeriodMs("300", 0.5), 100);
  assert.equal(tempoPeriodMs("137", 1), 438);
});
test("拒绝无效拍速与未支持拍数，防止生成过快周期", () => {
  for (const bpm of [
    "",
    " ",
    "29.9",
    "300.1",
    "NaN",
    "Infinity",
    "1e2",
    "120.55",
    "-60",
    "节奏",
  ])
    assert.throws(() => tempoPeriodMs(bpm, 1), /拍数/);
  for (const beats of [0, NaN, Infinity, -1, 3, 32])
    assert.throws(() => tempoPeriodMs("120", beats), /每轮/);
  assert.throws(() => tempoPeriodMs("300", 0.25), /不足/);
});
test("秒数边界及半速倍速遵守原周期范围，不吞掉非法输入", () => {
  assert.equal(effectPeriodMs("0.100"), 100);
  assert.equal(effectPeriodMs("3600"), 3600000);
  assert.equal(scaledEffectPeriod("0.201", 0.5), 101);
  assert.equal(scaledEffectPeriod("1.8", 2), 3600);
  assert.throws(() => scaledEffectPeriod("0.101", 0.5), /周期/);
  assert.throws(() => scaledEffectPeriod("3600", 2), /周期/);
  for (const value of ["", "-2", "0.099", "3600.001", "1e2"])
    assert.throws(() => effectPeriodMs(value));
});
test("敲拍以平均间隔估计，重复点击不移动基准", () => {
  const taps = new EffectTapTempo();
  assert.deepEqual(taps.tap(1000), {
    count: 1,
    bpm: null,
    state: "collecting",
  });
  assert.deepEqual(taps.tap(1100), { count: 1, bpm: null, state: "tooFast" });
  assert.deepEqual(taps.tap(1500), { count: 2, bpm: 120, state: "estimated" });
  assert.equal(taps.tap(1990).bpm, 121.2);
  assert.equal(taps.tap(2500).bpm, 120);
  assert.equal(taps.tap(2500).state, "tooFast");
});
test("敲拍有界，长间断重新计拍，异常时间清空，不保留旧估计", () => {
  const taps = new EffectTapTempo();
  for (let i = 0; i < 10000; i++) {
    const value = taps.tap(i * 500);
    assert.equal(value.count, Math.min(i + 1, 9));
    assert.equal(value.bpm, i ? 120 : null);
  }
  assert.deepEqual(taps.tap(5_002_000), {
    count: 1,
    bpm: null,
    state: "collecting",
  });
  assert.equal(taps.tap(5_004_000).bpm, 30);
  assert.equal(taps.tap(5_004_200).count, 3);
  for (const now of [NaN, Infinity, -1]) {
    assert.deepEqual(taps.tap(now), { count: 0, bpm: null, state: "invalid" });
    assert.equal(taps.tap(0).count, 1);
  }
  taps.tap(1000);
  assert.equal(taps.tap(999).state, "invalid");
  assert.equal(taps.tap(2000).count, 1);
  taps.reset();
  assert.equal(taps.tap(3000).count, 1);
});
