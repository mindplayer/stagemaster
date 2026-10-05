import assert from "node:assert/strict";
import {
  constants,
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { basename, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  fileHash,
  fileInventory,
  readJson,
} from "./signalling-package-files.mjs";
import { inspectMacBundle } from "./mac-bundle-inspection.mjs";
import { runCommand } from "./package-renderer.mjs";
import { runtimeAssetTest, successfulReport } from "./packaging-plan.mjs";
import {
  assertFileReport,
  fileAccessPlan,
  fileAccessRun,
  scopedFileEntitlements,
} from "./file-access-plan.mjs";
import {
  canonicalFolders,
  entitlements,
  entitlementXml,
  prepareFileAccessParents,
  sandboxDenials,
} from "./file-access-platform.mjs";

async function signature(plan, bundle, label) {
  const command = {
    program: "/usr/bin/codesign",
    args: ["--verify", "--deep", "--strict", bundle],
    log: join(plan.logs, `${label}-signature.log`),
  };
  const completed = await runCommand(command, plan);
  return { command, completed, ...entitlements(plan.root, bundle) };
}
async function report(plan, program, label) {
  const directory = join(plan.report, label);
  mkdirSync(directory);
  const command = fileAccessRun(plan, program, label, directory);
  const completed = await runCommand(command, plan);
  const json = readJson(join(directory, "index.json"));
  assertFileReport(
    json,
    readFileSync(join(directory, "index.html"), "utf8"),
    readFileSync(join(plan.logs, `${label}-engine.log`), "utf8"),
  );
  return {
    command,
    completed,
    tests: successfulReport(json, runtimeAssetTest),
  };
}

export async function qualifyRendererFiles() {
  if (process.argv.length !== 3)
    throw new Error("文件资格只接受一个项目内独立 Game 主程序路径");
  const root = realpathSync(fileURLToPath(new URL("../../", import.meta.url)));
  const input = fileAccessPlan(
    root,
    process.argv[2],
    "previs-file-access-Check",
  );
  if (
    realpathSync(input.source) !== input.source ||
    realpathSync(input.sourceBundle) !== input.sourceBundle
  )
    throw new Error("来源经过链接，不接受重新定向");
  const original = inspectMacBundle(root, input.source);
  const sourceFiles = await fileInventory(input.sourceBundle);
  const engine =
    process.env.STAGEMASTER_UE_ROOT ?? "/Users/Shared/Epic Games/UE_5.8";
  const templateSource = join(
    engine,
    "Engine/Content/Automation/Report-Template.html",
  );
  const version = readJson(join(engine, "Engine/Build/Build.version"));
  if (version.MajorVersion !== 5 || version.MinorVersion !== 8)
    throw new Error("官方报告模板需来自已验证的 UE 5.8");
  if (
    !existsSync(templateSource) ||
    !readFileSync(templateSource, "utf8").includes("<html")
  )
    throw new Error("缺少官方非空自动化报告模板");
  prepareFileAccessParents(root);
  const temporary = mkdtempSync(join(root, "tmp/previs-file-access-"));
  const plan = fileAccessPlan(root, input.source, basename(temporary));
  for (const folder of [
    plan.archive,
    plan.logs,
    plan.user,
    plan.cache,
    plan.runtimeTemporary,
    plan.report,
  ])
    mkdirSync(folder, { recursive: true });
  canonicalFolders(plan);
  const evidence = {
    status: "running",
    scope: "development-file-qualification-only",
    customerPackageVerified: false,
    plan,
    original,
    version,
    sourceFiles,
    startedAt: new Date().toISOString(),
  };
  const record = () =>
    writeFileSync(
      join(plan.archive, "build-record.json"),
      `${JSON.stringify(evidence, null, 2)}\n`,
    );
  record();
  try {
    evidence.originalSignature = await signature(
      plan,
      plan.sourceBundle,
      "original",
    );
    const rights = scopedFileEntitlements(
      plan,
      evidence.originalSignature.values,
    );
    cpSync(plan.sourceBundle, plan.bundle, {
      recursive: true,
      verbatimSymlinks: true,
      mode: constants.COPYFILE_FICLONE,
    });
    const template = join(
      plan.bundle,
      "Contents/UE/Engine/Content/Automation/Report-Template.html",
    );
    mkdirSync(join(plan.bundle, "Contents/UE/Engine/Content/Automation"), {
      recursive: true,
    });
    copyFileSync(templateSource, template);
    evidence.template = {
      source: templateSource,
      target: template,
      sha256: await fileHash(templateSource),
    };
    assert.equal(await fileHash(template), evidence.template.sha256);
    writeFileSync(plan.entitlementFile, entitlementXml(root, rights), {
      flag: "wx",
    });
    evidence.sign = {
      program: "/usr/bin/codesign",
      args: [
        "--force",
        "--sign",
        "-",
        "--timestamp=none",
        "--generate-entitlement-der",
        "--entitlements",
        plan.entitlementFile,
        plan.bundle,
      ],
      log: join(plan.logs, "development-sign.log"),
    };
    await runCommand(evidence.sign, plan);
    evidence.signature = await signature(plan, plan.bundle, "development");
    assert.deepEqual(evidence.signature.values, rights);
    const program = join(plan.bundle, "Contents/MacOS/StageMasterPreview");
    evidence.dependencies = inspectMacBundle(root, program);
    evidence.allowed = await report(plan, program, "allowed");
    const denied = join(plan.temporary, "denied-report");
    mkdirSync(denied);
    const sentinel = join(denied, "index.json");
    writeFileSync(sentinel, '{"sentinel":"范围外不得覆盖"}\n', { flag: "wx" });
    const hash = await fileHash(sentinel);
    const negative = fileAccessRun(plan, program, "denied", denied);
    const completed = await runCommand(negative, plan);
    assert.equal(await fileHash(sentinel), hash);
    assert.equal(existsSync(join(denied, "index.html")), false);
    assert.match(
      readFileSync(join(plan.logs, "denied-engine.log"), "utf8"),
      /Failed to write test report json/,
    );
    const denials = sandboxDenials(root, denied, completed.pid);
    writeFileSync(join(plan.logs, "scope-denials.log"), denials);
    evidence.denied = {
      command: negative,
      completed,
      sentinel,
      sha256: hash,
      kernelLog: join(plan.logs, "scope-denials.log"),
    };
    evidence.restored = await report(plan, program, "restored");
    assert.deepEqual(await fileInventory(plan.sourceBundle), sourceFiles);
    evidence.sourceFiles = sourceFiles;
    const copied = await fileInventory(plan.bundle);
    const changed = copied.filter(
      (item) =>
        !sourceFiles.some(
          (source) =>
            source.path === item.path && source.sha256 === item.sha256,
        ),
    );
    const permitted = [
      "Contents/MacOS/StageMasterPreview",
      "Contents/_CodeSignature/CodeResources",
      "Contents/UE/Engine/Content/Automation/Report-Template.html",
    ];
    assert.ok(
      changed.every((item) => permitted.includes(item.path)),
      "副本修改超出主程序签名／资源封套／模板",
    );
    assert.ok(
      sourceFiles.every((item) =>
        copied.some((target) => target.path === item.path),
      ),
      "副本丢失原发布文件",
    );
    evidence.changedFiles = changed;
    evidence.status = "development-file-verified";
    console.log(
      `Development 文件资格通过：${plan.bundle}（不是客户权限／GPU／整包验收）`,
    );
  } catch (error) {
    evidence.status = "failed";
    evidence.error = error.message;
    throw error;
  } finally {
    evidence.finishedAt = new Date().toISOString();
    record();
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href)
  qualifyRendererFiles().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
