import { test } from "node:test";
import assert from "node:assert/strict";
import {
  readPins,
  togglePin,
  persistPins,
} from "../src/components/resources/pinned-resources.ts";
test("固定偏好拒绝坏数据并去重、有界读取", () => {
  for (const raw of [
    null,
    "{",
    "null",
    JSON.stringify({ version: 2, projects: [] }),
    "x".repeat(65537),
  ])
    assert.deepEqual(readPins(raw), []);
  const projects = readPins(
    JSON.stringify({
      version: 1,
      projects: [
        null,
        { projectId: "", groups: [] },
        ...Array.from({ length: 30 }, (_, i) => ({
          projectId: `p${i}`,
          groups: [
            "g",
            "g",
            null,
            ...Array.from({ length: 12 }, (_, i) => `g${i}`),
          ],
          presets: ["p", "x".repeat(129)],
        })),
      ],
    }),
  );
  assert.equal(projects.length, 20);
  assert.equal(projects[0].groups.length, 8);
  assert.deepEqual(projects[0].presets, ["p"]);
});
test("项目和类别隔离，稳定顺序，满额不删旧项", () => {
  const a = togglePin([], "a", "groups", "g", ["g"]);
  const b = togglePin(a, "b", "groups", "b", ["b"]);
  const c = togglePin(b, "a", "presets", "p", ["p"]);
  assert.deepEqual(c[0], { projectId: "a", groups: ["g"], presets: ["p"] });
  assert.deepEqual(a[0], { projectId: "a", groups: ["g"], presets: [] });
  assert.deepEqual(togglePin(c, "a", "groups", "g", ["g"])[0].groups, []);
  const names = Array.from({ length: 9 }, (_, i) => `g${i}`);
  let full = c;
  for (const id of names.slice(0, 8))
    full = togglePin(full, "a", "groups", id, names);
  const before = JSON.stringify(full);
  assert.throws(
    () => togglePin(full, "a", "groups", names[8], names),
    /最多固定 8/,
  );
  assert.equal(JSON.stringify(full), before);
  const pruned = togglePin(full, "a", "groups", names[8], [names[8]]);
  assert.deepEqual(pruned[0].groups, [names[8]]);
  assert.deepEqual(pruned[0].presets, ["p"]);
  assert.throws(
    () => togglePin(full, "a", "groups", "missing", names),
    /不可用/,
  );
});
test("最多保留最近修改的二十个工程，写失败保留当前会话数据", () => {
  let projects = readPins(
    JSON.stringify({
      version: 1,
      projects: Array.from({ length: 20 }, (_, i) => ({
        projectId: `p${i}`,
        groups: ["g"],
      })),
    }),
  );
  projects = togglePin(projects, "new", "presets", "p", ["p"]);
  assert.equal(projects.length, 20);
  assert.equal(projects[0].projectId, "new");
  assert.equal(projects.at(-1)?.projectId, "p18");
  const failure = persistPins(projects, () => {
    throw new Error("QuotaExceededError");
  });
  assert.equal(failure.projects, projects);
  assert.match(failure.problem, /本次使用/);
  let saved = "";
  assert.equal(
    persistPins(projects, (raw) => {
      saved = raw;
    }).problem,
    "",
  );
  assert.deepEqual(readPins(saved), projects);
});
