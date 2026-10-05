import assert from "node:assert/strict";
import { test } from "node:test";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { noticeHash } from "../previs/signalling-notices-files.mjs";
import { fixture } from "./notice-supplement-fixtures.mjs";
function descriptor(f, file, text, url) {
  writeFileSync(join(f.assets, file), text);
  return {
    file,
    sha256: noticeHash(Buffer.from(text)),
    bytes: Buffer.byteLength(text),
    url,
  };
}
test("源码显式版本指针绑定锁定文件和原告知头，不由许可字符串猜模板", (t) => {
  const f = fixture(t),
    p = f.source.provenance,
    prefix = p.upstreamManifest.url.replace("crates/fixture/Cargo.toml", ""),
    header = "/* Fixture explicit pointer: https://mozilla.org/MPL/2.0/. */",
    code = header + "\nfixture source\n";
  f.base.report.packages[0].license =
    f.source.license =
    f.proof.license =
      "MPL-2.0";
  const original = descriptor(
    f,
    "manifests/original-source.rs",
    code,
    prefix + "crates/fixture/lib.rs",
  );
  p.kind = "cargo-license-pointer";
  p.sourceFiles.push({ file: "lib.rs", sha256: original.sha256 });
  f.proof.fileHashes["lib.rs"] = original.sha256;
  f.proof.archiveFilesCompared = 4;
  f.proof.licenseHeaderSha256 = noticeHash(Buffer.from(header));
  p.licensePointer = {
    file: "lib.rs",
    sha256: original.sha256,
    headerBytes: Buffer.byteLength(header),
    headerSha256: f.proof.licenseHeaderSha256,
    url: "https://mozilla.org/MPL/2.0/",
    upstreamSource: original,
  };
  f.source.notices = [
    {
      ...descriptor(f, "texts/header.txt", header, original.url),
      sourcePath: "crates/fixture/lib.rs#license-header",
    },
    {
      ...descriptor(
        f,
        "texts/versioned-original.txt",
        "Fixture versioned official original\n",
        "https://www.mozilla.org/media/MPL/2.0/index.txt",
      ),
      sourcePath: "MPL/2.0/index.txt",
    },
  ];
  f.update();
  assert.equal(f.run().report.packages[0].notices.length, 2);
  f.proof.licenseHeaderSha256 = "b".repeat(64);
  assert.throws(f.run);
  f.proof.licenseHeaderSha256 = p.licensePointer.headerSha256;
  p.licensePointer.url = "https://mozilla.org/MPL/2.1/";
  f.update();
  assert.throws(f.run);
  p.licensePointer.url = "https://mozilla.org/MPL/2.0/";
  p.licensePointer.upstreamSource.url = "https://example.invalid/source";
  f.update();
  assert.throws(f.run);
});
test("仓库官方迁移保持原提交／清单，重定向身份或目录变化拒绝", (t) => {
  const f = fixture(t),
    p = f.source.provenance,
    metadata = { id: 42, full_name: "fixture/moved" };
  p.remoteRepository = "fixture/moved";
  p.repositoryRedirect = {
    status: 301,
    url: `https://api.github.com/repos/${p.repository}/git/trees/${p.commit}?recursive=1`,
    location: `https://api.github.com/repositories/42/git/trees/${p.commit}?recursive=1`,
    canonical: descriptor(
      f,
      "manifests/redirect.json",
      JSON.stringify(metadata),
      "https://api.github.com/repositories/42",
    ),
  };
  p.upstreamManifest.url = p.upstreamManifest.url.replace(
    "fixture/repo/",
    "fixture/moved/",
  );
  f.source.notices[0].url = f.source.notices[0].url.replace(
    "fixture/repo/",
    "fixture/moved/",
  );
  f.update();
  assert.equal(f.run().report.packages[0].materialStatus, "collected");
  p.repositoryRedirect.location = p.repositoryRedirect.location.replace(
    p.commit,
    "b".repeat(40),
  );
  f.update();
  assert.throws(f.run);
  p.repositoryRedirect.location = `https://api.github.com/repositories/42/git/trees/${p.commit}?recursive=1`;
  p.repositoryRedirect.canonical = descriptor(
    f,
    "manifests/redirect.json",
    JSON.stringify({ id: 42, full_name: "other/repo" }),
    "https://api.github.com/repositories/42",
  );
  f.update();
  assert.throws(f.run);
});
