import test from "node:test";
import assert from "node:assert/strict";
import { isCheckCurrent, filterIssues } from "../src/check-tools.ts";
import type { ProjectCheck, CheckIssue } from "../src/check-types.ts";

const check: ProjectCheck = {
  generation: 7,
  audioResource: null,
  deviceRelease: "unavailable",
  report: {
    projectId: "工程 A",
    revisionId: "保存版本",
    desktopReady: true,
    issues: [],
    programs: [],
    limits: {
      steps: 1024,
      attributes: 512,
      targetValues: 262144,
      effectChannels: 16384,
      keyframes: 131072,
    },
  },
};
test("freshness rejects unsaved drafts, edits, save/reopen generations and another project", () => {
  assert.ok(isCheckCurrent(check, "工程 A", 7, false));
  assert.ok(!isCheckCurrent(check, "工程 A", 7, true));
  assert.ok(!isCheckCurrent(check, "工程 A", 8, false));
  assert.ok(!isCheckCurrent(check, "工程 B", 7, false));
  assert.ok(!isCheckCurrent(null, "工程 A", 7, false));
});
test("issue filters compose without changing source order or diagnostic identity", () => {
  const issues: CheckIssue[] = [
    {
      code: "patch.missing",
      severity: "error",
      message: "PAR 1 尚未配适",
      location: { kind: "fixture", id: "1" },
    },
    {
      code: "stage.unplaced",
      severity: "warning",
      message: "PAR 1 尚未布置",
      location: { kind: "placement", id: "1" },
    },
    {
      code: "patch.missing",
      severity: "error",
      message: "PAR 2 尚未配适",
      location: { kind: "fixture", id: "2" },
    },
  ];
  assert.deepEqual(filterIssues(issues, " par  配适 ", "error"), [
    issues[0],
    issues[2],
  ]);
  assert.deepEqual(filterIssues(issues, "1", "warning"), [issues[1]]);
  assert.deepEqual(filterIssues(issues, "无匹配", "all"), []);
  assert.equal(issues.length, 3);
});
