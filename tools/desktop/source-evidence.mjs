import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { sourceGit } from "./source-git.mjs";
import { sourceFiles } from "./source-files.mjs";
import {
  sourceRoots,
  sourceExtras,
  sourcePath,
  sourceLimits,
  requiredInputs,
} from "./source-scope.mjs";

export async function captureSourceEvidence(root, limits = sourceLimits) {
  const git = sourceGit(root),
    { rows, total } = await sourceFiles(root, git.objectFormat, limits);
  const byPath = new Map(rows.map((row) => [row.path, row]));
  for (const required of requiredInputs)
    if (!byPath.has(required)) throw Error("构建来源缺少必要输入：" + required);
  const changed = new Set();
  for (const row of rows) {
    const original = git.tree.get(row.path);
    if (
      !original ||
      original.oid !== row.gitBlob ||
      (original.mode === "100755") !== row.executable
    )
      changed.add(row.path);
  }
  for (const path of git.tree.keys())
    if (sourcePath(path) && !byPath.has(path)) changed.add(path);
  const inputs = rows.map(({ gitBlob, ...row }) => row);
  const fingerprint = createHash("sha256")
    .update(JSON.stringify(inputs))
    .digest("hex");
  return {
    version: 1,
    scope: {
      roots: sourceRoots,
      extras: sourceExtras,
      excluded:
        "tests/examples/generated/cache/dependency trees, unrelated docs, runtime/data/output; not whole supply chain or immutable checkout",
    },
    git: {
      head: git.head,
      objectFormat: git.objectFormat,
      dirty: changed.size > 0,
      changedPaths: [...changed].sort(),
    },
    fingerprint,
    totalBytes: total,
    inputs,
    capturedAt: new Date().toISOString(),
    tools: {
      node: process.version,
      platform: process.platform,
      arch: process.arch,
    },
  };
}
export function sameSourceEvidence(before, after) {
  assert.equal(after.git.head, before.git.head, "构建期间 Git 基线发生变化");
  assert.equal(
    after.fingerprint,
    before.fingerprint,
    "构建期间来源输入发生变化",
  );
  assert.deepEqual(after.git, before.git, "构建期间来源差异状态发生变化");
}
