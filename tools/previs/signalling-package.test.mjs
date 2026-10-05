import assert from "node:assert/strict";
import { test } from "node:test";
import { spawnSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  signallingPackagePlan,
  signallingEnvironment,
  successfulSignallingReport,
} from "./signalling-package-plan.mjs";
import {
  fileInventory,
  lockedPackages,
  packageLicenses,
} from "./signalling-package-files.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const manifest = () => ({ dependencies: { a: "1.0.0" } });
const lock = () => ({
  lockfileVersion: 3,
  packages: {
    "": manifest(),
    "node_modules/a": {
      version: "1.0.0",
      license: "MIT",
      integrity: "sha512-fixture",
      resolved: "https://registry.npmjs.org/a/-/a-1.0.0.tgz",
    },
  },
});
function fixture(t) {
  const folder = mkdtempSync(join(root, "tmp/previs-signalling-test-"));
  t.after(() => rmSync(folder, { recursive: true, force: true }));
  return folder;
}
const write = (file, data) => {
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, data);
};

test("组装只允许 Mac ARM64 与隔离实例，不接受路径穿越", () => {
  for (const [platform, arch] of [
    ["linux", "arm64"],
    ["darwin", "x64"],
  ])
    assert.throws(
      () =>
        signallingPackagePlan(
          root,
          process.execPath,
          "previs-signalling-A1",
          platform,
          arch,
        ),
      /只支持/,
    );
  for (const id of ["", "../other", "previs-signalling-a/other"])
    assert.throws(
      () => signallingPackagePlan(root, process.execPath, id),
      /名称无效/,
    );
});

test("安装限定离线、无脚本／审计／全局／workspace，两个配置独立", () => {
  const plan = signallingPackagePlan(
    "/project space",
    "/installed/node/bin/node",
    "previs-signalling-A1",
  );
  for (const flag of [
    "--offline",
    "--ignore-scripts",
    "--omit=dev",
    "--global=false",
    "--workspaces=false",
    "--audit=false",
    "--fund=false",
  ])
    assert.ok(plan.install.args.includes(flag));
  assert.notEqual(plan.config, plan.globalConfig);
  assert.ok(plan.install.args.includes(`--prefix=${plan.bundle}`));
  for (const field of [
    "temporary",
    "archive",
    "logs",
    "bundle",
    "config",
    "globalConfig",
  ])
    assert.ok(plan[field].startsWith("/project space/"));
  assert.equal(plan.env.NODE_ENV, "production");
  assert.ok(plan.env.NODE_COMPILE_CACHE.startsWith(plan.temporary));
});

test("不继承 NODE／npm／DYLD 加载配置，不改调用者／HOME", () => {
  const original = {
    HOME: "original",
    NODE_OPTIONS: "bad",
    NODE_PATH: "outside",
    NODE_ENV: "test",
    NPM_CONFIG_USERCONFIG: "outside",
    npm_config_cache: "outside",
    DYLD_INSERT_LIBRARIES: "outside",
    PATH: "original",
  };
  assert.deepEqual(signallingEnvironment(original), {
    HOME: "original",
    PATH: "original",
  });
  assert.equal(original.NODE_OPTIONS, "bad");
});

test("锁和清单一致才接纳，依赖对象顺序不是语义", () => {
  assert.equal(lockedPackages(manifest(), lock()).length, 1);
  const a = { dependencies: { a: "1", b: "2" } };
  const b = lock();
  b.packages[""].dependencies = { b: "2", a: "1" };
  assert.equal(lockedPackages(a, b).length, 1);
});

for (const [name, edit] of [
  ["版本不匹配", (l) => (l.packages[""].dependencies.a = "2.0.0")],
  ["旧锁格式", (l) => (l.lockfileVersion = 1)],
  [
    "目录穿越",
    (l) =>
      (l.packages["node_modules/../elsewhere"] = l.packages["node_modules/a"]),
  ],
  ["本地链接", (l) => (l.packages["node_modules/a"].link = true)],
  [
    "非注册源",
    (l) =>
      (l.packages["node_modules/a"].resolved =
        "https://elsewhere.invalid/a.tgz"),
  ],
  ["无散列", (l) => delete l.packages["node_modules/a"].integrity],
  ["无许可元数据", (l) => delete l.packages["node_modules/a"].license],
])
  test(`拒绝${name}`, () => {
    const l = lock();
    edit(l);
    assert.throws(() => lockedPackages(manifest(), l));
  });

