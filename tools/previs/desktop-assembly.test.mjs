import assert from "node:assert/strict";
import { test } from "node:test";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  desktopAssemblyPlan,
  desktopFileEntitlements,
} from "./desktop-assembly-plan.mjs";
import {
  executableName,
  plainAncestors,
  prepareDesktopFolders,
  unchangedDesktopResources,
  unchangedSignalling,
  changedGameFiles,
} from "./desktop-assembly-files.mjs";
import { fileAccessKey } from "./file-access-plan.mjs";
import { fileInventory } from "./signalling-package-files.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const sources = [
  "tmp/桌面.app",
  "data/Game/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
  "data/Node/previs",
];
const plan = (project = root) =>
  desktopAssemblyPlan(project, sources, "previs-desktop-A1");
const base = () => ({
  "com.apple.security.app-sandbox": true,
  "com.apple.security.get-task-allow": true,
  "com.apple.security.network.client": true,
  "com.apple.security.network.server": true,
});
function fixture(t) {
  const directory = mkdtempSync(join(root, "tmp/previs-desktop-test-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  return directory;
}
function write(file, value = "source") {
  mkdirSync(join(file, ".."), { recursive: true });
  writeFileSync(file, value);
}

test("组装限定 Mac ARM64 和三个来源，无隐式组件或构建", () => {
  for (const [os, arch] of [
    ["linux", "arm64"],
    ["darwin", "x64"],
  ])
    assert.throws(
      () => desktopAssemblyPlan(root, sources, "previs-desktop-A1", os, arch),
      /仅支持/,
    );
  for (const values of [
    [],
    sources.slice(1),
    [...sources, "extra"],
    [null, ...sources.slice(1)],
  ])
    assert.throws(
      () => desktopAssemblyPlan(root, values, "previs-desktop-A1"),
      /三个来源/,
    );
  for (const id of ["", "../x", "previs-desktop-a/escape"])
    assert.throws(() => desktopAssemblyPlan(root, sources, id), /名称无效/);
  assert.equal("install" in plan(), false);
});
test("项目根／包外／错误 Game 入口拒绝", () => {
  for (let index = 0; index < 3; index++)
    for (const value of [".", "/outside", "../escape"]) {
      const values = [...sources];
      values[index] = value;
      assert.throws(
        () => desktopAssemblyPlan(root, values, "previs-desktop-A1"),
        /项目内/,
      );
    }
  for (const [index, value] of [
    [0, "tmp/folder"],
    [1, "data/Other.app/Contents/MacOS/StageMasterPreview"],
  ]) {
    const values = [...sources];
    values[index] = value;
    assert.throws(
      () => desktopAssemblyPlan(root, values, "previs-desktop-A1"),
      /主程序/,
    );
  }
});
test("目录与宿主验收实例一致，副本身份／组合最低系统独立", () => {
  const p = plan();
  assert.equal(
    p.runtime.user,
    join(root, "tmp/desktop-previs-desktop-A1/previs/user"),
  );
  assert.equal(
    p.runtime.temporary,
    join(root, "tmp/desktop-previs-desktop-A1/tmp/previs"),
  );
  assert.equal(
    p.runtime.logs,
    join(root, "tmp/desktop-previs-desktop-A1/logs/previs"),
  );
  assert.equal(p.launchEnvironment.STAGEMASTER_ACCEPTANCE_INSTANCE, p.id);
  assert.equal(p.minimumMacOS, "14.0");
  assert.notEqual(p.bundleId, "cn.stagemaster.desktop");
  assert.equal("CFFIXED_USER_HOME" in p.launchEnvironment, false);
  assert.equal("HOME" in p.launchEnvironment, false);
});
test("只保留四项 Development 权利与准确五个目录", () => {
  const p = plan(),
    rights = desktopFileEntitlements(p, base());
  assert.equal(Object.keys(rights).length, 5);
  assert.deepEqual(
    rights[fileAccessKey],
    Object.values(p.runtime).map((dir) => `${dir}/`),
  );
  for (const key of Object.keys(base())) {
    const missing = base();
    delete missing[key];
    assert.throws(() => desktopFileEntitlements(p, missing), /四项/);
    assert.throws(
      () => desktopFileEntitlements(p, { ...base(), [key]: false }),
      /四项/,
    );
  }
  for (const key of ["com.apple.security.inherit", fileAccessKey])
    assert.throws(
      () => desktopFileEntitlements(p, { ...base(), [key]: true }),
      /四项/,
    );
});
for (const key of ["user", "cache", "temporary", "logs", "report"])
  test(`文件资格 ${key} 不得变成项目根／其他实例`, () => {
    for (const value of [
      root,
      plan().instance,
      join(root, "tmp/desktop-other/previs/user"),
    ]) {
      const p = plan();
      p.runtime[key] = value;
      assert.throws(() => desktopFileEntitlements(p, base()), /准确桌面实例/);
    }
  });
test("写入前预检所有父路径，后方链接不引发前方目录创建", (t) => {
  const project = fixture(t),
    p = plan(project),
    other = join(project, "other");
  mkdirSync(join(project, "logs"));
  mkdirSync(other);
  symlinkSync(other, join(project, "logs/PREVIS-007"));
  assert.throws(() => prepareDesktopFolders(p), /链接/);
  assert.equal(existsSync(p.archive), false);
  assert.equal(existsSync(p.temporary), false);
  assert.equal(existsSync(p.instance), false);
});
test("任意重定向和文件冲突在 mkdir 前拒绝", (t) => {
  const project = fixture(t),
    p = plan(project);
  p.bundle = join(project, "wrong.app");
  assert.throws(() => prepareDesktopFolders(p), /重定向/);
  write(join(project, "data"), "keep");
  assert.throws(() => prepareDesktopFolders(plan(project)), /无效/);
  assert.equal(readFileSync(join(project, "data"), "utf8"), "keep");
  assert.equal(existsSync(join(project, "tmp")), false);
});
test("规范目录创建，重复实例保持文件并拒绝覆盖", (t) => {
  const p = plan(fixture(t));
  prepareDesktopFolders(p);
  for (const directory of Object.values(p.runtime))
    assert.equal(existsSync(directory), true);
  const record = join(p.archive, "keep.json");
  write(record, "keep");
  assert.throws(() => prepareDesktopFolders(p), /已存在/);
  assert.equal(readFileSync(record, "utf8"), "keep");
});
test("来源祖先文件／目录链接拒绝，程序名称不能穿越", (t) => {
  const folder = fixture(t);
  mkdirSync(join(folder, "real"));
  symlinkSync(join(folder, "real"), join(folder, "link"));
  assert.throws(() => plainAncestors(join(folder, "link/child")), /链接/);
  write(join(folder, "file"));
  assert.throws(() => plainAncestors(join(folder, "file/child")), /无效/);
  for (const name of ["../x", "", "a/b", "x;cmd"])
    assert.throws(() => executableName(name), /无效/);
  assert.equal(executableName("stagemaster-desktop"), "stagemaster-desktop");
});
test("Game 仅签名／模板可变，Pak／嵌套库不能改变或丢失", async (t) => {
  const project = fixture(t),
    p = plan(project);
  write(join(p.copiedGame, "Contents/MacOS/StageMasterPreview"));
  write(join(p.copiedGame, "Contents/UE/Content.pak"));
  const original = await fileInventory(p.copiedGame);
  write(join(p.copiedGame, "Contents/MacOS/StageMasterPreview"), "signed");
  assert.equal((await changedGameFiles(p, original)).length, 1);
  write(join(p.copiedGame, "Contents/UE/Content.pak"), "changed");
  await assert.rejects(changedGameFiles(p, original), /超出/);
  rmSync(join(p.copiedGame, "Contents/UE/Content.pak"));
  await assert.rejects(changedGameFiles(p, original), /丢失/);
});
test("信令不能加观测薄层，桌面 sidecar／图标须逐字保持", async (t) => {
  const project = fixture(t),
    p = plan(project);
  write(join(p.component, "signalling.mjs"));
  const source = await fileInventory(p.component);
  write(join(p.copiedGame, "game"));
  await unchangedSignalling(p, source);
  write(join(p.component, "signalling.mjs"), "observer");
  await assert.rejects(unchangedSignalling(p, source), /信令上下文/);
  const desktop = join(project, "original");
  write(join(desktop, "Contents/MacOS/host"));
  write(join(p.bundle, "Contents/MacOS/host"));
  const original = await fileInventory(desktop);
  await unchangedDesktopResources(p, original, "desktop");
  write(join(p.bundle, "Contents/MacOS/host"), "changed");
  await assert.rejects(
    unchangedDesktopResources(p, original, "desktop"),
    /资源被改变/,
  );
});
test("正式 CLI 缺来源或额外参数直接拒绝，无签名或启动", () => {
  const script = fileURLToPath(
    new URL("./assemble-development-desktop.mjs", import.meta.url),
  );
  for (const values of [[], ["one"], [...sources, "extra"]]) {
    const result = spawnSync(process.execPath, [script, ...values], {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, TMPDIR: join(root, "tmp") },
    });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /三个来源/);
    assert.equal(result.stdout, "");
  }
});
