import {
  noticeDirectory,
  noticeHash,
  noticePath,
  noticeText,
} from "../previs/signalling-notices-files.mjs";
import { readJson } from "../previs/signalling-package-files.mjs";
import {
  verifiedOriginal,
  verifyIdentity,
} from "./notice-supplement-identities.mjs";

const fail = (message) => {
  throw new Error(message);
};
const check = (condition, message) => {
  if (!condition) fail(message);
};
const digest = (value) =>
  typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
const key = (value) =>
  `${value.ecosystem}/${value.name ?? value.path}@${value.version}`;
function unique(values, label, limit = 1024) {
  check(Array.isArray(values) && values.length <= limit, `${label}预算无效`);
  const entries = new Map();
  for (const value of values) {
    check(
      value &&
        ["cargo", "npm"].includes(value.ecosystem) &&
        typeof value.version === "string" &&
        typeof (value.name ?? value.path) === "string",
      `${label}身份无效`,
    );
    check(!entries.has(key(value)), `${label}身份重复`);
    entries.set(key(value), value);
  }
  return entries;
}
export function supplementNotices(base, assetDirectory, proofEntries) {
  const assets = noticeDirectory(assetDirectory),
    catalogText = noticeText(assets, "sources.json"),
    catalog = readJson(noticePath(assets, "sources.json")),
    sourceReport = base.report;
  check(
    sourceReport?.format === "stagemaster.desktop-notice-materials" &&
      sourceReport.version === 1 &&
      sourceReport.commercialReleaseApproved === false &&
      sourceReport.reviewStillRequired === true &&
      sourceReport.bundledBinaryInventory === false,
    "原材料不是未批准的保守源码集",
  );
  check(
    typeof base.text === "string" &&
      Buffer.byteLength(base.text) <= 16 * 1024 * 1024 &&
      noticeHash(Buffer.from(base.text)) === sourceReport.textSha256 &&
      Buffer.byteLength(base.text) === sourceReport.textBytes,
    "原材料全文变化或预算超限",
  );
  check(
    catalog.format === "stagemaster.desktop-notice-supplements" &&
      catalog.version === 1 &&
      digest(base.reportSha256) &&
      catalog.baseReportSha256 === base.reportSha256 &&
      catalog.baseTextSha256 === sourceReport.textSha256,
    "补充未绑定原材料",
  );
  const packages = unique(sourceReport.packages, "原包"),
    sources = unique(catalog.entries, "补充", 128),
    proofs = unique(proofEntries, "证明", 128);
  check(proofs.size === sources.size, "补充证明缺失或多余");
  const report = structuredClone(sourceReport),
    outputPackages = new Map(report.packages.map((p) => [key(p), p]));
  const chunks = [base.text];
  let bytes = Buffer.byteLength(base.text);
  for (const [identity, source] of sources) {
    const entry = packages.get(identity),
      proof = proofs.get(identity),
      p = source.provenance;
    check(
      entry?.materialStatus === "missing" &&
        proof &&
        ["ecosystem", "name", "version", "license", "manifestSha256"].every(
          (k) => entry[k] === source[k] && proof[k] === source[k],
        ),
      "补充身份／版本／许可／清单与原包或证明不符",
    );
    const discriminator =
      source.ecosystem === "cargo" ? "archiveSha256" : "integrity";
    check(
      entry[discriminator] === source[discriminator] &&
        proof[discriminator] === source[discriminator],
      "补充原包摘要不符",
    );
    check(
      p &&
        [
          "cargo-fixed-commit",
          "cargo-license-pointer",
          "npm-fixed-commit",
          "npm-parent-release",
        ].includes(p.kind) &&
        (source.ecosystem === "cargo") === p.kind.startsWith("cargo-"),
      "补充来源类别无效",
    );
    check(
      Array.isArray(source.notices) &&
        source.notices.length > 0 &&
        source.notices.length <= 16 &&
        new Set(source.notices.map((n) => n.file)).size ===
          source.notices.length,
      "单包补充原文预算或重复拒绝",
    );
    const prefix = verifyIdentity(assets, source, proof, packages);
    const output = outputPackages.get(identity);
    output.originalMaterialStatus = output.materialStatus;
    output.notices = source.notices.map((descriptor) => {
      if (prefix)
        check(
          descriptor.url === prefix + descriptor.sourcePath.split("#")[0] ||
            (p.kind === "cargo-license-pointer" &&
              descriptor.url ===
                "https://www.mozilla.org/media/MPL/2.0/index.txt"),
          "补充原文URL不属于绑定来源",
        );
      const notice = verifiedOriginal(assets, descriptor),
        part = `\n===== ${source.name}@${source.version} / ${descriptor.url} =====\n${notice.text}\n`;
      bytes += Buffer.byteLength(part);
      check(bytes <= 16 * 1024 * 1024, "补充合并全文预算超限");
      chunks.push(part);
      return { ...descriptor, origin: p.kind };
    });
    output.materialStatus = "collected";
    output.supplementalProvenance = p;
    output.supplementVerification =
      source.ecosystem === "cargo"
        ? {
            currentLockedArchiveVerified: true,
            archiveFilesCompared: proof.archiveFilesCompared,
          }
        : {
            currentInstalledManifestVerified: true,
            archiveIntegrityVerifiedAtAcquisition: true,
            currentArchiveRechecked: false,
          };
  }
  const text = chunks.join("");
  report.textSha256 = noticeHash(Buffer.from(text));
  report.textBytes = bytes;
  report.missingPackages = report.packages
    .filter((p) => p.materialStatus !== "collected")
    .map((p) => ({
      ecosystem: p.ecosystem,
      name: p.name ?? p.path,
      version: p.version,
      status: p.materialStatus,
    }));
  report.supplementalMaterials = {
    catalogSha256: catalogText.sha256,
    sourceCount: sources.size,
    allNpmArchivesQualified: false,
  };
  return { report, text };
}
