import assert from "node:assert/strict";
import {
  constants,
  closeSync,
  cpSync,
  fsyncSync,
  ftruncateSync,
  lstatSync,
  openSync,
  readlinkSync,
  readdirSync,
  realpathSync,
  writeFileSync,
  writeSync,
} from "node:fs";
import { dirname, join, relative } from "node:path";
import { spawnSync } from "node:child_process";
import {
  platformOptions,
  executableName,
  plainAncestors,
} from "../previs/desktop-assembly-files.mjs";
import {
  fileHash,
  fileInventory,
} from "../previs/signalling-package-files.mjs";
import { handoffBytes, handoffPath } from "./notice-handoff-paths.mjs";
export function permissionTree(bundle) {
  const entries = [];
  let totalBytes = 0;
  function visit(file) {
    if (entries.length >= 8192)
      throw new Error("候选目录／链接权限清单预算超限");
    const stat = lstatSync(file),
      key = relative(bundle, file);
    if (stat.isFile()) {
      totalBytes += stat.size;
      if (stat.size > 1024 ** 3 || totalBytes > 4 * 1024 ** 3)
        throw new Error("候选文件字节预算超限");
    }
    const kind = stat.isDirectory()
      ? "directory"
      : stat.isSymbolicLink()
        ? "symlink"
        : stat.isFile()
          ? "file"
          : "special";
    if (kind === "special") throw new Error("候选特殊文件拒绝");
    const entry = { path: key, kind, mode: stat.mode & 0o7777 };
    if (kind === "symlink") {
      entry.target = readlinkSync(file);
      if (entry.target.length > 2048) throw new Error("候选链接预算超限");
      if (!realpathSync(file).startsWith(realpathSync(bundle) + "/"))
        throw new Error("候选链接越出包范围");
    }
    entries.push(entry);
    if (kind === "directory")
      for (const name of readdirSync(file).sort()) visit(join(file, name));
  }
  visit(bundle);
  return entries;
}
export function inspectHandoffSignatures(root, bundle, name) {
  if (process.platform !== "darwin" || process.arch !== "arm64")
    throw new Error("交接签名只读核验仅支持Mac ARM64");
  executableName(name);
  const paths = {
    desktop: bundle,
    host: join(bundle, "Contents/MacOS/stagemaster-execution-host"),
    renderer: join(bundle, "Contents/Resources/previs/StageMasterPreview.app"),
    node: join(bundle, "Contents/Resources/previs/node"),
  };
  const results = [];
  for (const [role, file] of Object.entries(paths)) {
    const verify = spawnSync(
      "/usr/bin/codesign",
      ["--verify", "--strict", file],
      { ...platformOptions(root), maxBuffer: 64 * 1024 },
    );
    if (verify.error || verify.status !== 0)
      throw new Error("交接原签名核验失败：" + role);
    const display = spawnSync(
      "/usr/bin/codesign",
      ["-d", "--verbose=4", "--entitlements", ":-", file],
      { ...platformOptions(root), maxBuffer: 64 * 1024 },
    );
    if (display.error || display.status !== 0)
      throw new Error("交接原资格读取失败：" + role);
    results.push({
      role,
      verifyExit: verify.status,
      readExit: display.status,
      details: display.stdout + display.stderr,
    });
  }
  return results;
}
export function copyHandoffBundle(source, destination) {
  cpSync(source, destination, {
    recursive: true,
    verbatimSymlinks: true,
    errorOnExist: true,
    force: false,
  });
}
export function validateSignatureReports(entries) {
  assert.deepEqual(
    entries?.map((e) => e.role),
    ["desktop", "host", "renderer", "node"],
    "交接签名入口身份不一致",
  );
  for (const entry of entries)
    assert.ok(
      entry.verifyExit === 0 &&
        entry.readExit === 0 &&
        typeof entry.details === "string" &&
        Buffer.byteLength(entry.details) <= 64 * 1024,
      "交接签名诊断未通过或预算超限",
    );
}
export function saveHandoffFile(path, bytes) {
  plainAncestors(dirname(path));
  writeFileSync(path, bytes, { flag: "wx" });
}
export function progressReceipt(path, record) {
  plainAncestors(dirname(path));
  const fd = openSync(
    path,
    constants.O_CREAT |
      constants.O_EXCL |
      constants.O_WRONLY |
      constants.O_NOFOLLOW,
    0o600,
  );
  const identity = lstatSync(path);
  const update = () => {
    const current = lstatSync(path);
    if (
      !current.isFile() ||
      current.isSymbolicLink() ||
      current.dev !== identity.dev ||
      current.ino !== identity.ino
    )
      throw new Error("交接回执所有权已变化");
    const bytes = Buffer.from(JSON.stringify(record, null, 2) + "\n");
    ftruncateSync(fd, 0);
    let offset = 0;
    while (offset < bytes.length)
      offset += writeSync(fd, bytes, offset, bytes.length - offset, offset);
    fsyncSync(fd);
  };
  try {
    update();
  } catch (error) {
    closeSync(fd);
    throw error;
  }
  return { update, close: () => closeSync(fd) };
}
export async function unchangedHandoffInputs(inputs) {
  for (const entry of inputs.evidence)
    assert.equal(
      await fileHash(handoffPath(inputs.root, entry.path)),
      entry.sha256,
      "交接来源证据在复制中变化",
    );
  for (const [key, hash] of Object.entries(
    inputs.sourceEvidence.productSourceHashes,
  ))
    assert.equal(
      await fileHash(handoffPath(inputs.root, key)),
      hash,
      "交接产品来源在复制中变化：" + key,
    );
  for (const [key, hash] of Object.entries(inputs.sourceEvidence.sourceHashes))
    assert.equal(
      await fileHash(handoffPath(inputs.root, key)),
      hash,
      "交接构建来源在复制中变化：" + key,
    );
  assert.deepEqual(
    await fileInventory(inputs.bundle),
    inputs.files,
    "交接原候选在复制中变化",
  );
  for (const original of [
    inputs.materials.reportInput,
    inputs.materials.textInput,
  ])
    assert.deepEqual(
      handoffBytes(inputs.root, original.path, 16 * 1024 * 1024).bytes,
      original.bytes,
      "交接原材料在复制中变化",
    );
}
