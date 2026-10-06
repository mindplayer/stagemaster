import assert from "node:assert/strict";
import { test } from "node:test";
import {
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { captureSourceEvidence } from "../desktop/source-evidence.mjs";
import { fileHash } from "./signalling-package-files.mjs";
import { snapshotAssemblyFixture } from "./internal-source-fixture.mjs";

test("当前自动快照原记录可只读接入组装，不能要求伪造旧20摘要封套", async (t) => {
  const f = await snapshotAssemblyFixture(t);
  const result = await f.check();
  assert.equal(result.sourceCommit, f.raw.sourceEvidence.git.head);
  assert.equal(
    result.sourceBinding.fingerprint,
    f.raw.sourceEvidence.fingerprint,
  );
  assert.equal(result.sourceBinding.git.dirty, false);
  assert.equal(result.customerReleaseQualified, false);
  assert.equal(result.recordSha256, await f.recordHash());
  assert.equal(f.runtimeExists(), false);
});

test("SHA-256真实仓库基线和文档后继不重标原编译来源", async (t) => {
  const f = await snapshotAssemblyFixture(t, "sha256");
  const original = f.raw.sourceEvidence.git.head;
  assert.equal(original.length, 64);
  f.write(join(f.root, "README.md"), "documentation only\n");
  f.commit();
  assert.notEqual(f.git(["rev-parse", "HEAD"]), original);
  const result = await f.check();
  assert.equal(result.sourceCommit, original);
  assert.equal(result.sourceBinding.git.objectFormat, "sha256");
  assert.equal(f.runtimeExists(), false);
});

test("真实未提交输入纳入版本后保留原dirty和差异，不假装纯提交构建", async (t) => {
  const f = await snapshotAssemblyFixture(t);
  const file = "crates/sample/src/lib.rs";
  f.write(join(f.root, file), "pub const VALUE: u8 = 9;\n");
  f.raw.sourceEvidence = await captureSourceEvidence(f.root);
  await f.save();
  const original = f.raw.sourceEvidence.git.head;
  f.commit();
  const result = await f.check();
  assert.equal(result.sourceCommit, original);
  assert.equal(result.sourceBinding.git.dirty, true);
  assert.deepEqual(result.sourceBinding.git.changedPaths, [file]);
  assert.equal(f.runtimeExists(), false);
});

test("编译输入随后变化只读拒绝，源码和包原字节均不回写", async (t) => {
  const f = await snapshotAssemblyFixture(t);
  const original = readFileSync(f.rawFile);
  f.write(join(f.root, "crates/sample/src/lib.rs"), "changed source\n");
  await assert.rejects(f.check(), /完整输入或范围改变/);
  assert.deepEqual(readFileSync(f.rawFile), original);
  assert.equal(f.runtimeExists(), false);
});

for (const [label, change] of [
  ["版本", (source) => (source.version = 2)],
  ["输入摘要", (source) => (source.inputs[0].sha256 = "0".repeat(64))],
  ["输入省略", (source) => source.inputs.pop()],
  ["范围", (source) => source.scope.roots.pop()],
  ["指纹", (source) => (source.fingerprint = "0".repeat(64))],
  ["字节数", (source) => source.totalBytes++],
  ["额外环境字段", (source) => (source.tools.environment = "not permitted")],
  ["差异标记", (source) => (source.git.dirty = true)],
  ["基线格式", (source) => (source.git.head = "short")],
  ["时间", (source) => (source.capturedAt = "not a timestamp")],
])
  test(`原记录与包内来源重新一致散列仍拒绝错误${label}`, async (t) => {
    const f = await snapshotAssemblyFixture(t);
    change(f.raw.sourceEvidence);
    await f.save();
    await assert.rejects(f.check(), /优化来源/);
    assert.equal(f.runtimeExists(), false);
  });

for (const [label, change] of [
  ["未完成", (raw) => (raw.status = "running")],
  ["退出失败", (raw) => (raw.commandResult.code = 1)],
  ["任务", (raw) => (raw.task = "OTHER")],
  ["客户资格", (raw) => (raw.customerReleaseQualified = true)],
  ["已有原生资格", (raw) => (raw.nativeRepresentativeVerified = true)],
  ["后台目标", (raw) => (raw.host.targetDirectory += "-wrong")],
  ["后台能力", (raw) => (raw.host.args = ["build", "--release"])],
  ["隔离能力", (raw) => (raw.cli.args = ["tauri.js", "build"])],
  ["来源路径", (raw) => (raw.sourceResourcePath = "../elsewhere")],
])
  test(`直接来源拒绝${label}，不能绕旧保护`, async (t) => {
    const f = await snapshotAssemblyFixture(t);
    change(f.raw);
    await f.save();
    await assert.rejects(f.check(), /优化来源/);
    assert.equal(f.runtimeExists(), false);
  });

test("现存runtime拒绝，不能复用旧实例或覆盖其记录", async (t) => {
  const f = await snapshotAssemblyFixture(t);
  const runtime = join(f.root, "tmp/desktop-" + f.build.instance);
  mkdirSync(runtime, { recursive: true });
  const sentinel = join(runtime, "kept.txt");
  writeFileSync(sentinel, "keep");
  await assert.rejects(f.check(), /实例已存在/);
  assert.equal(readFileSync(sentinel, "utf8"), "keep");
});

test("包内来源缺失、额外同名、错字节、链接均拒绝", async (t) => {
  for (const kind of ["missing", "duplicate", "bytes", "link"]) {
    const f = await snapshotAssemblyFixture(t);
    const resource = join(
      f.bundle,
      "Contents/Resources/stagemaster-source.json",
    );
    if (kind === "missing") rmSync(resource);
    if (kind === "duplicate")
      f.write(
        join(f.bundle, "Contents/Resources/extra/stagemaster-source.json"),
        "duplicate",
      );
    if (kind === "bytes") writeFileSync(resource, "different bytes");
    if (kind === "link") {
      const target = join(f.bundle, "Contents/Resources/source-copy.json");
      f.write(target, readFileSync(resource));
      rmSync(resource);
      symlinkSync(target, resource);
    }
    await f.save(false);
    await assert.rejects(f.check(), /优化来源|链接|快照|归档来源/);
    assert.equal(f.runtimeExists(), false);
  }
});

test("旧20摘要封套兼容新版原记录，但完整快照始终权威，不能降级子集", async (t) => {
  const f = await snapshotAssemblyFixture(t);
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
  const hashes = {};
  for (const file of [
    ...required,
    ...Array.from({ length: 12 }, (_, i) => `tools/old-source-${i}.mjs`),
  ]) {
    f.write(join(f.root, file), "fixture source " + file);
    hashes[file] = await fileHash(join(f.root, file));
  }
  f.commit();
  f.raw.sourceEvidence = await captureSourceEvidence(f.root);
  await f.save();
  const wrapper = join(f.root, "data/legacy-envelope.json");
  const envelope = {
    task: "DESKTOP-005",
    buildExit: 0,
    hostProfile: "release",
    sourceCommit: f.raw.sourceEvidence.git.head,
    customerReleaseQualified: false,
    nativeRepresentativeVerified: false,
    rawRecord: f.rawFile,
    recordSha256: await f.recordHash(),
    bundle: f.bundle,
    identifier: f.build.identifier,
    archiveFileCount: f.actual().length,
    sourceHashes: hashes,
  };
  f.write(wrapper, JSON.stringify(envelope));
  const result = await f.check(wrapper);
  assert.equal(result.sourceBinding.version, 1);
  f.write(join(f.root, "crates/sample/src/lib.rs"), "changed outside twenty\n");
  await assert.rejects(f.check(wrapper), /完整输入或范围改变/);
  assert.equal(f.runtimeExists(), false);
});

test("直接来源保持身份／准确归档路径／链接和混合格式保护", async (t) => {
  const f = await snapshotAssemblyFixture(t);
  const { internalReleaseSource } =
    await import("./desktop-internal-release.mjs");
  await assert.rejects(
    internalReleaseSource(
      f.plan,
      f.rawFile,
      f.actual(),
      "cn.stagemaster.desktop",
    ),
    /实际Plist/,
  );
  const copy = join(f.root, "data/copied-record.json");
  f.write(copy, readFileSync(f.rawFile));
  await assert.rejects(f.check(copy), /原记录位置/);
  const link = join(f.root, "data/record-link.json");
  symlinkSync(f.rawFile, link);
  await assert.rejects(f.check(link), /链接/);
  f.raw.rawRecord = f.rawFile;
  await f.save();
  await assert.rejects(f.check(), /混用/);
  assert.equal(f.runtimeExists(), false);
});

test("畸形或超预算快照不能按缺省和旧封套继续", async (t) => {
  const f = await snapshotAssemblyFixture(t);
  f.raw.sourceEvidence.inputs = Array(4097).fill({});
  await f.save();
  await assert.rejects(f.check(), /输入数量/);
  f.raw.sourceEvidence = null;
  await f.save();
  await assert.rejects(f.check(), /快照结构/);
  const text = readFileSync(f.rawFile, "utf8");
  f.write(
    f.rawFile,
    text.replace('"task":"DESKTOP-005"', '"task":"OTHER","task":"DESKTOP-005"'),
  );
  await assert.rejects(f.check(), /重复键/);
  assert.equal(f.runtimeExists(), false);
});
