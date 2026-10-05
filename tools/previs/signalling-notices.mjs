import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  lockedPackages,
  packageLicenses,
  readJson,
} from "./signalling-package-files.mjs";
import {
  noticeDirectory,
  noticeHash,
  noticePath,
  noticeText,
  saveNoticeFiles,
} from "./signalling-notices-files.mjs";

const defaultAssets = fileURLToPath(new URL("./notices/", import.meta.url));
const token = (value) =>
  typeof value === "string" &&
  value.length > 0 &&
  value.length <= 512 &&
  !/[\x00-\x1f\x7f]/.test(value);
const matchesSource = (entry, source) =>
  ["name", "version", "license", "integrity"].every(
    (key) => entry[key] === source[key],
  );
function sources(assets, packages) {
  const catalog = readJson(noticePath(assets, "sources.json"));
  if (!Array.isArray(catalog) || catalog.length > 256)
    throw new Error("补充告知来源预算或结构无效");
  const seen = new Set();
  return new Map(
    catalog.map((source) => {
      if (
        !source ||
        ![source.name, source.version, source.license, source.integrity].every(
          token,
        ) ||
        !/^[a-f0-9]{64}$/.test(source.sha256)
      )
        throw new Error("补充告知来源元数据无效");
      if (seen.has(source.name))
        throw new Error(`补充告知目标重复：${source.name}`);
      seen.add(source.name);
      if (!packages.some((entry) => entry.name === source.name))
        throw new Error(`补充告知目标未锁定：${source.name}`);
      if (!packages.some((entry) => matchesSource(entry, source)))
        throw new Error(`补充告知版本／许可／完整性不匹配：${source.name}`);
      if (!["embedded-package", "pinned-upstream"].includes(source.origin))
        throw new Error(`补充告知来源类别无效：${source.name}`);
      if (
        source.origin === "pinned-upstream" &&
        (!/^[a-f0-9]{40}$/.test(source.gitHead) ||
          source.sourceUrl !==
            `https://raw.githubusercontent.com/EpicGames/PixelStreamingInfrastructure/${source.gitHead}/LICENSE.md`)
      )
        throw new Error(`补充告知来源不是固定提交：${source.name}`);
      return [source.name, source];
    }),
  );
}

export function collectNotices(bundle, supplementalDirectory = defaultAssets) {
  const directory = noticeDirectory(bundle),
    assets = noticeDirectory(supplementalDirectory);
  const locked = lockedPackages(
    readJson(noticePath(directory, "package.json")),
    readJson(noticePath(directory, "package-lock.json")),
  );
  if (locked.length > 256) throw new Error("告知锁定包预算超限");
  const packages = packageLicenses(directory, locked).sort((a, b) =>
    a.path < b.path ? -1 : a.path > b.path ? 1 : 0,
  );
  for (const entry of packages) {
    const inferredName = entry.path.slice(
      entry.path.lastIndexOf("node_modules/") + 13,
    );
    if (
      !token(entry.name) ||
      entry.name !== inferredName ||
      ![entry.version, entry.license, entry.integrity].every(token)
    )
      throw new Error(`告知包身份不匹配：${entry.path}`);
  }
  const supplemental = sources(assets, packages);
  const chunks = [
    "StageMaster 信令组件第三方告知材料\n仅记录来源与原文，不代表商业发行批准；仍需全产品许可审查。\n",
  ];
  let total = Buffer.byteLength(chunks[0]);
  const append = (label, text) => {
    const chunk = `\n===== ${label} =====\n${text}\n`;
    total += Buffer.byteLength(chunk);
    if (total > 8 * 1024 * 1024) throw new Error("告知合并文本预算超限");
    chunks.push(chunk);
  };
  const runtime = noticeText(directory, "licenses/node-LICENSE");
  append("Node / licenses/node-LICENSE", runtime.text);
  const runtimeNotice = {
    origin: "package",
    file: "licenses/node-LICENSE",
    sha256: runtime.sha256,
    bytes: runtime.bytes,
  };
  const entries = packages.map((entry) => {
    if (entry.licenseFiles.length > 16)
      throw new Error(`告知单包文件预算超限：${entry.path}`);
    const notices = entry.licenseFiles.toSorted().map((file) => {
      const notice = noticeText(directory, file);
      append(`${entry.name}@${entry.version} / ${file}`, notice.text);
      return {
        origin: "package",
        file,
        sha256: notice.sha256,
        bytes: notice.bytes,
      };
    });
    const source = supplemental.get(entry.name);
    if (source && matchesSource(entry, source)) {
      const base =
        source.origin === "embedded-package"
          ? join(directory, entry.path)
          : assets;
      const notice = noticeText(base, source.file);
      if (notice.sha256 !== source.sha256)
        throw new Error(`补充告知文本哈希不匹配：${entry.path}`);
      const file =
        source.origin === "embedded-package"
          ? `${entry.path}/${source.file}`
          : source.file;
      append(
        `${entry.name}@${entry.version} / ${source.sourceUrl ?? file}`,
        notice.text,
      );
      notices.push({
        origin: source.origin,
        file,
        sha256: notice.sha256,
        bytes: notice.bytes,
        ...(source.origin === "pinned-upstream"
          ? { gitHead: source.gitHead, sourceUrl: source.sourceUrl }
          : {}),
      });
    }
    return {
      name: entry.name,
      path: entry.path,
      version: entry.version,
      license: entry.license,
      integrity: entry.integrity,
      originalNeedsLicenseReview: entry.needsLicenseReview,
      notices,
      materialStatus: notices.length ? "complete" : "missing",
    };
  });
  const text = chunks.join("");
  return {
    report: {
      format: "stagemaster.previs-notice-materials",
      version: 1,
      scope: "node-and-locked-signalling-dependency-notice-materials-only",
      runtimeNotice,
      packages: entries,
      missingPackages: entries
        .filter((entry) => entry.materialStatus === "missing")
        .map((entry) => entry.path),
      textFile: "licenses/THIRD-PARTY-NOTICES.txt",
      textSha256: noticeHash(Buffer.from(text)),
      textBytes: Buffer.byteLength(text),
      commercialReleaseApproved: false,
      reviewStillRequired: true,
    },
    text,
  };
}
export function writeNotices(bundle, assets = defaultAssets) {
  const result = collectNotices(bundle, assets);
  saveNoticeFiles(bundle, result.report, result.text);
  return result;
}
