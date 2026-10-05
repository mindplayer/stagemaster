import assert from "node:assert/strict";
import { test } from "node:test";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { desktopBuildPlan } from "../desktop/build-plan.mjs";
import { desktopAssemblyPlan } from "./desktop-assembly-plan.mjs";
import {
  desktopAssemblyInputs,
  internalReleaseSource,
} from "./desktop-internal-release.mjs";
import { fileHash, fileInventory } from "./signalling-package-files.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const required = [
  "apps/desktop/Cargo.toml",
  "apps/desktop/src/storage_paths.rs",
  "tools/desktop/build-plan.mjs",
  "tools/desktop/release-build.mjs",
  "apps/desktop/tauri.conf.json",
  "apps/ui-prototype/package.json",
  "apps/ui-prototype/package-lock.json",
  "Cargo.lock",
];
function write(file, value) {
  mkdirSync(join(file, ".."), { recursive: true });
  writeFileSync(
    file,
    typeof value === "string" ? value : JSON.stringify(value),
  );
}
async function fixture(t) {
  const project = mkdtempSync(join(root, "tmp/previs-optimized-test-"));
  t.after(() => rmSync(project, { recursive: true, force: true }));
  const build = desktopBuildPlan(
    project,
    "build-internal-release",
    "desktop-release-Own1",
  );
  const bundle = join(build.archive, build.config.productName + ".app");
  for (const file of [
    "Contents/Info.plist",
    "Contents/MacOS/stagemaster-desktop",
    "Contents/MacOS/stagemaster-execution-host",
    "Contents/Resources/icon.icns",
  ])
    write(join(bundle, file), "unchanged " + file);
  const actual = await fileInventory(bundle);
  const hashes = {};
  for (const file of [
    ...required,
    ...Array.from({ length: 12 }, (_, i) => `tools/source-${i}.mjs`),
  ]) {
    write(join(project, file), "source " + file);
    hashes[file] = await fileHash(join(project, file));
  }
  const rawFile = join(build.archive, "build-record.json");
  const raw = {
    task: "DESKTOP-005",
    status: "isolated-release-built",
    plan: build,
    commandResult: { code: 0, signal: null },
    bundle,
    files: structuredClone(actual),
    originalFiles: structuredClone(actual),
    host: {
      profile: "release",
      status: 0,
      targetDirectory: build.target,
      args: build.hostArgs,
      sha256: actual.find((f) => f.path.endsWith("/stagemaster-execution-host"))
        .sha256,
    },
    cli: { args: ["tauri.js", ...build.cliArgs] },
  };
  write(rawFile, raw);
  const file = join(project, "data/committed-build.json");
  const committed = {
    task: "DESKTOP-005",
    sourceCommit: "a".repeat(40),
    rawRecord: rawFile,
    recordSha256: await fileHash(rawFile),
    bundle,
    identifier: build.identifier,
    sourceHashes: hashes,
    buildExit: 0,
    archiveFileCount: actual.length,
    hostProfile: "release",
    customerReleaseQualified: false,
    nativeRepresentativeVerified: false,
  };
  write(file, committed);
  const plan = desktopAssemblyPlan(
    project,
    [
      bundle,
      "data/Game/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
      "data/Node/previs",
    ],
    "previs-desktop-Check",
  );
  return {
    project,
    build,
    file,
    rawFile,
    raw,
    committed,
    plan,
    actual,
    check: () => internalReleaseSource(plan, file, actual, build.identifier),
  };
}

