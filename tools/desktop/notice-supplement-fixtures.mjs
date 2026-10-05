import assert from "node:assert/strict";
import {
  mkdtempSync,
  mkdirSync,
  writeFileSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { noticeHash } from "../previs/signalling-notices-files.mjs";
import { supplementNotices } from "./notice-supplements.mjs";
const root = fileURLToPath(new URL("../../", import.meta.url));
export function fixture(t) {
  const assets = mkdtempSync(join(root, "tmp/desktop-supplements-test-"));
  t.after(() => rmSync(assets, { recursive: true, force: true }));
  mkdirSync(join(assets, "texts"));
  mkdirSync(join(assets, "manifests"));
  const text = "Fixture source material\n",
    original = "fixture original manifest\n",
    notice = "Copyright fixture\r\nMIT fixture original\r\n";
  writeFileSync(join(assets, "manifests/original.toml"), original);
  writeFileSync(join(assets, "texts/license.txt"), notice);
  const entry = {
    ecosystem: "cargo",
    name: "fixture",
    version: "1.0.0",
    license: "MIT",
    source: "registry+https://github.com/rust-lang/crates.io-index",
    manifestSha256: "1".repeat(64),
    archiveSha256: "3".repeat(64),
    materialStatus: "missing",
    notices: [],
  };
  const report = {
    format: "stagemaster.desktop-notice-materials",
    version: 1,
    packages: [
      entry,
      {
        ecosystem: "npm",
        path: "node_modules/optional",
        version: "1.0.0",
        materialStatus: "not-installed",
        optional: true,
        notices: [],
      },
    ],
    missingPackages: [],
    textFile: "licenses/THIRD-PARTY-NOTICES.txt",
    textSha256: noticeHash(Buffer.from(text)),
    textBytes: Buffer.byteLength(text),
    commercialReleaseApproved: false,
    reviewStillRequired: true,
    bundledBinaryInventory: false,
  };
  const base = {
    report,
    text,
    reportSha256: noticeHash(Buffer.from(JSON.stringify(report))),
  };
  const commit = "a".repeat(40),
    url = "https://raw.githubusercontent.com/fixture/repo/" + commit + "/";
  const source = {
    ecosystem: "cargo",
    name: "fixture",
    version: "1.0.0",
    license: "MIT",
    manifestSha256: entry.manifestSha256,
    archiveSha256: entry.archiveSha256,
    provenance: {
      kind: "cargo-fixed-commit",
      repository: "fixture/repo",
      remoteRepository: "fixture/repo",
      commit,
      cratePath: "crates/fixture",
      sourceFiles: [
        { file: "Cargo.toml", sha256: entry.manifestSha256 },
        { file: "Cargo.toml.orig", sha256: noticeHash(Buffer.from(original)) },
        { file: ".cargo_vcs_info.json", sha256: "2".repeat(64) },
      ],
      upstreamManifestPath: "crates/fixture/Cargo.toml",
      upstreamManifest: {
        file: "manifests/original.toml",
        sha256: noticeHash(Buffer.from(original)),
        bytes: Buffer.byteLength(original),
        url: url + "crates/fixture/Cargo.toml",
      },
    },
    notices: [
      {
        file: "texts/license.txt",
        sha256: noticeHash(Buffer.from(notice)),
        bytes: Buffer.byteLength(notice),
        url: url + "LICENSE-MIT",
        sourcePath: "LICENSE-MIT",
      },
    ],
  };
  const catalog = {
    format: "stagemaster.desktop-notice-supplements",
    version: 1,
    baseReportSha256: base.reportSha256,
    baseTextSha256: report.textSha256,
    entries: [source],
  };
  const proof = {
    ecosystem: entry.ecosystem,
    name: entry.name,
    version: entry.version,
    license: entry.license,
    manifestSha256: entry.manifestSha256,
    archiveSha256: entry.archiveSha256,
    repository: "fixture/repo",
    commit,
    cratePath: "crates/fixture",
    fileHashes: Object.fromEntries(
      source.provenance.sourceFiles.map((f) => [f.file, f.sha256]),
    ),
    archiveFilesCompared: 3,
  };
  const update = () =>
    writeFileSync(join(assets, "sources.json"), JSON.stringify(catalog));
  update();
  return {
    assets,
    base,
    source,
    catalog,
    proof,
    notice,
    update,
    run: () => supplementNotices(base, assets, [proof]),
  };
}
