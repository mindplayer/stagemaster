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
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  desktopBuildPlan,
  hostEnvironment,
  internalEnvironment,
  projectTarget,
} from "./build-plan.mjs";
import {
  buildInternalRelease,
  prepareInternalFolders,
} from "./release-build.mjs";
import { runCommand } from "../previs/package-renderer.mjs";
import { prepareWorkspaceFolders } from "./workspace-folders.mjs";
const root = fileURLToPath(new URL("../../", import.meta.url));
function fixture(t) {
  const folder = mkdtempSync(join(root, "tmp/desktop-release-plan-test-"));
  t.after(() => rmSync(folder, { recursive: true, force: true }));
  return {
    folder,
    plan: desktopBuildPlan(
      folder,
      "build-internal-release",
      "desktop-release-Own1",
    ),
  };
}

test("内部计划在所属目录准备，不重定向或重用旧目标", (t) => {
  const { plan } = fixture(t);
  prepareInternalFolders(plan);
  assert.ok(
    existsSync(plan.archive) &&
      existsSync(plan.logs) &&
      existsSync(plan.sidecarDirectory),
  );
  assert.throws(() => prepareInternalFolders(plan), /已存在/);
  assert.throws(
    () => prepareInternalFolders({ ...plan, target: "/tmp/outside" }),
    /不可重定向/,
  );
});
test("链接父目录在任何新写入前拒绝", (t) => {
  const { plan, folder } = fixture(t);
  const outside = join(folder, "other-owner");
  mkdirSync(outside);
  symlinkSync(outside, join(folder, "data"));
  assert.throws(() => prepareInternalFolders(plan), /链接/);
  for (const path of [plan.archive, plan.logs, plan.sidecarDirectory])
    assert.equal(existsSync(path), false);
});
for (const linked of ["tmp", "logs"]) {
  test(`公共入口的${linked}链接在任何新目录写入前拒绝`, (t) => {
    const { folder } = fixture(t);
    const outside = join(folder, "different-owner");
    mkdirSync(outside);
    symlinkSync(outside, join(folder, linked));
    assert.throws(() => prepareWorkspaceFolders(folder), /链接/);
    assert.equal(existsSync(join(outside, "cargo-home")), false);
    assert.equal(existsSync(join(outside, "DESKTOP-001")), false);
    if (linked === "logs") assert.equal(existsSync(join(folder, "tmp")), false);
  });
}
test("后台缓存链接也在公共目录写入前拒绝", (t) => {
  const { folder } = fixture(t);
  const outside = join(folder, "different-owner");
  mkdirSync(outside);
  symlinkSync(outside, join(folder, "target-link"));
  assert.throws(
    () => prepareWorkspaceFolders(folder, [join(folder, "target-link")]),
    /链接/,
  );
  assert.equal(existsSync(join(folder, "logs")), false);
});
test("后台目标越界在任何公共目录写入前拒绝", (t) => {
  const { folder } = fixture(t);
  assert.throws(() => prepareWorkspaceFolders(folder, ["/tmp"]), /项目内/);
  assert.equal(existsSync(join(folder, "tmp")), false);
});
test("原debug后台临时目录仍强制项目tmp，不继承系统目录", () => {
  const plan = desktopBuildPlan(root, "build");
  const env = hostEnvironment(plan, {
    TMPDIR: "/outside",
    HOME: "unchanged",
    CARGO_TARGET_DIR: join(root, "tmp/test-target"),
  });
  assert.equal(env.TMPDIR, join(root, "tmp"));
  assert.equal(env.HOME, "unchanged");
  assert.equal(env.CARGO_TARGET_DIR, join(root, "tmp/test-target"));
});
test("内部后台临时目录／目标由准确计划持有，原环境不修改", () => {
  const plan = desktopBuildPlan(
    root,
    "build-internal-release",
    "desktop-release-Own1",
  );
  const original = {
    CARGO_TARGET_DIR: "/other-project",
    NODE_OPTIONS: "outside",
    DYLD_LIBRARY_PATH: "outside",
    APPLE_API_KEY_PATH: "private",
    TAURI_SIGNING_PRIVATE_KEY: "private",
    HOME: "unchanged",
  };
  const env = internalEnvironment(plan, original);
  assert.equal(env.CARGO_TARGET_DIR, plan.target);
  assert.equal(env.TMPDIR, plan.temporary);
  for (const key of [
    "NODE_OPTIONS",
    "DYLD_LIBRARY_PATH",
    "APPLE_API_KEY_PATH",
    "TAURI_SIGNING_PRIVATE_KEY",
  ])
    assert.equal(key in env, false);
  assert.equal(env.HOME, "unchanged");
  assert.equal(original.NODE_OPTIONS, "outside");
  assert.equal(hostEnvironment(plan, env).TMPDIR, plan.temporary);
});
for (const target of [root, "/tmp", "../other-project"]) {
  test(`目标${target}不能写项目根或外部`, () =>
    assert.throws(() => projectTarget(root, target), /项目内/));
}
for (const args of [
  ["build", "--release"],
  ["build-internal-release", "--extra"],
  ["shipping"],
]) {
  test(`CLI ${args.join(" ")} 在任何构建前拒绝`, () => {
    const result = spawnSync(
      process.execPath,
      [join(root, "tools/desktop/run.mjs"), ...args],
      {
        cwd: root,
        env: { ...process.env, TMPDIR: join(root, "tmp") },
        encoding: "utf8",
      },
    );
    assert.equal(result.status, 1);
    assert.match(result.stderr, /额外参数|命令无效/);
    assert.equal(result.stdout, "");
  });
}

test("明确隔离的真实子进程不重新继承已清理的父环境", async (t) => {
  const { folder } = fixture(t);
  const key = "TAURI_SIGNING_STAGEMASTER_PARENT_TEST";
  const old = process.env[key];
  process.env[key] = "test-only-not-a-secret";
  try {
    await runCommand(
      {
        program: process.execPath,
        args: [
          "-e",
          `console.log(JSON.stringify({inherited: ${JSON.stringify(key)} in process.env, tmp: process.env.TMPDIR}))`,
        ],
        log: join(folder, "child.log"),
      },
      { root: folder, env: { TMPDIR: folder }, inheritEnvironment: false },
    );
    assert.deepEqual(
      JSON.parse(readFileSync(join(folder, "child.log"), "utf8")),
      {
        inherited: false,
        tmp: folder,
      },
    );
  } finally {
    if (old === undefined) delete process.env[key];
    else process.env[key] = old;
  }
});

test("真实子进程非零退出不得登记成功", async (t) => {
  const { folder } = fixture(t);
  await assert.rejects(
    runCommand(
      {
        program: process.execPath,
        args: ["-e", "process.exit(9)"],
        log: join(folder, "failed-child.log"),
      },
      { root: folder, env: { TMPDIR: folder }, inheritEnvironment: false },
    ),
    /子进程失败：9/,
  );
});

test("内部实际构建准备后失败保存failed记录而非资格成功", async (t) => {
  const { folder, plan } = fixture(t);
  await assert.rejects(
    buildInternalRelease(folder, plan.instance),
    /ENOENT|仅内部Mac/,
  );
  if (process.platform === "darwin" && process.arch === "arm64") {
    const record = JSON.parse(
      readFileSync(join(plan.archive, "build-record.json"), "utf8"),
    );
    assert.equal(record.status, "failed");
    assert.equal(record.customerReleaseQualified, false);
    assert.equal("bundle" in record, false);
    assert.ok(record.finishedAt && record.error.includes("ENOENT"));
  }
});
