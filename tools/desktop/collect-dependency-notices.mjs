import { existsSync, mkdirSync, lstatSync, readFileSync } from "node:fs";
import { join, resolve, relative, isAbsolute, sep, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { plainAncestors } from "../previs/desktop-assembly-files.mjs";
import {
  noticeHash,
  noticePath,
  saveNoticeFiles,
} from "../previs/signalling-notices-files.mjs";
import { fileHash } from "../previs/signalling-package-files.mjs";
import { collectRustNotices, collectUiNotices } from "./dependency-notices.mjs";

const root = resolve(fileURLToPath(new URL("../../", import.meta.url)));
const sources = [
  "Cargo.toml",
  "Cargo.lock",
  "apps/desktop/Cargo.toml",
  "apps/execution-host/Cargo.toml",
  "apps/ui-prototype/package.json",
  "apps/ui-prototype/package-lock.json",
];
async function inputHashes() {
  const hashes = {};
  for (const source of sources) {
    const file = join(root, source);
    plainAncestors(dirname(file));
    const stat = lstatSync(file);
    if (!stat.isFile() || stat.size > 1024 * 1024)
      throw new Error(`材料来源链接／类型／预算拒绝：${source}`);
    hashes[source] = await fileHash(file);
  }
  return hashes;
}
export function noticeOutput(value) {
  if (typeof value !== "string" || !value || /[\x00-\x1f\x7f]/.test(value))
    throw new Error("需明确全新材料目录");
  const output = resolve(root, value),
    base = join(root, "data/DESKTOP-006"),
    key = relative(base, output);
  if (!key || key === ".." || key.startsWith(`..${sep}`) || isAbsolute(key))
    throw new Error("材料输出仅限data/DESKTOP-006/内的新目录");
  plainAncestors(output);
  if (existsSync(output)) throw new Error(`材料目标已存在，不覆盖：${output}`);
  return output;
}
export async function collectDesktopNotices(value) {
  const output = noticeOutput(value),
    cargoHome = join(root, "tmp/cargo-home");
  const sourceHashes = await inputHashes();
  plainAncestors(cargoHome);
  plainAncestors(join(root, "tmp/framework-001-light-target"));
  const args = [
    "metadata",
    "--format-version",
    "1",
    "--filter-platform",
    "aarch64-apple-darwin",
    "--locked",
    "--offline",
  ];
  const result = spawnSync("cargo", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
    timeout: 60000,
    env: {
      ...process.env,
      TMPDIR: join(root, "tmp"),
      CARGO_HOME: cargoHome,
      CARGO_TARGET_DIR: join(root, "tmp/framework-001-light-target"),
    },
  });
  if (result.error || result.status !== 0)
    throw new Error(
      `锁定离线Cargo元数据失败：${result.error?.message ?? result.stderr}`,
    );
  const metadata = JSON.parse(result.stdout);
  const rust = collectRustNotices(metadata, join(cargoHome, "registry/src"));
  const verified = verifyRustArchives(
    readFileSync(join(root, "Cargo.lock"), "utf8"),
    rust.sources.map((entry) => ({
      ...entry,
      archive: noticePath(
        join(cargoHome, "registry/cache"),
        `${entry.registryDirectory}/${entry.name}-${entry.version}.crate`,
      ),
    })),
  );
  for (const [index, entry] of rust.entries.entries())
    Object.assign(entry, {
      archiveSha256: verified[index].archiveSha256,
      archiveFilesCompared: verified[index].filesCompared,
    });
  const ui = collectUiNotices(join(root, "apps/ui-prototype"));
  const chunks = [
    "StageMaster 桌面／界面第三方告知材料\n仅收集锁定源码资料，不代表法律审查、最终二进制清单或客户发行批准。\n",
  ];
  let bytes = Buffer.byteLength(chunks[0]);
  for (const chunk of [...rust.chunks, ...ui.chunks]) {
    const part = `\n===== ${chunk.label} =====\n${chunk.text}\n`;
    bytes += Buffer.byteLength(part);
    if (bytes > 16 * 1024 * 1024) throw new Error("桌面告知合并预算超限");
    chunks.push(part);
  }
  const text = chunks.join("");
  if (JSON.stringify(sourceHashes) !== JSON.stringify(await inputHashes()))
    throw new Error("收集过程中工程依赖来源发生变化");
  const packages = [...rust.entries, ...ui.entries];
  const report = {
    format: "stagemaster.desktop-notice-materials",
    version: 1,
    target: "aarch64-apple-darwin",
    scope:
      "conservative-non-dev-source-materials-including-build-and-optional-packages",
    cargoCommand: ["cargo", ...args],
    sourceHashes,
    packages,
    missingPackages: packages
      .filter((p) => p.materialStatus !== "collected")
      .map((p) => ({
        ecosystem: p.ecosystem,
        name: p.name ?? p.path,
        version: p.version,
        status: p.materialStatus,
      })),
    textFile: "licenses/THIRD-PARTY-NOTICES.txt",
    textSha256: noticeHash(Buffer.from(text)),
    textBytes: bytes,
    commercialReleaseApproved: false,
    reviewStillRequired: true,
    bundledBinaryInventory: false,
  };
  noticeOutput(value);
  mkdirSync(dirname(output), { recursive: true });
  mkdirSync(output);
  mkdirSync(join(output, "licenses"));
  saveNoticeFiles(output, report, text);
  return { output, report };
}
export function verifyRustArchives(lockText, entries) {
  const result = spawnSync(
    "python3",
    [join(root, "tools/desktop/rust-notice-archives.py")],
    {
      cwd: root,
      encoding: "utf8",
      timeout: 60000,
      maxBuffer: 8 * 1024 * 1024,
      input: JSON.stringify({ lockText, entries }),
      env: {
        ...process.env,
        TMPDIR: join(root, "tmp"),
        PYTHONDONTWRITEBYTECODE: "1",
      },
    },
  );
  if (result.error || result.status !== 0)
    throw new Error(result.error?.message ?? result.stderr.trim());
  const verified = JSON.parse(result.stdout);
  if (!Array.isArray(verified) || verified.length !== entries.length)
    throw new Error("原包核对结果数量无效");
  for (const [index, entry] of entries.entries())
    if (
      verified[index].name !== entry.name ||
      verified[index].version !== entry.version ||
      !/^[a-f0-9]{64}$/.test(verified[index].archiveSha256)
    )
      throw new Error("原包核对结果身份无效");
  return verified;
}
if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  try {
    if (process.argv.length !== 3)
      throw new Error("只接受一个全新项目内材料目录参数");
    const result = await collectDesktopNotices(process.argv[2]);
    console.log(
      JSON.stringify({
        output: result.output,
        packages: result.report.packages.length,
        missing: result.report.missingPackages.length,
        commercialReleaseApproved: false,
      }),
    );
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