test("明确三个来源，优化记录只能走唯一显式CLI标志", () => {
  const sources = ["desktop", "game", "node"];
  assert.deepEqual(desktopAssemblyInputs(sources), {
    sources,
    releaseRecord: undefined,
  });
  assert.deepEqual(
    desktopAssemblyInputs([...sources, "--internal-release-record", "record"]),
    { sources, releaseRecord: "record" },
  );
  for (const args of [
    [],
    [...sources, "extra"],
    [...sources, "--other", "record"],
    [...sources, "--internal-release-record", ""],
    [...sources, "--internal-release-record", "r", "extra"],
    ["desktop", null, "node"],
  ])
    assert.throws(() => desktopAssemblyInputs(args), /三个来源/);
});
test("完整优化来源绑定原实例和版本，只读资格不创建runtime", async (t) => {
  const f = await fixture(t),
    result = await f.check();
  assert.equal(result.instance, "desktop-release-Own1");
  assert.equal(result.identifier, f.build.identifier);
  assert.equal(result.sourceCommit, f.committed.sourceCommit);
  assert.equal(result.customerReleaseQualified, false);
  assert.equal(result.recordSha256, await fileHash(f.file));
  assert.equal(readFileSync(f.file, "utf8"), JSON.stringify(f.committed));
});
for (const [label, change] of [
  ["提交记录任务", (r) => (r.task = "PREVIS-007")],
  ["未成功构建", (r) => (r.buildExit = 1)],
  ["debug后台", (r) => (r.hostProfile = "debug")],
  ["源码版本", (r) => (r.sourceCommit = "short")],
  ["客户资格", (r) => (r.customerReleaseQualified = true)],
  ["已有原生资格", (r) => (r.nativeRepresentativeVerified = true)],
  ["错误包", (r) => (r.bundle += ".other")],
  ["错误身份", (r) => (r.identifier = "cn.stagemaster.desktop")],
  ["缺文件数", (r) => r.archiveFileCount--],
  ["缺源码清单", (r) => delete r.sourceHashes[required[0]]],
  ["错误源码摘要", (r) => (r.sourceHashes[required[0]] = "0".repeat(64))],
  ["原记录摘要", (r) => (r.recordSha256 = "0".repeat(64))],
])
  test(`优化来源拒绝${label}，不能借旧debug资格`, async (t) => {
    const f = await fixture(t);
    change(f.committed);
    write(f.file, f.committed);
    await assert.rejects(f.check(), /优化来源/);
  });
for (const [label, change] of [
  ["原任务", (r) => (r.task = "OTHER")],
  ["未完成", (r) => (r.status = "running")],
  ["退出失败", (r) => (r.commandResult.code = 1)],
  ["中断", (r) => (r.commandResult.signal = "SIGTERM")],
  ["覆盖计划", (r) => (r.plan.target += "-other")],
  ["原debug后台", (r) => (r.host.profile = "debug")],
  ["后台失败", (r) => (r.host.status = 1)],
  ["后台目标", (r) => (r.host.targetDirectory += "-other")],
  ["关闭audio", (r) => (r.host.args = ["build", "--release"])],
  ["关闭隔离能力", (r) => (r.cli.args = ["tauri.js", "build"])],
  ["原包清单", (r) => (r.files[0].sha256 = "0".repeat(64))],
])
  test(`重新散列的原记录仍拒绝${label}`, async (t) => {
    const f = await fixture(t);
    change(f.raw);
    write(f.rawFile, f.raw);
    f.committed.recordSha256 = await fileHash(f.rawFile);
    write(f.file, f.committed);
    await assert.rejects(f.check(), /优化来源/);
  });
test("错误实际身份与源代码改变拒绝", async (t) => {
  const f = await fixture(t);
  await assert.rejects(
    internalReleaseSource(f.plan, f.file, f.actual, "cn.stagemaster.desktop"),
    /实际Plist/,
  );
  write(join(f.project, required[0]), "changed");
  await assert.rejects(f.check(), /源码／配置／锁改变/);
});
test("记录文件／祖先链接与越界输入拒绝", async (t) => {
  const f = await fixture(t),
    link = join(f.project, "link.json");
  symlinkSync(f.file, link);
  await assert.rejects(
    internalReleaseSource(f.plan, link, f.actual, f.build.identifier),
    /链接/,
  );
  await assert.rejects(
    internalReleaseSource(
      f.plan,
      root + "/Cargo.lock",
      f.actual,
      f.build.identifier,
    ),
    /项目内/,
  );
  const folder = join(f.project, "linked");
  symlinkSync(join(f.project, "data"), folder);
  await assert.rejects(
    internalReleaseSource(
      f.plan,
      join(folder, "committed-build.json"),
      f.actual,
      f.build.identifier,
    ),
    /链接/,
  );
});
test("畸形／重复JSON记录拒绝，不接受最后键覆盖", async (t) => {
  const f = await fixture(t);
  write(f.file, '{"task":"OTHER","task":"DESKTOP-005"}');
  await assert.rejects(f.check(), /重复键/);
});
