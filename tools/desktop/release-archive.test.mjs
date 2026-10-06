import assert from "node:assert/strict";
import { test } from "node:test";
import { cpSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { desktopBuildPlan } from "./build-plan.mjs";
import { archiveInternalBundle } from "./release-archive.mjs";
import { fileHash } from "../previs/signalling-package-files.mjs";
import { plistValue } from "../previs/desktop-assembly-files.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
function bundle(folder, identifier) {
  mkdirSync(join(folder, "Contents/MacOS"), { recursive: true });
  writeFileSync(
    join(folder, "Contents/Info.plist"),
    `<?xml version="1.0"?><plist version="1.0"><dict><key>CFBundleIdentifier</key><string>${identifier}</string></dict></plist>`,
  );
  writeFileSync(
    join(folder, "Contents/MacOS/stagemaster-execution-host"),
    "fixed host fixture; not executable qualification",
  );
}
async function fixture(t, resource = Buffer.from('{"version":1}\n')) {
  const folder = mkdtempSync(join(root, "tmp/desktop-013-archive-case-"));
  t.after(() => rmSync(folder, { recursive: true, force: true }));
  const plan = desktopBuildPlan(
    folder,
    "build-internal-release",
    "desktop-release-Own1",
  );
  mkdirSync(plan.archive, { recursive: true });
  const original = join(folder, "original.app");
  bundle(original, plan.identifier);
  mkdirSync(join(original, "Contents/Resources"), { recursive: true });
  writeFileSync(
    join(original, "Contents/Resources/stagemaster-source.json"),
    resource,
  );
  return {
    folder,
    plan,
    original,
    resource,
    record: {
      plan,
      host: {
        sha256: await fileHash(
          join(original, "Contents/MacOS/stagemaster-execution-host"),
        ),
      },
    },
  };
}
test("正确来源／身份／后台真正封存并保留原逐文件清单", async (t) => {
  const f = await fixture(t);
  await archiveInternalBundle(f.record, f.original, undefined, f.resource);
  assert.equal(
    f.record.sourceResourcePath,
    "Contents/Resources/stagemaster-source.json",
  );
  assert.deepEqual(f.record.files, f.record.originalFiles);
});
test("源码资源不一致或缺失均不能借有效主机身份通过", async (t) => {
  const f = await fixture(t);
  await assert.rejects(
    archiveInternalBundle(
      f.record,
      f.original,
      undefined,
      Buffer.from("wrong source"),
    ),
    /来源文件.*不一致/,
  );
  const missing = await fixture(t);
  rmSync(join(missing.original, "Contents/Resources/stagemaster-source.json"));
  await assert.rejects(
    archiveInternalBundle(
      missing.record,
      missing.original,
      undefined,
      missing.resource,
    ),
    /唯一内部来源/,
  );
});
test("后台字节不符和既有归档均拒绝，不覆盖旧目标", async (t) => {
  const f = await fixture(t);
  f.record.host.sha256 = "f".repeat(64);
  await assert.rejects(
    archiveInternalBundle(f.record, f.original, undefined, f.resource),
    /未使用本轮/,
  );
  const existing = await fixture(t);
  await archiveInternalBundle(
    existing.record,
    existing.original,
    undefined,
    existing.resource,
  );
  await assert.rejects(
    archiveInternalBundle(
      existing.record,
      existing.original,
      undefined,
      existing.resource,
    ),
    /EEXIST/,
  );
});
test("复制过程中原目标被另一内部身份替换，归档不能借旧身份检查通过", async (t) => {
  const folder = mkdtempSync(join(root, "tmp/desktop-013-archive-test-"));
  t.after(() => rmSync(folder, { recursive: true, force: true }));
  const plan = desktopBuildPlan(
    folder,
    "build-internal-release",
    "desktop-release-Own1",
  );
  mkdirSync(plan.archive, { recursive: true });
  const original = join(folder, "original.app"),
    replacement = join(folder, "replacement.app");
  bundle(original, plan.identifier);
  bundle(replacement, "cn.stagemaster.acceptance.desktop-release-other");
  const record = {
    plan,
    host: {
      sha256: await fileHash(
        join(original, "Contents/MacOS/stagemaster-execution-host"),
      ),
    },
  };
  const operation = archiveInternalBundle(
    record,
    original,
    (source, target, options) => {
      cpSync(replacement, source, { recursive: true, force: true });
      cpSync(source, target, options);
    },
  );
  await assert.rejects(
    operation.then(() => {
      console.log(
        JSON.stringify({
          planned: plan.identifier,
          archived: plistValue(
            folder,
            join(record.bundle, "Contents/Info.plist"),
            "CFBundleIdentifier",
          ),
          hostHashSame: true,
        }),
      );
    }),
    /归档.*身份|身份.*归档/,
  );
});
