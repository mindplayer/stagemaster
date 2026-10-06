import assert from "node:assert/strict";
import { cpSync } from "node:fs";
import { join } from "node:path";
import { plistValue } from "../previs/desktop-assembly-files.mjs";
import {
  fileHash,
  fileInventory,
} from "../previs/signalling-package-files.mjs";
import { verifySourceResource } from "./source-resource.mjs";

/** The isolated builder's original archive step, kept separate from compiling. */
export async function archiveInternalBundle(
  record,
  original,
  copy = cpSync,
  sourceBytes = undefined,
) {
  const { plan } = record;
  if (
    plistValue(
      plan.root,
      join(original, "Contents/Info.plist"),
      "CFBundleIdentifier",
    ) !== plan.identifier
  )
    throw new Error("内部发布身份与实例不匹配");
  const bundle = join(plan.archive, `${plan.config.productName}.app`);
  copy(original, bundle, {
    recursive: true,
    verbatimSymlinks: true,
    errorOnExist: true,
    force: false,
  });
  record.bundle = bundle;
  record.originalFiles = await fileInventory(original);
  record.files = await fileInventory(bundle);
  assert.deepEqual(record.files, record.originalFiles, "内部优化包复制不等价");
  if (
    plistValue(
      plan.root,
      join(bundle, "Contents/Info.plist"),
      "CFBundleIdentifier",
    ) !== plan.identifier
  )
    throw new Error("归档内部身份与本轮实例不匹配");
  if (
    (await fileHash(
      join(bundle, "Contents/MacOS/stagemaster-execution-host"),
    )) !== record.host.sha256
  )
    throw new Error("内部优化包未使用本轮release后台");
  if (sourceBytes)
    record.sourceResourcePath = verifySourceResource(
      bundle,
      record.files,
      sourceBytes,
    );
}
