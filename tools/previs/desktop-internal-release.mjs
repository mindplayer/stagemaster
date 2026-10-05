import assert from "node:assert/strict";
import { lstatSync, realpathSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { readJson } from "../project-format/check.mjs";
import { desktopBuildPlan } from "../desktop/build-plan.mjs";
import { fileHash } from "./signalling-package-files.mjs";

export function desktopAssemblyInputs(args) {
  if (
    !Array.isArray(args) ||
    ![3, 5].includes(args.length) ||
    args.some((value) => typeof value !== "string" || !value) ||
    (args.length === 5 && args[3] !== "--internal-release-record")
  )
    throw new Error("需明确三个来源及可选的内部优化构建记录");
  return { sources: args.slice(0, 3), releaseRecord: args[4] };
}

function ownedFile(root, value) {
  assert.equal(typeof value, "string", "优化来源文件路径无效");
  const file = resolve(root, value),
    inside = relative(root, file);
  assert.ok(
    inside &&
      inside !== ".." &&
      !inside.startsWith(`..${sep}`) &&
      !isAbsolute(inside),
    "优化来源文件必须在项目内",
  );
  assert.ok(
    lstatSync(file).isFile() && realpathSync(file) === file,
    "优化来源文件经过链接或不是文件",
  );
  for (let folder = dirname(file); ; folder = dirname(folder)) {
    assert.ok(
      lstatSync(folder).isDirectory() && realpathSync(folder) === folder,
      "优化来源文件祖先经过链接",
    );
    if (folder === root) break;
  }
  return file;
}

// Internal build provenance only, not a certificate or customer distribution qualification.
export async function internalReleaseSource(
  plan,
  recordPath,
  actualFiles,
  identifier,
) {
  const file = ownedFile(plan.root, recordPath),
    committed = readJson(file);
  assert.equal(committed.task, "DESKTOP-005", "优化来源不是既定构建记录");
  assert.equal(committed.buildExit, 0, "优化来源构建未成功");
  assert.equal(committed.hostProfile, "release", "优化来源后台不是release");
  assert.match(
    committed.sourceCommit,
    /^[a-f0-9]{40}$/,
    "优化来源源码版本无效",
  );
  assert.equal(
    committed.customerReleaseQualified,
    false,
    "优化来源不得冒称客户资格",
  );
  assert.equal(
    committed.nativeRepresentativeVerified,
    false,
    "优化来源实例必须尚未原生验收",
  );
  const rawFile = ownedFile(plan.root, committed.rawRecord);
  assert.equal(
    await fileHash(rawFile),
    committed.recordSha256,
    "优化来源原构建记录改变",
  );
  const raw = readJson(rawFile);
  assert.equal(raw.task, "DESKTOP-005", "优化来源原记录任务错误");
  assert.equal(raw.status, "isolated-release-built", "优化来源原构建未完成");
  assert.equal(raw.commandResult?.code, 0, "优化来源原构建退出失败");
  assert.equal(raw.commandResult?.signal, null, "优化来源原构建被中断");
  const expected = desktopBuildPlan(
    plan.root,
    "build-internal-release",
    raw.plan?.instance,
  );
  assert.deepEqual(raw.plan, expected, "优化来源构建计划／能力／命令改变");
  assert.equal(
    rawFile,
    join(expected.archive, "build-record.json"),
    "优化来源原记录位置错误",
  );
  const bundle = join(expected.archive, `${expected.config.productName}.app`);
  assert.equal(plan.desktop, bundle, "优化来源不是准确归档包");
  assert.equal(committed.bundle, bundle, "优化来源提交记录包不一致");
  assert.equal(raw.bundle, bundle, "优化来源原记录包不一致");
  assert.equal(
    committed.identifier,
    expected.identifier,
    "优化来源提交身份错误",
  );
  assert.equal(identifier, expected.identifier, "优化来源实际Plist身份错误");
  assert.equal(raw.host?.profile, "release", "优化来源原后台不是release");
  assert.equal(raw.host?.status, 0, "优化来源原后台构建失败");
  assert.equal(
    raw.host?.targetDirectory,
    expected.target,
    "优化来源后台目标错误",
  );
  assert.deepEqual(
    raw.host?.args,
    expected.hostArgs,
    "优化来源后台能力／命令错误",
  );
  assert.deepEqual(
    raw.cli?.args.slice(1),
    expected.cliArgs,
    "优化来源桌面能力／命令错误",
  );
  assert.deepEqual(actualFiles, raw.files, "优化来源包文件改变");
  assert.deepEqual(raw.originalFiles, raw.files, "优化来源复制不等价");
  assert.equal(
    committed.archiveFileCount,
    actualFiles.length,
    "优化来源文件数不一致",
  );
  const host = actualFiles.find(
    (entry) => entry.path === "Contents/MacOS/stagemaster-execution-host",
  );
  assert.ok(host && host.sha256 === raw.host.sha256, "优化来源包内后台不一致");
  const hashes = committed.sourceHashes;
  assert.ok(
    hashes && Object.keys(hashes).length === 20,
    "优化来源缺完整源码／配置／锁记录",
  );
  for (const required of [
    "apps/desktop/Cargo.toml",
    "apps/desktop/src/storage_paths.rs",
    "tools/desktop/build-plan.mjs",
    "tools/desktop/release-build.mjs",
    "apps/desktop/tauri.conf.json",
    "apps/ui-prototype/package.json",
    "apps/ui-prototype/package-lock.json",
    "Cargo.lock",
  ])
    assert.ok(Object.hasOwn(hashes, required), `优化来源缺少记录：${required}`);
  for (const [source, hash] of Object.entries(hashes)) {
    assert.match(hash, /^[a-f0-9]{64}$/, "优化来源文件摘要无效");
    assert.equal(
      await fileHash(ownedFile(plan.root, source)),
      hash,
      `优化来源源码／配置／锁改变：${source}`,
    );
  }
  return {
    record: file,
    recordSha256: await fileHash(file),
    rawRecord: rawFile,
    sourceCommit: committed.sourceCommit,
    instance: expected.instance,
    identifier: expected.identifier,
    profile: "release",
    customerReleaseQualified: false,
  };
}
