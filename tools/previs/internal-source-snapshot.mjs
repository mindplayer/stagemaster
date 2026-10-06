import assert from "node:assert/strict";
import { existsSync, lstatSync } from "node:fs";
import { join, posix } from "node:path";
import { captureSourceEvidence } from "../desktop/source-evidence.mjs";
import { sourceLimits, sourcePath } from "../desktop/source-scope.mjs";
import {
  verifySourceResource,
  sourceResourceName,
} from "../desktop/source-resource.mjs";

function shape(value, keys) {
  assert.ok(
    value && typeof value === "object" && !Array.isArray(value),
    "优化来源快照结构无效",
  );
  assert.deepEqual(
    Object.keys(value).sort(),
    [...keys].sort(),
    "优化来源快照结构或字段无效",
  );
}
function originalMetadata(source) {
  shape(source.git, ["head", "objectFormat", "dirty", "changedPaths"]);
  const git = source.git;
  assert.ok(
    ["sha1", "sha256"].includes(git.objectFormat),
    "优化来源Git格式无效",
  );
  assert.match(
    git.head,
    git.objectFormat === "sha1" ? /^[a-f0-9]{40}$/ : /^[a-f0-9]{64}$/,
    "优化来源Git基线无效",
  );
  assert.equal(typeof git.dirty, "boolean", "优化来源差异状态无效");
  assert.ok(
    Array.isArray(git.changedPaths) &&
      git.changedPaths.length <= sourceLimits.entries,
    "优化来源差异清单无效",
  );
  assert.equal(
    git.dirty,
    git.changedPaths.length > 0,
    "优化来源差异标记不一致",
  );
  for (const file of git.changedPaths)
    assert.ok(
      typeof file === "string" &&
        file.length <= 4096 &&
        !posix.isAbsolute(file) &&
        posix.normalize(file) === file &&
        !file.split("/").includes("..") &&
        sourcePath(file),
      "优化来源差异路径无效",
    );
  assert.deepEqual(
    git.changedPaths,
    [...new Set(git.changedPaths)].sort(),
    "优化来源差异顺序或重复无效",
  );
  assert.ok(
    typeof source.capturedAt === "string" &&
      source.capturedAt.length <= 64 &&
      Number.isFinite(Date.parse(source.capturedAt)),
    "优化来源快照时间无效",
  );
  assert.equal(
    new Date(source.capturedAt).toISOString(),
    source.capturedAt,
    "优化来源快照时间不规范",
  );
  shape(source.tools, ["node", "platform", "arch"]);
  assert.ok(
    typeof source.tools.node === "string" &&
      source.tools.node.length <= 128 &&
      /^v\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?$/.test(source.tools.node),
    "优化来源Node记录无效",
  );
  assert.equal(source.tools.platform, "darwin", "优化来源构建平台不一致");
  assert.equal(source.tools.arch, "arm64", "优化来源构建架构不一致");
}

/** A bounded input binding, not proof of native, supplier or signing qualification. */
export async function internalSourceSnapshot(
  plan,
  raw,
  actualFiles,
  ownedFile,
) {
  const source = raw.sourceEvidence;
  shape(source, [
    "version",
    "scope",
    "git",
    "fingerprint",
    "totalBytes",
    "inputs",
    "capturedAt",
    "tools",
  ]);
  assert.equal(source.version, 1, "优化来源快照版本不支持");
  assert.equal(raw.customerReleaseQualified, false, "优化来源不得冒称客户资格");
  assert.ok(
    Array.isArray(source.inputs) && source.inputs.length <= sourceLimits.files,
    "优化来源快照输入数量无效",
  );
  originalMetadata(source);
  assert.equal(
    existsSync(join(plan.root, "tmp/desktop-" + raw.plan.instance)),
    false,
    "优化来源实例已存在，不能重复组装",
  );
  const current = await captureSourceEvidence(plan.root);
  for (const key of ["version", "scope", "fingerprint", "totalBytes", "inputs"])
    assert.deepEqual(
      source[key],
      current[key],
      "优化来源完整输入或范围改变：" + key,
    );
  const files = actualFiles.filter(
    (row) =>
      row.path.startsWith("Contents/Resources/") &&
      row.path.split("/").at(-1) === sourceResourceName,
  );
  assert.equal(files.length, 1, "优化来源必须有唯一包内快照");
  assert.equal(raw.sourceResourcePath, files[0].path, "优化来源快照路径不一致");
  const resource = ownedFile(plan.root, join(plan.desktop, files[0].path));
  const bytes = Buffer.from(JSON.stringify(source, null, 2) + "\n");
  assert.equal(
    lstatSync(resource).size,
    bytes.length,
    "优化来源包内快照长度不一致",
  );
  verifySourceResource(plan.desktop, actualFiles, bytes);
  return {
    version: 1,
    git: source.git,
    fingerprint: source.fingerprint,
    inputs: source.inputs.length,
    totalBytes: source.totalBytes,
    resource: files[0].path,
  };
}
