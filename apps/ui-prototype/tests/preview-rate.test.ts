import { test } from "node:test";
import assert from "node:assert/strict";
import { previewRatePercent } from "../src/preview-rate.ts";
test("预演倍率精确输入只接受范围内的整数百分比", () => {
  for (const value of ["25", "50", "100", "125", "400", " 75 "])
    assert.equal(previewRatePercent(value), Number(value));
  for (const value of [
    "",
    " ",
    "0",
    "24",
    "401",
    "-50",
    "99.5",
    "1e2",
    "Infinity",
    "NaN",
    "1000",
    "四十",
  ])
    assert.throws(() => previewRatePercent(value), /25—400/);
});
