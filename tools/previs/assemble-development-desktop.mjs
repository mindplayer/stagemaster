import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import {
  constants,
  copyFileSync,
  cpSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  desktopAssemblyPlan,
  desktopFileEntitlements,
} from "./desktop-assembly-plan.mjs";
import {
  changedGameFiles,
  desktopSources,
  plainAncestors,
  platformOptions,
  prepareDesktopFolders,
  unchangedSignalling,
  unchangedSources,
  unchangedDesktopResources,
} from "./desktop-assembly-files.mjs";
import { entitlements, entitlementXml } from "./file-access-platform.mjs";
import { fileHash, readJson } from "./signalling-package-files.mjs";
import { inspectMacBundle } from "./mac-bundle-inspection.mjs";
import { compareMacVersions } from "./mac-load-commands.mjs";
import { runCommand } from "./package-renderer.mjs";

async function verify(plan, bundle, name) {
  const command = {
    program: "/usr/bin/codesign",
    args: ["--verify", "--deep", "--strict", bundle],
    log: join(plan.logs, `${name}-verify.log`),
  };
  return { command, completed: await runCommand(command, plan) };
}

export async function assembleDevelopmentDesktop(sources) {
  const root = realpathSync(fileURLToPath(new URL("../../", import.meta.url)));
  const input = desktopAssemblyPlan(root, sources, "previs-desktop-Check");
  // Read and qualify all sources before any output write, signing or launch.
  const original = await desktopSources(input);
  const dependencies = inspectMacBundle(root, input.game);
  for (const image of [
    dependencies,
    original.node,
    original.desktopImage,
    original.hostImage,
  ])
    if (compareMacVersions(image.minimumMacOS, input.minimumMacOS) > 0)
      throw new Error("组件要求高于本内部配置最低系统");
  const engine =
    process.env.STAGEMASTER_UE_ROOT ?? "/Users/Shared/Epic Games/UE_5.8";
  const version = readJson(join(engine, "Engine/Build/Build.version"));
  const template = join(
    engine,
    "Engine/Content/Automation/Report-Template.html",
  );
  if (
    version.MajorVersion !== 5 ||
    version.MinorVersion !== 8 ||
    !readFileSync(template, "utf8").includes("<html")
  )
    throw new Error("需原 UE 5.8 官方报告模板");
  const rights = entitlements(root, input.gameBundle).values;
  desktopFileEntitlements(input, rights);
  for (const directory of ["tmp", "data/PREVIS-007", "logs/PREVIS-007"])
    plainAncestors(join(root, directory));
  mkdirSync(join(root, "tmp"), { recursive: true });
  const temporary = mkdtempSync(join(root, "tmp/previs-desktop-"));
  const plan = desktopAssemblyPlan(root, sources, basename(temporary));
  prepareDesktopFolders(plan);
  const evidence = {
    task: "PREVIS-007",
    status: "running",
    scope: "internal-development-assembly-only",
    customerPackageVerified: false,
    gpuVerified: false,
    original,
    dependencies,
    plan,
    startedAt: new Date().toISOString(),
  };
  const record = () =>
    writeFileSync(
      join(plan.archive, "assembly-record.json"),
      JSON.stringify(evidence, null, 2) + "\n",
    );
  record();
  try {
    evidence.originalGameSignature = await verify(
      plan,
      plan.gameBundle,
      "original-game",
    );
    const baselineSignature = spawnSync(
      "/usr/bin/codesign",
      ["--verify", "--deep", "--strict", plan.desktop],
      platformOptions(root),
    );
    if (baselineSignature.error) throw baselineSignature.error;
    evidence.originalDesktopSignature = {
      status: baselineSignature.status,
      stderr: baselineSignature.stderr,
    };
    const copy = {
      recursive: true,
      verbatimSymlinks: true,
      mode: constants.COPYFILE_FICLONE,
    };
    cpSync(plan.desktop, plan.bundle, copy);
    cpSync(plan.signalling, plan.component, copy);
    cpSync(plan.gameBundle, plan.copiedGame, copy);
    const target = join(
      plan.copiedGame,
      "Contents/UE/Engine/Content/Automation/Report-Template.html",
    );
    mkdirSync(dirname(target), { recursive: true });
    copyFileSync(template, target);
    evidence.template = {
      source: template,
      target,
      sha256: await fileHash(template),
    };
    assert.equal(await fileHash(target), evidence.template.sha256);
    const scoped = desktopFileEntitlements(plan, rights);
    writeFileSync(plan.entitlementFile, entitlementXml(root, scoped), {
      flag: "wx",
    });
    const gameSign = {
      program: "/usr/bin/codesign",
      args: [
        "--force",
        "--sign",
        "-",
        "--timestamp=none",
        "--generate-entitlement-der",
        "--entitlements",
        plan.entitlementFile,
        plan.copiedGame,
      ],
      log: join(plan.logs, "game-sign.log"),
    };
    evidence.gameSign = {
      command: gameSign,
      completed: await runCommand(gameSign, plan),
    };
    evidence.gameSignature = await verify(plan, plan.copiedGame, "copied-game");
    assert.deepEqual(entitlements(root, plan.copiedGame).values, scoped);
    evidence.entitlements = scoped;
    const info = join(plan.bundle, "Contents/Info.plist");
    for (const [key, value] of [
      ["CFBundleIdentifier", plan.bundleId],
      ["LSMinimumSystemVersion", plan.minimumMacOS],
    ])
      execFileSync(
        "/usr/libexec/PlistBuddy",
        ["-c", `Set :${key} ${value}`, info],
        platformOptions(root),
      );
    const outerSign = {
      program: "/usr/bin/codesign",
      args: ["--force", "--sign", "-", "--timestamp=none", plan.bundle],
      log: join(plan.logs, "desktop-sign.log"),
    };
    evidence.desktopSign = {
      command: outerSign,
      completed: await runCommand(outerSign, plan),
    };
    evidence.desktopSignature = await verify(
      plan,
      plan.bundle,
      "copied-desktop",
    );
    evidence.copiedDependencies = inspectMacBundle(
      root,
      join(plan.copiedGame, "Contents/MacOS/StageMasterPreview"),
    );
    await unchangedSignalling(plan, original.files.signalling);
    evidence.changedGameFiles = await changedGameFiles(
      plan,
      original.files.gameBundle,
    );
    evidence.assembledFiles = await unchangedDesktopResources(
      plan,
      original.files.desktop,
      original.executable,
    );
    await unchangedSources(plan, original.files);
    evidence.status = "development-assembled";
    console.log(
      `内部桌面组装完成：${plan.bundle}（未代表实际 GPU／客户发行通过）`,
    );
    return evidence;
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
  assembleDevelopmentDesktop(process.argv.slice(2)).catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
