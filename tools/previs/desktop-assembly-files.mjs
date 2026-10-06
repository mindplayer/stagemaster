import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, lstatSync, mkdirSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { signallingEnvironment } from "./signalling-package-plan.mjs";
import {
  fileInventory,
  lockedPackages,
  packageLicenses,
  qualifyNode,
  readJson,
} from "./signalling-package-files.mjs";
import { desktopAssemblyPlan } from "./desktop-assembly-plan.mjs";
import { internalReleaseSource } from "./desktop-internal-release.mjs";
import { plainAncestors } from "./plain-ancestors.mjs";
export { plainAncestors } from "./plain-ancestors.mjs";

export function prepareDesktopFolders(plan) {
  const expected = desktopAssemblyPlan(
    plan.root,
    [plan.desktop, plan.game, plan.signalling],
    plan.id,
  );
  assert.deepEqual(plan, expected, "组装计划不允许重定向");
  const folders = [
    plan.temporary,
    plan.archive,
    plan.logs,
    plan.instance,
    ...Object.values(plan.runtime),
  ];
  for (const folder of folders) plainAncestors(folder);
  for (const folder of [plan.archive, plan.logs, plan.instance])
    if (existsSync(folder))
      throw new Error(`实例目标已存在，不覆盖：${folder}`);
  for (const folder of folders) mkdirSync(folder, { recursive: true });
  for (const folder of folders)
    if (realpathSync(folder) !== folder) throw new Error("组装目录改变");
}

export function platformOptions(root) {
  return {
    cwd: root,
    encoding: "utf8",
    timeout: 10000,
    maxBuffer: 4 * 1024 * 1024,
    env: {
      ...signallingEnvironment(process.env),
      TMPDIR: join(root, "tmp"),
      NODE_DISABLE_COMPILE_CACHE: "1",
    },
  };
}

export function plistValue(root, file, key) {
  return execFileSync(
    "/usr/libexec/PlistBuddy",
    ["-c", `Print :${key}`, file],
    platformOptions(root),
  ).trim();
}

export function executableName(value) {
  if (!/^[a-zA-Z0-9_-]+$/.test(value)) throw new Error("桌面主程序名称无效");
  return value;
}

export async function desktopSources(plan, releaseRecord) {
  for (const directory of [plan.desktop, plan.gameBundle, plan.signalling]) {
    plainAncestors(directory);
    if (!lstatSync(directory).isDirectory()) throw new Error("来源不是目录");
  }
  if (existsSync(join(plan.desktop, "Contents/Resources/previs")))
    throw new Error("来源桌面已带预演组件，不叠加安装");
  const files = {};
  for (const key of ["desktop", "gameBundle", "signalling"])
    files[key] = await fileInventory(plan[key]);
  const info = join(plan.desktop, "Contents/Info.plist");
  const identifier = plistValue(plan.root, info, "CFBundleIdentifier");
  const internalRelease = releaseRecord
    ? await internalReleaseSource(
        plan,
        releaseRecord,
        files.desktop,
        identifier,
      )
    : undefined;
  if (!internalRelease && identifier !== "cn.stagemaster.desktop")
    throw new Error("不是原 StageMaster 桌面包");
  const executable = executableName(
    plistValue(plan.root, info, "CFBundleExecutable"),
  );
  const desktopImage = qualifyNode(
    plan.root,
    join(plan.desktop, "Contents/MacOS", executable),
  );
  const hostImage = qualifyNode(
    plan.root,
    join(plan.desktop, "Contents/MacOS/stagemaster-execution-host"),
  );
  const node = qualifyNode(plan.root, join(plan.signalling, "node"));
  if (
    execFileSync(
      join(plan.signalling, "node"),
      ["--version"],
      platformOptions(plan.root),
    ).trim() !== "v24.17.0"
  )
    throw new Error("不是已验证的 Node 24.17.0");
  const license = join(plan.signalling, "licenses/node-LICENSE");
  if (!lstatSync(license).isFile() || lstatSync(license).size === 0)
    throw new Error("缺完整 Node 许可");
  const packages = lockedPackages(
    readJson(join(plan.signalling, "package.json")),
    readJson(join(plan.signalling, "package-lock.json")),
  );
  const licenses = packageLicenses(plan.signalling, packages);
  return {
    executable,
    desktopImage,
    hostImage,
    node,
    licenses,
    files,
    internalRelease,
  };
}

export async function unchangedSources(plan, original) {
  for (const key of ["desktop", "gameBundle", "signalling"])
    assert.deepEqual(
      await fileInventory(plan[key]),
      original[key],
      `来源改变：${key}`,
    );
}

export async function unchangedSignalling(plan, original) {
  const files = await fileInventory(plan.component);
  assert.deepEqual(
    files.filter((file) => !file.path.startsWith("StageMasterPreview.app/")),
    original,
    "复制的信令上下文改变",
  );
}

export async function changedGameFiles(plan, original) {
  const copied = await fileInventory(plan.copiedGame);
  const allowed = [
    "Contents/MacOS/StageMasterPreview",
    "Contents/_CodeSignature/CodeResources",
    "Contents/UE/Engine/Content/Automation/Report-Template.html",
  ];
  assert.ok(
    original.every((file) =>
      copied.some((target) => target.path === file.path),
    ),
    "Game 丢失原文件",
  );
  const changed = copied.filter(
    (file) =>
      !original.some(
        (source) => source.path === file.path && source.sha256 === file.sha256,
      ),
  );
  assert.ok(
    changed.every((file) => allowed.includes(file.path)),
    "Game 改变超出主签名／封套／模板",
  );
  return changed;
}

export async function unchangedDesktopResources(plan, original, executable) {
  const copied = await fileInventory(plan.bundle);
  const mutable = [
    "Contents/Info.plist",
    `Contents/MacOS/${executable}`,
    "Contents/_CodeSignature/CodeResources",
  ];
  for (const file of original) {
    const target = copied.find((value) => value.path === file.path);
    assert.ok(target, `桌面丢失原文件：${file.path}`);
    if (!mutable.includes(file.path))
      assert.deepEqual(target, file, `桌面资源被改变：${file.path}`);
  }
  assert.ok(
    copied.every(
      (file) =>
        original.some((source) => source.path === file.path) ||
        mutable.includes(file.path) ||
        file.path.startsWith("Contents/Resources/previs/"),
    ),
    "桌面新增无关资源",
  );
  return copied;
}
