import {
  constants,
  closeSync,
  existsSync,
  fstatSync,
  lstatSync,
  mkdirSync,
  openSync,
  readSync,
} from "node:fs";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { plainAncestors } from "../previs/desktop-assembly-files.mjs";
import { fileHash, readJson } from "../previs/signalling-package-files.mjs";
import {
  noticeDirectory,
  noticePath,
  noticeText,
  saveNoticeFiles,
} from "../previs/signalling-notices-files.mjs";
import { proveSupplementSources } from "./notice-supplement-proofs.mjs";
import { supplementNotices } from "./notice-supplements.mjs";

const root = resolve(fileURLToPath(new URL("../../", import.meta.url)));
const baseDirectory = join(root, "data/DESKTOP-006/materials-final-a"),
  assets = join(root, "tools/desktop/notices");
export function supplementOutput(value) {
  if (typeof value !== "string" || !value || /[\x00-\x1f\x7f]/.test(value))
    throw new Error("需明确全新补充材料目录");
  const output = resolve(root, value),
    key = relative(join(root, "data/DESKTOP-009"), output);
  if (!key || key === ".." || key.startsWith(`..${sep}`) || isAbsolute(key))
    throw new Error("补充输出仅限data/DESKTOP-009/内的新目录");
  plainAncestors(output);
  if (existsSync(output)) throw new Error("补充目标已存在，不覆盖");
  return output;
}
export function readBaseMaterialText(directory) {
  const file = noticePath(directory, "licenses/THIRD-PARTY-NOTICES.txt"),
    fd = openSync(
      file,
      constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK,
    );
  try {
    const stat = fstatSync(fd);
    if (!stat.isFile() || !stat.size || stat.size > 16 * 1024 * 1024)
      throw new Error("原材料合并全文类型或预算无效");
    const bytes = Buffer.alloc(stat.size + 1);
    let length = 0;
    while (length < bytes.length) {
      const count = readSync(fd, bytes, length, bytes.length - length, null);
      if (!count) break;
      length += count;
    }
    if (length !== stat.size) throw new Error("原材料全文读取中变化");
    const text = new TextDecoder("utf-8", {
      fatal: true,
      ignoreBOM: true,
    }).decode(bytes.subarray(0, length));
    if (!text.trim() || text.includes("\0"))
      throw new Error("原材料全文空白或NUL拒绝");
    return text;
  } finally {
    closeSync(fd);
  }
}
async function sourceHashes(report) {
  const expected = [
    "Cargo.toml",
    "Cargo.lock",
    "apps/desktop/Cargo.toml",
    "apps/execution-host/Cargo.toml",
    "apps/ui-prototype/package.json",
    "apps/ui-prototype/package-lock.json",
  ];
  if (
    !report.sourceHashes ||
    Object.keys(report.sourceHashes).toSorted().join("\n") !==
      expected.toSorted().join("\n")
  )
    throw new Error("原材料工程来源范围不符");
  const hashes = {};
  for (const file of expected) {
    // fileHash is used only after project-local ancestry / plain-file checks.
    const path = join(root, file);
    plainAncestors(dirname(path));
    const stat = lstatSync(path);
    if (!stat.isFile() || stat.size > 1024 * 1024)
      throw new Error(`工程来源链接／类型／预算拒绝：${file}`);
    hashes[file] = await fileHash(path);
    if (hashes[file] !== report.sourceHashes[file])
      throw new Error(`原材料与当前工程来源不符：${file}`);
  }
  return hashes;
}
export async function collectSupplementedNotices(value) {
  const output = supplementOutput(value),
    base = noticeDirectory(baseDirectory),
    report = readJson(noticePath(base, "licenses/notices.json")),
    text = readBaseMaterialText(base),
    reportSha256 = await fileHash(noticePath(base, "licenses/notices.json"));
  const before = await sourceHashes(report),
    proofs = proveSupplementSources(report, assets);
  const result = supplementNotices(
    { report, text, reportSha256 },
    assets,
    proofs,
  );
  if (
    JSON.stringify(before) !== JSON.stringify(await sourceHashes(report)) ||
    reportSha256 !== (await fileHash(noticePath(base, "licenses/notices.json")))
  )
    throw new Error("补充期间工程或原材料发生变化");
  supplementOutput(value);
  mkdirSync(dirname(output), { recursive: true });
  mkdirSync(output);
  mkdirSync(join(output, "licenses"));
  saveNoticeFiles(output, result.report, result.text);
  return { output, ...result };
}
if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  try {
    if (process.argv.length !== 3)
      throw new Error("只接受一个全新项目内材料目录");
    const result = await collectSupplementedNotices(process.argv[2]);
    console.log(
      JSON.stringify({
        output: result.output,
        supplements: result.report.supplementalMaterials.sourceCount,
        installedMissing: result.report.missingPackages.filter(
          (p) => p.status === "missing",
        ).length,
        notInstalled: result.report.missingPackages.filter(
          (p) => p.status === "not-installed",
        ).length,
        commercialReleaseApproved: false,
      }),
    );
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