test("空依赖不能登记为信令锁定安装", () =>
  assert.throws(
    () =>
      lockedPackages(
        { dependencies: {} },
        { lockfileVersion: 3, packages: { "": { dependencies: {} } } },
      ),
    /不一致/,
  ));

test("许可缺项明确标待审阅，已发布文本不丢弃", (t) => {
  const folder = fixture(t);
  write(
    join(folder, "node_modules/a/package.json"),
    JSON.stringify({ name: "a", version: "1.0.0" }),
  );
  let result = packageLicenses(folder, lockedPackages(manifest(), lock()));
  assert.equal(result[0].needsLicenseReview, true);
  write(join(folder, "node_modules/a/LICENSE"), "fixture notice");
  result = packageLicenses(folder, lockedPackages(manifest(), lock()));
  assert.equal(result[0].needsLicenseReview, false);
  assert.deepEqual(result[0].licenseFiles, ["node_modules/a/LICENSE"]);
  write(
    join(folder, "node_modules/a/package.json"),
    JSON.stringify({ version: "2.0.0" }),
  );
  assert.throws(
    () => packageLicenses(folder, lockedPackages(manifest(), lock())),
    /版本不匹配/,
  );
});

test("文件清单检查字节等价、内部文件别名和禁止原生扩展", async (t) => {
  const folder = fixture(t);
  write(join(folder, "bundle/a.txt"), "same");
  symlinkSync("a.txt", join(folder, "bundle/alias.txt"));
  const files = await fileInventory(join(folder, "bundle"));
  assert.equal(files.length, 2);
  assert.equal(files[0].sha256, files[1].sha256);
  write(join(folder, "bundle/unqualified.node"), "not native");
  await assert.rejects(fileInventory(join(folder, "bundle")), /不支持/);
});

test("组件和许可不能跟随包外链接", async (t) => {
  const folder = fixture(t);
  write(join(folder, "outside"), "outside");
  mkdirSync(join(folder, "bundle"));
  symlinkSync(join(folder, "outside"), join(folder, "bundle/escape"));
  await assert.rejects(fileInventory(join(folder, "bundle")), /越界/);
  write(
    join(folder, "bundle/node_modules/a/package.json"),
    JSON.stringify({ version: "1.0.0" }),
  );
  symlinkSync(
    join(folder, "outside"),
    join(folder, "bundle/node_modules/a/LICENSE"),
  );
  assert.throws(
    () =>
      packageLicenses(
        join(folder, "bundle"),
        lockedPackages(manifest(), lock()),
      ),
    /越界/,
  );
});

const names = [
  "viewer credentials cannot authorize another origin or the renderer",
  "authorized paths are removed before upstream logging",
  "actual loopback service rejects unauthorized upgrades and negotiates official config",
  "a waiting viewer discovers a late renderer without another list request or connection",
  "a renderer ready before the initial directory request cannot double-discover a viewer",
  "a cancelled waiting viewer stays closed; a new viewer discovers the current renderer",
  "父进程 EOF 结束所属信令服务并释放两个真实回环端口",
];
const report = () =>
  names.map((name) => `# Subtest: ${name}`).join("\n") +
  "\n# tests 7\n# pass 7\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n";
test("完整原用例＋生命周期报告才通过", () =>
  assert.equal(successfulSignallingReport(report()).length, 7));
for (const [name, edit] of [
  ["空", () => ""],
  ["缺用例", (s) => s.replace(names[0], "other")],
  ["失败", (s) => s.replace("# fail 0", "# fail 1")],
  ["取消", (s) => s.replace("# cancelled 0", "# cancelled 1")],
  ["跳过", (s) => s.replace("# skipped 0", "# skipped 1")],
  ["数不符", (s) => s.replace("# pass 7", "# pass 6")],
  ["重复摘要", (s) => s + "# tests 7\n"],
])
  test(`报告拒绝${name}`, () =>
    assert.throws(() => successfulSignallingReport(edit(report())), /报告/));

test("正式组装 CLI 额外参数在写入前拒绝", () => {
  const script = fileURLToPath(
    new URL("./package-signalling.mjs", import.meta.url),
  );
  const result = spawnSync(process.execPath, [script, "unexpected"], {
    cwd: root,
    env: { ...process.env, TMPDIR: join(root, "tmp") },
    encoding: "utf8",
  });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /不接受额外参数/);
  assert.equal(result.stdout, "");
});
