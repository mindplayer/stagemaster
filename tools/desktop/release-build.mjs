import assert from "node:assert/strict";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { desktopBuildPlan, internalEnvironment } from "./build-plan.mjs";
import { prepareHost } from "./prepare-host.mjs";
import {
  plainAncestors,
  plistValue,
} from "../previs/desktop-assembly-files.mjs";
import {
  fileHash,
  fileInventory,
} from "../previs/signalling-package-files.mjs";
import { runCommand } from "../previs/package-renderer.mjs";

export function prepareInternalFolders(plan) {
  assert.deepEqual(
    plan,
    desktopBuildPlan(plan.root, plan.command, plan.instance),
    "内部构建计划不可重定向",
  );
  for (const folder of [
    plan.temporary,
    plan.archive,
    plan.logs,
    plan.target,
    plan.sidecarDirectory,
  ])
    plainAncestors(folder);
  for (const folder of [plan.archive, plan.logs])
    if (existsSync(folder))
      throw new Error(`内部构建目标已存在，不覆盖：${folder}`);
  for (const folder of [
    plan.temporary,
    plan.archive,
    plan.logs,
    plan.target,
    plan.sidecarDirectory,
  ])
    mkdirSync(folder, { recursive: true });
  for (const folder of [
    plan.temporary,
    plan.archive,
    plan.logs,
    plan.target,
    plan.sidecarDirectory,
  ])
    if (realpathSync(folder) !== folder) throw new Error("内部构建目录已改变");
}

export async function buildInternalRelease(root, instance) {
  if (process.platform !== "darwin" || process.arch !== "arm64")
    throw new Error("本增量仅内部Mac ARM64优化桌面验收");
  const plan = desktopBuildPlan(root, "build-internal-release", instance);
  prepareInternalFolders(plan);
  const record = {
    task: "DESKTOP-005",
    status: "running",
    plan,
    customerReleaseQualified: false,
    startedAt: new Date().toISOString(),
  };
  const save = () =>
    writeFileSync(
      join(plan.archive, "build-record.json"),
      JSON.stringify(record, null, 2) + "\n",
    );
  save();
  try {
    const base = JSON.parse(
      readFileSync(join(root, "apps/desktop/tauri.conf.json"), "utf8"),
    );
    const config = {
      ...plan.config,
      app: {
        windows: base.app.windows.map((window) => ({
          ...window,
          title: `${window.title} · 内部发布验收`,
        })),
      },
    };
    writeFileSync(plan.configFile, JSON.stringify(config, null, 2) + "\n", {
      flag: "wx",
    });
    const env = internalEnvironment(plan, process.env);
    record.host = prepareHost(plan, env);
    record.host.sha256 = await fileHash(record.host.output);
    const cli = {
      program: process.execPath,
      args: [
        join(root, "apps/ui-prototype/node_modules/@tauri-apps/cli/tauri.js"),
        ...plan.cliArgs,
      ],
      log: join(plan.logs, "tauri-build.log"),
    };
    record.cli = cli;
    record.commandResult = await runCommand(cli, {
      root: join(root, "apps/desktop"),
      env,
      inheritEnvironment: false,
    });
    const original = join(
      plan.target,
      "release/bundle/macos",
      `${config.productName}.app`,
    );
    const info = join(original, "Contents/Info.plist");
    if (plistValue(root, info, "CFBundleIdentifier") !== plan.identifier)
      throw new Error("内部发布身份与实例不匹配");
    const bundle = join(plan.archive, `${config.productName}.app`);
    cpSync(original, bundle, {
      recursive: true,
      verbatimSymlinks: true,
      errorOnExist: true,
      force: false,
    });
    record.bundle = bundle;
    record.originalFiles = await fileInventory(original);
    record.files = await fileInventory(bundle);
    assert.deepEqual(
      record.files,
      record.originalFiles,
      "内部优化包复制不等价",
    );
    if (
      (await fileHash(
        join(bundle, "Contents/MacOS/stagemaster-execution-host"),
      )) !== record.host.sha256
    )
      throw new Error("内部优化包未使用本轮release后台");
    record.status = "isolated-release-built";
    console.log(
      `内部优化桌面已构建：${bundle}（尚未原生验收，非客户签名／Shipping资格）`,
    );
  } catch (error) {
    record.status = "failed";
    record.error = error.message;
    throw error;
  } finally {
    record.finishedAt = new Date().toISOString();
    save();
  }
  return record;
}
