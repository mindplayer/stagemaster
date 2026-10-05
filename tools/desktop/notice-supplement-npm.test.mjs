import assert from "node:assert/strict";
import { test } from "node:test";
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
import { proveSupplementSources } from "./notice-supplement-proofs.mjs";
const root = fileURLToPath(new URL("../../", import.meta.url));
const json = (value) => JSON.stringify(value);
function fixture(t, parent = false) {
  const directory = mkdtempSync(join(root, "tmp/desktop-npm-supplement-test-")),
    assets = join(directory, "assets"),
    ui = join(directory, "ui");
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  mkdirSync(assets);
  mkdirSync(ui);
  const write = (dir, file, text) => {
    mkdirSync(join(dir, file, ".."), { recursive: true });
    writeFileSync(join(dir, file), text);
  };
  const descriptor = (file, text, url) => {
    write(assets, file, text);
    return {
      file,
      sha256: noticeHash(Buffer.from(text)),
      bytes: Buffer.byteLength(text),
      url,
    };
  };
  const hex = "1".repeat(128),
    integrity = "sha512-" + Buffer.from(hex, "hex").toString("base64"),
    commit = "a".repeat(40);
  const prefix =
      "https://raw.githubusercontent.com/fixture/repo/" + commit + "/",
    name = "@fixture/native",
    version = "1.0.0",
    license = "MIT";
  const manifest = json({ name, version, license }),
    text = "Original materials\n",
    notice = "MIT fixture original\n";
  const entry = {
    ecosystem: "npm",
    name,
    version,
    license,
    path: "node_modules/" + name,
    integrity,
    resolved: "https://registry.npmjs.org/@fixture/native/-/native-1.0.0.tgz",
    manifestSha256: noticeHash(Buffer.from(manifest)),
    materialStatus: "missing",
    archiveIntegrityVerified: false,
    notices: [],
  };
  write(ui, entry.path + "/package.json", manifest);
  const metadata = {
    name,
    version,
    license,
    gitHead: commit,
    dist: { integrity, tarball: entry.resolved },
  };
  const provenance = {
    kind: parent ? "npm-parent-release" : "npm-fixed-commit",
    metadata: descriptor(
      "metadata/target.json",
      json(metadata),
      "https://registry.npmjs.org/" + encodeURIComponent(name) + "/" + version,
    ),
    releaseManifest: descriptor("manifests/target.json", manifest),
    targetArchiveSha512: hex,
    archiveIntegrityVerifiedAtAcquisition: true,
  };
  const packages = [entry];
  let parentEntry, parentManifest, parentMetadata;
  if (!parent) {
    Object.assign(provenance, {
      repository: "fixture/repo",
      commit,
      upstreamManifestPath: "package.json",
      upstreamManifest: descriptor(
        "manifests/upstream.json",
        manifest,
        prefix + "package.json",
      ),
    });
  } else {
    parentManifest = {
      name: "publisher",
      version,
      license,
      optionalDependencies: { [name]: version },
    };
    parentEntry = {
      ...entry,
      name: "publisher",
      path: "node_modules/publisher",
      materialStatus: "collected",
      resolved: "https://registry.npmjs.org/publisher/-/publisher-1.0.0.tgz",
      manifestSha256: noticeHash(Buffer.from(json(parentManifest))),
    };
    packages.push(parentEntry);
    write(ui, parentEntry.path + "/package.json", json(parentManifest));
    write(ui, parentEntry.path + "/LICENSE.md", notice);
    parentMetadata = {
      name: parentEntry.name,
      version,
      license,
      gitHead: commit,
      dist: { integrity, tarball: parentEntry.resolved },
    };
    provenance.parent = {
      name: parentEntry.name,
      version,
      license,
      path: parentEntry.path,
      integrity,
      manifestSha256: parentEntry.manifestSha256,
      metadata: descriptor(
        "metadata/parent.json",
        json(parentMetadata),
        "https://registry.npmjs.org/publisher/1.0.0",
      ),
      releaseManifest: descriptor(
        "manifests/parent.json",
        json(parentManifest),
      ),
      archiveSha512: hex,
    };
  }
  const source = {
    ecosystem: "npm",
    name,
    version,
    license,
    manifestSha256: entry.manifestSha256,
    integrity,
    provenance,
    notices: [
      {
        ...descriptor(
          "texts/license.txt",
          notice,
          parent
            ? parentEntry.resolved + "#package/LICENSE.md"
            : prefix + "LICENSE.md",
        ),
        sourcePath: "LICENSE.md",
      },
    ],
  };
  const report = {
    format: "stagemaster.desktop-notice-materials",
    version: 1,
    packages,
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
      reportSha256: noticeHash(Buffer.from(json(report))),
    },
    catalog = {
      format: "stagemaster.desktop-notice-supplements",
      version: 1,
      baseReportSha256: base.reportSha256,
      baseTextSha256: report.textSha256,
      entries: [source],
    };
  const update = () => write(assets, "sources.json", json(catalog));
  update();
  const run = () =>
    supplementNotices(
      base,
      assets,
      proveSupplementSources(report, assets, { uiDirectory: ui }),
    );
  return {
    assets,
    ui,
    base,
    source,
    metadata,
    parentMetadata,
    parentManifest,
    parentEntry,
    update,
    descriptor,
    write,
    run,
  };
}
test("npm固定提交及同版本父包材料均核对当前安装，不冒作全部归档复验", (t) => {
  for (const parent of [false, true]) {
    const f = fixture(t, parent),
      result = f.run(),
      entry = result.report.packages[0];
    assert.equal(entry.materialStatus, "collected");
    assert.equal(entry.archiveIntegrityVerified, false);
    assert.equal(
      entry.supplementVerification.currentInstalledManifestVerified,
      true,
    );
    assert.equal(entry.supplementVerification.currentArchiveRechecked, false);
    assert.equal(
      result.report.supplementalMaterials.allNpmArchivesQualified,
      false,
    );
    assert.deepEqual(f.run(), result);
  }
});
test("npm注册元数据版本、许可、完整性、原清单或提交变化拒绝", (t) => {
  for (const mutate of [
    (m) => {
      m.version = "2.0.0";
    },
    (m) => {
      m.license = "other";
    },
    (m) => {
      m.dist.integrity = "sha512-wrong";
    },
    (m) => {
      m.gitHead = "main";
    },
  ]) {
    const f = fixture(t);
    mutate(f.metadata);
    f.source.provenance.metadata = f.descriptor(
      "metadata/target.json",
      json(f.metadata),
      f.source.provenance.metadata.url,
    );
    f.update();
    assert.throws(f.run);
  }
  const manifest = fixture(t);
  manifest.write(
    manifest.ui,
    "node_modules/@fixture/native/package.json",
    json({ name: "other", version: "1.0.0", license: "MIT" }),
  );
  assert.throws(manifest.run);
});
test("npm父包的确切子包声明、锁身份、提交和原文缺失或变化拒绝", (t) => {
  const selected = fixture(t, true);
  selected.parentManifest.optionalDependencies["@fixture/native"] = "2.0.0";
  selected.write(
    selected.ui,
    "node_modules/publisher/package.json",
    json(selected.parentManifest),
  );
  assert.throws(selected.run);
  const commit = fixture(t, true);
  commit.parentMetadata.gitHead = "b".repeat(40);
  commit.source.provenance.parent.metadata = commit.descriptor(
    "metadata/parent.json",
    json(commit.parentMetadata),
    commit.source.provenance.parent.metadata.url,
  );
  commit.update();
  assert.throws(commit.run);
  const identity = fixture(t, true);
  identity.source.provenance.parent.integrity = "sha512-wrong";
  identity.update();
  assert.throws(identity.run);
  const text = fixture(t, true);
  text.write(text.ui, "node_modules/publisher/LICENSE.md", "changed original");
  assert.throws(text.run);
});
test("npm当前清单和父包告知链接拒绝", (t) => {
  const f = fixture(t);
  const target = join(f.ui, "node_modules/@fixture/native/package.json");
  rmSync(target);
  symlinkSync(join(f.assets, "manifests/target.json"), target);
  assert.throws(f.run);
  const p = fixture(t, true);
  const license = join(p.ui, "node_modules/publisher/LICENSE.md");
  rmSync(license);
  symlinkSync(join(p.assets, "texts/license.txt"), license);
  assert.throws(p.run);
});
