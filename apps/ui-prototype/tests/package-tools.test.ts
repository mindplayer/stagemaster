import test from "node:test";
import assert from "node:assert/strict";
import {
  filterPrograms,
  packageCurrent,
  selectFiltered,
  selectionKey,
} from "../src/package-tools.ts";
import type { PackageCandidate, PackageResult } from "../src/package-types.ts";
const programs: PackageCandidate[] = [
  { kind: "scene", id: "a", name: "开场" },
  { kind: "sequence", id: "b", name: "完整节目" },
];
test("筛选不改变已选项，批量选择去重且拒绝超限", () => {
  assert.deepEqual(filterPrograms(programs, " 完整 ", "all"), [programs[1]]);
  assert.deepEqual(filterPrograms(programs, "", "scene"), [programs[0]]);
  const selected = selectFiltered([programs[0]], programs)!;
  assert.equal(selected.length, 2);
  assert.equal(selectionKey(selected), selectionKey([...selected].reverse()));
  const many = Array.from({ length: 65 }, (_, i) => ({
    kind: "scene" as const,
    id: String(i),
    name: String(i),
  }));
  assert.equal(selectFiltered([], many), null);
  assert.equal(selectFiltered([], many.slice(0, 64))!.length, 64);
});
test("工程、草稿和节目范围改变会使成功及失败结果过期", () => {
  const result: PackageResult = {
    generation: 7,
    token: null,
    report: null,
    issues: [],
  };
  const key = selectionKey(programs);
  assert.equal(packageCurrent(result, "p", 7, false, key, programs), true);
  assert.equal(packageCurrent(result, "p", 8, false, key, programs), false);
  assert.equal(packageCurrent(result, "p", 7, true, key, programs), false);
  assert.equal(
    packageCurrent(result, "p", 7, false, key, programs.slice(0, 1)),
    false,
  );
  assert.equal(packageCurrent(null, "p", 7, false, key, programs), false);
});
