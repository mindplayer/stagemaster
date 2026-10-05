import { execFileSync, spawnSync } from "node:child_process";
import { lstatSync, mkdirSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { withoutLoaderOverrides } from "./packaging-tools.mjs";
import { parseStrict } from "../project-format/check.mjs";

const options = (root) => ({
  cwd: root,
  encoding: "utf8",
  timeout: 10000,
  maxBuffer: 4 * 1024 * 1024,
  env: withoutLoaderOverrides({ ...process.env, TMPDIR: `${root}/tmp` }),
});
export function entitlements(root, bundle) {
  const xml = execFileSync(
    "/usr/bin/codesign",
    ["-d", "--entitlements", "-", "--xml", bundle],
    { ...options(root), stdio: ["ignore", "pipe", "pipe"] },
  );
  const json = execFileSync(
    "/usr/bin/plutil",
    ["-convert", "json", "-o", "-", "--", "-"],
    { ...options(root), input: xml },
  );
  return { xml, values: parseStrict(json) };
}
export function entitlementXml(root, values) {
  return execFileSync(
    "/usr/bin/plutil",
    ["-convert", "xml1", "-o", "-", "--", "-"],
    { ...options(root), input: JSON.stringify(values) },
  );
}
export function canonicalFolders(plan) {
  for (const key of [
    "root",
    "temporary",
    "archive",
    "logs",
    "user",
    "cache",
    "runtimeTemporary",
    "report",
  ])
    if (realpathSync(plan[key]) !== plan[key])
      throw new Error(`运行目录越界或经过链接：${key}`);
}

// Validate every existing ancestor before creating any output: a late check is too late.
export function prepareFileAccessParents(root) {
  const folders = [
    root,
    join(root, "tmp"),
    join(root, "data"),
    join(root, "data/PREVIS-004"),
    join(root, "logs"),
    join(root, "logs/PREVIS-004"),
  ];
  for (const folder of folders) {
    const entry = lstatSync(folder, { throwIfNoEntry: false });
    if (entry && (!entry.isDirectory() || realpathSync(folder) !== folder))
      throw new Error(`输出父目录不是规范目录，写入前拒绝：${folder}`);
  }
  for (const folder of folders) mkdirSync(folder, { recursive: true });
}
export function sandboxDenials(root, target, pid, execute = spawnSync) {
  const result = execute(
    "/usr/bin/log",
    [
      "show",
      "--last",
      "5m",
      "--style",
      "compact",
      "--predicate",
      `eventMessage CONTAINS "${target}"`,
    ],
    options(root),
  );
  if (result.error || result.status !== 0)
    throw new Error("不能取得范围外写入的系统拒绝证据");
  const lines = result.stdout.split(/\r?\n/).map((line) => line.trimEnd());
  const owner = `Sandbox: StageMasterPreview(${pid}) deny(1) file-write-`;
  if (
    !Number.isSafeInteger(pid) ||
    pid < 1 ||
    ["html", "json"].some(
      (extension) =>
        !lines.some(
          (line) =>
            line.includes(owner) &&
            line.endsWith(`${target}/index.${extension}`),
        ),
    )
  )
    throw new Error("未找到所属范围外目录的实际沙盒拒绝");
  return result.stdout;
}
