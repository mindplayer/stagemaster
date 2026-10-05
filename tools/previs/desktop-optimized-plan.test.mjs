import assert from "node:assert/strict";
import { test } from "node:test";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import {
  desktopAssemblyPlan,
  desktopFileEntitlements,
} from "./desktop-assembly-plan.mjs";
import { fileAccessKey } from "./file-access-plan.mjs";
import { prepareDesktopFolders } from "./desktop-assembly-files.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const sources = [
  "data/内部优化.app",
  "data/Game/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
  "data/Node/previs",
];

test("优化组装保留已编译身份与准确原实例，不只换Plist", () => {
  const p = desktopAssemblyPlan(root, sources, "desktop-release-Own1");
  assert.equal(p.id, "desktop-release-Own1");
  assert.equal(p.bundleId, "cn.stagemaster.acceptance.desktop-release-own1");
  assert.equal(p.launchEnvironment.STAGEMASTER_ACCEPTANCE_INSTANCE, p.id);
  assert.equal(p.instance, join(root, "tmp/desktop-desktop-release-Own1"));
  assert.equal(p.archive, join(root, "data/PREVIS-007/desktop-release-Own1"));
  assert.notEqual(
    p.archive,
    join(root, "data/DESKTOP-005/desktop-release-Own1"),
  );
});

test("优化Game资格仍只有原四项和准确实例五目录", () => {
  const p = desktopAssemblyPlan(root, sources, "desktop-release-Own1");
  const original = {
    "com.apple.security.app-sandbox": true,
    "com.apple.security.get-task-allow": true,
    "com.apple.security.network.client": true,
    "com.apple.security.network.server": true,
  };
  const actual = desktopFileEntitlements(p, original);
  assert.deepEqual(
    Object.keys(actual).sort(),
    [...Object.keys(original), fileAccessKey].sort(),
  );
  assert.deepEqual(
    actual[fileAccessKey],
    Object.values(p.runtime).map((dir) => dir + "/"),
  );
});

test("优化runtime已有时拒绝，不清空／共享旧实例且不新建归档", (t) => {
  const project = mkdtempSync(join(root, "tmp/previs-optimized-owner-test-"));
  t.after(() => rmSync(project, { recursive: true, force: true }));
  const p = desktopAssemblyPlan(project, sources, "desktop-release-Own1");
  mkdirSync(p.instance, { recursive: true });
  writeFileSync(join(p.instance, "keep"), "old owner");
  assert.throws(() => prepareDesktopFolders(p), /已存在/);
  assert.equal(readFileSync(join(p.instance, "keep"), "utf8"), "old owner");
  assert.equal(existsSync(p.archive), false);
  assert.equal(existsSync(p.logs), false);
});

test("优化实例长度与字符保持原Rust隔离边界", () => {
  for (const id of [
    "desktop-release-" + "a".repeat(64),
    "desktop-release-A/escape",
    "desktop-release-",
    "desktop-release-中文",
    null,
  ])
    assert.throws(() => desktopAssemblyPlan(root, sources, id), /名称无效/);
});
