import {
  noticeHash,
  noticePath,
  noticeText,
} from "../previs/signalling-notices-files.mjs";
import { readJson } from "../previs/signalling-package-files.mjs";

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
const repository = (value) =>
  typeof value === "string" && /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(value);
export function verifiedOriginal(assets, descriptor) {
  check(
    descriptor &&
      digest(descriptor.sha256) &&
      Number.isSafeInteger(descriptor.bytes) &&
      descriptor.bytes > 0,
    "补充原文描述无效",
  );
  const notice = noticeText(assets, descriptor.file);
  check(
    notice.sha256 === descriptor.sha256 && notice.bytes === descriptor.bytes,
    "补充原文摘要或大小不匹配",
  );
  return notice;
}
function pinnedPrefix(provenance) {
  check(
    repository(provenance.repository) &&
      repository(provenance.remoteRepository ?? provenance.repository) &&
      /^[a-f0-9]{40}$/.test(provenance.commit),
    "补充来源不是固定提交",
  );
  return `https://raw.githubusercontent.com/${provenance.remoteRepository ?? provenance.repository}/${provenance.commit}/`;
}
function redirectIdentity(assets, provenance) {
  if (
    provenance.remoteRepository === provenance.repository ||
    !provenance.remoteRepository
  )
    return;
  const redirect = provenance.repositoryRedirect;
  check(
    redirect?.status === 301 &&
      redirect.url ===
        `https://api.github.com/repos/${provenance.repository}/git/trees/${provenance.commit}?recursive=1`,
    "仓库迁移不是原固定提交的官方重定向",
  );
  verifiedOriginal(assets, redirect.canonical);
  const metadata = readJson(noticePath(assets, redirect.canonical.file));
  check(
    Number.isSafeInteger(metadata.id) &&
      metadata.id > 0 &&
      metadata.full_name === provenance.remoteRepository &&
      redirect.canonical.url ===
        `https://api.github.com/repositories/${metadata.id}` &&
      redirect.location ===
        `https://api.github.com/repositories/${metadata.id}/git/trees/${provenance.commit}?recursive=1`,
    "仓库迁移身份不匹配",
  );
}
function cargoIdentity(assets, source, proof) {
  const p = source.provenance,
    prefix = pinnedPrefix(p);
  redirectIdentity(assets, p);
  check(
    proof.archiveSha256 === source.archiveSha256 &&
      proof.repository === p.repository &&
      proof.commit === p.commit &&
      proof.cratePath === p.cratePath &&
      Number.isSafeInteger(proof.archiveFilesCompared) &&
      proof.archiveFilesCompared >= 3,
    "Rust原包／固定提交／目录证明不匹配",
  );
  check(
    Array.isArray(p.sourceFiles) &&
      p.sourceFiles.length >= 3 &&
      p.sourceFiles.length <= 17 &&
      new Set(p.sourceFiles.map((f) => f.file)).size === p.sourceFiles.length,
    "Rust原文件预算或身份无效",
  );
  for (const file of p.sourceFiles)
    check(
      digest(file.sha256) && proof.fileHashes?.[file.file] === file.sha256,
      "Rust原文件未锁定核对",
    );
  check(
    p.sourceFiles.some(
      (f) => f.file === "Cargo.toml" && f.sha256 === source.manifestSha256,
    ) && p.sourceFiles.some((f) => f.file === ".cargo_vcs_info.json"),
    "Rust清单或VCS证明缺失",
  );
  const origin = p.sourceFiles.find((f) => f.file === "Cargo.toml.orig");
  check(
    origin &&
      p.upstreamManifest?.sha256 === origin.sha256 &&
      p.upstreamManifest.url === prefix + p.upstreamManifestPath,
    "固定提交清单与原发布清单不相同",
  );
  verifiedOriginal(assets, p.upstreamManifest);
  if (p.kind === "cargo-license-pointer") {
    const pointer = p.licensePointer;
    check(
      source.license === "MPL-2.0" &&
        pointer?.url === "https://mozilla.org/MPL/2.0/" &&
        proof.fileHashes?.[pointer.file] === pointer.sha256 &&
        proof.licenseHeaderSha256 === pointer.headerSha256,
      "源码许可指针未由锁定原文件证明",
    );
    const upstream = verifiedOriginal(assets, pointer.upstreamSource);
    check(
      upstream.sha256 === pointer.sha256 &&
        pointer.upstreamSource.url ===
          prefix + (p.cratePath ? p.cratePath + "/" : "") + pointer.file,
      "固定源码摘要或来源不匹配",
    );
    const header = Buffer.from(upstream.text).subarray(0, pointer.headerBytes);
    check(
      noticeHash(header) === pointer.headerSha256 &&
        header.toString("utf8").includes(pointer.url),
      "源码告知头不匹配",
    );
  }
  return prefix;
}
function npmMetadata(assets, descriptor, entry) {
  verifiedOriginal(assets, descriptor);
  const metadata = readJson(noticePath(assets, descriptor.file));
  check(
    descriptor.url ===
      `https://registry.npmjs.org/${encodeURIComponent(entry.name)}/${entry.version}` &&
      metadata.name === entry.name &&
      metadata.version === entry.version &&
      metadata.license === entry.license &&
      metadata.dist?.integrity === entry.integrity &&
      metadata.dist?.tarball === entry.resolved &&
      /^[a-f0-9]{40}$/.test(metadata.gitHead),
    "npm固定版本元数据不匹配",
  );
  return metadata;
}
function archiveDigest(hex, integrity) {
  check(
    typeof hex === "string" &&
      /^[a-f0-9]{128}$/.test(hex) &&
      "sha512-" + Buffer.from(hex, "hex").toString("base64") === integrity,
    "npm取得时原包摘要与锁不符",
  );
}
function npmIdentity(assets, source, proof, packages) {
  const p = source.provenance,
    entry = packages.get(key(source)),
    metadata = npmMetadata(assets, p.metadata, entry);
  archiveDigest(p.targetArchiveSha512, entry.integrity);
  check(
    p.archiveIntegrityVerifiedAtAcquisition === true &&
      p.releaseManifest.sha256 === proof.manifestSha256,
    "npm原发布清单证明不匹配",
  );
  verifiedOriginal(assets, p.releaseManifest);
  if (p.kind === "npm-fixed-commit") {
    const prefix = pinnedPrefix(p);
    check(
      metadata.gitHead === p.commit &&
        p.upstreamManifest.sha256 === proof.manifestSha256 &&
        p.upstreamManifest.url === prefix + p.upstreamManifestPath,
      "npm上游清单或提交不匹配",
    );
    verifiedOriginal(assets, p.upstreamManifest);
    return prefix;
  }
  const parent = p.parent,
    parentEntry = packages.get(key({ ...parent, ecosystem: "npm" }));
  check(
    parentEntry &&
      [
        "name",
        "version",
        "license",
        "path",
        "integrity",
        "manifestSha256",
      ].every((k) => parent[k] === parentEntry[k]),
    "npm发布父包不在同一锁定材料集",
  );
  const parentMetadata = npmMetadata(assets, parent.metadata, parentEntry);
  check(
    parentMetadata.gitHead === metadata.gitHead &&
      parent.license === source.license &&
      proof.parent?.manifestSha256 === parent.manifestSha256 &&
      proof.parent.optionalTargetVersion === source.version,
    "npm父子发布版本或提交关系不匹配",
  );
  archiveDigest(parent.archiveSha512, parent.integrity);
  verifiedOriginal(assets, parent.releaseManifest);
  check(
    parent.releaseManifest.sha256 === parent.manifestSha256,
    "npm父包原发布清单不匹配",
  );
  const manifest = readJson(noticePath(assets, parent.releaseManifest.file));
  check(
    manifest.optionalDependencies?.[source.name] === source.version,
    "npm父包未声明确切子包",
  );
  for (const notice of source.notices)
    check(
      notice.sha256 === proof.parent.noticeSha256 &&
        notice.url === parentEntry.resolved + "#package/LICENSE.md",
      "npm父包原文不匹配",
    );
  return null;
}
export function verifyIdentity(assets, source, proof, packages) {
  return source.ecosystem === "cargo"
    ? cargoIdentity(assets, source, proof)
    : npmIdentity(assets, source, proof, packages);
}
