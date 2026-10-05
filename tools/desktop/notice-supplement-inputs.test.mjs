import assert from "node:assert/strict";
import { test } from "node:test";
import {
  mkdtempSync,
  mkdirSync,
  writeFileSync,
  rmSync,
  readFileSync,
  symlinkSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { noticeHash } from "../previs/signalling-notices-files.mjs";
import { proveSupplementSources } from "./notice-supplement-proofs.mjs";
import {
  readBaseMaterialText,
  supplementOutput,
} from "./supplement-dependency-notices.mjs";
const root = fileURLToPath(new URL("../../", import.meta.url));
function directory(t) {
  const dir = mkdtempSync(join(root, "tmp/desktop-supplement-input-test-"));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  return dir;
}
test("原材料合并全文允许既有3MiB而单原文仍512KiB，16MiB封顶", (t) => {
  const dir = directory(t);
  mkdirSync(join(dir, "licenses"));
  const file = join(dir, "licenses/THIRD-PARTY-NOTICES.txt");
  const text = "x".repeat(3 * 1024 * 1024);
  writeFileSync(file, text);
  assert.equal(readBaseMaterialText(dir), text);
  for (const bytes of [
    Buffer.alloc(16 * 1024 * 1024 + 1, 65),
    Buffer.from([0xff]),
    Buffer.from("text\0invalid"),
    Buffer.from(" \n"),
  ]) {
    writeFileSync(file, bytes);
    assert.throws(() => readBaseMaterialText(dir));
  }
  rmSync(file);
  writeFileSync(join(dir, "original"), text);
  symlinkSync(join(dir, "original"), file);
  assert.throws(() => readBaseMaterialText(dir));
});
test("补充输出拒绝项目根、用户output、外部、旧目标和经过链接", (t) => {
  for (const path of [
    root,
    "output/release",
    "data/DESKTOP-009",
    "/tmp/outside",
  ])
    assert.throws(() => supplementOutput(path));
  const owned = mkdtempSync(join(root, "data/DESKTOP-009/output-guard-test-"));
  t.after(() => rmSync(owned, { recursive: true, force: true }));
  assert.throws(() => supplementOutput(owned));
  symlinkSync(directory(t), join(owned, "link"));
  assert.throws(() => supplementOutput(join(owned, "link/new")));
});
function cargo(t) {
  const dir = directory(t),
    cargoHome = join(dir, "cargo"),
    assets = join(dir, "assets"),
    uiDirectory = join(dir, "ui"),
    registryName = "fixture-index";
  const src = join(cargoHome, "registry/src", registryName, "sample-1.0.0"),
    cache = join(cargoHome, "registry/cache", registryName);
  for (const d of [src, cache, assets, uiDirectory])
    mkdirSync(d, { recursive: true });
  const commit = "a".repeat(40),
    contents = {
      "Cargo.toml": 'repository = "https://github.com/fixture/repo"\n',
      "Cargo.toml.orig": "fixture original\n",
      ".cargo_vcs_info.json": JSON.stringify({
        git: { sha1: commit },
        path_in_vcs: "crates/sample",
      }),
    };
  const files = Object.entries(contents).map(([file, text]) => {
    writeFileSync(join(src, file), text);
    return { file, sha256: noticeHash(Buffer.from(text)) };
  });
  const result = spawnSync(
    "python3",
    [
      "-c",
      `import io,json,sys,tarfile
b=io.BytesIO()
with tarfile.open(fileobj=b,mode="w:gz") as t:
 for k,v in json.load(sys.stdin).items():
  data=v.encode();m=tarfile.TarInfo("sample-1.0.0/"+k);m.size=len(data);t.addfile(m,io.BytesIO(data))
sys.stdout.buffer.write(b.getvalue())`,
    ],
    {
      input: JSON.stringify(contents),
      timeout: 6000,
      maxBuffer: 1024 * 1024,
      env: {
        ...process.env,
        TMPDIR: join(root, "tmp"),
        PYTHONDONTWRITEBYTECODE: "1",
      },
    },
  );
  assert.equal(
    result.status,
    0,
    JSON.stringify({
      error: result.error?.code,
      signal: result.signal,
      stderr: result.stderr.toString(),
    }),
  );
  const archive = join(cache, "sample-1.0.0.crate");
  writeFileSync(archive, result.stdout);
  const source = "registry+https://github.com/rust-lang/crates.io-index",
    archiveSha256 = noticeHash(result.stdout),
    lockText = `version = 4\n[[package]]\nname = "sample"\nversion = "1.0.0"\nsource = "${source}"\nchecksum = "${archiveSha256}"\n`;
  const entry = {
    ecosystem: "cargo",
    name: "sample",
    version: "1.0.0",
    source,
    license: "MIT",
    materialStatus: "missing",
    manifestSha256: files[0].sha256,
  };
  const catalog = {
    entries: [
      {
        ...entry,
        provenance: {
          kind: "cargo-fixed-commit",
          cratePath: "crates/sample",
          sourceFiles: files,
          upstreamManifest: { sha256: files[1].sha256 },
        },
      },
    ],
  };
  writeFileSync(join(assets, "sources.json"), JSON.stringify(catalog));
  const run = () =>
    proveSupplementSources({ packages: [entry] }, assets, {
      cargoHome,
      uiDirectory,
      registryName,
      lockText,
    });
  return { src, assets, catalog, archive, archiveSha256, run };
}
test("当前Rust清单、原始清单和VCS逐字节匹配锁定原包", (t) => {
  const f = cargo(t),
    proof = f.run()[0];
  assert.equal(proof.archiveSha256, f.archiveSha256);
  assert.equal(proof.archiveFilesCompared, 3);
  assert.equal(proof.repository, "fixture/repo");
  assert.equal(proof.cratePath, "crates/sample");
  const changed = JSON.stringify({
    git: { sha1: "b".repeat(40) },
    path_in_vcs: "crates/sample",
  });
  writeFileSync(join(f.src, ".cargo_vcs_info.json"), changed);
  assert.throws(f.run);
  f.catalog.entries[0].provenance.sourceFiles.find(
    (x) => x.file === ".cargo_vcs_info.json",
  ).sha256 = noticeHash(Buffer.from(changed));
  writeFileSync(join(f.assets, "sources.json"), JSON.stringify(f.catalog));
  assert.throws(f.run, /缓存文本与锁定原包不符/);
});
test("Rust原包修改或链接拒绝，不借静态索引宣称核验", (t) => {
  const changed = cargo(t);
  writeFileSync(changed.archive, "changed archive");
  assert.throws(changed.run, /原包摘要/);
  const linked = cargo(t),
    bytes = readFileSync(linked.archive);
  rmSync(linked.archive);
  writeFileSync(join(linked.assets, "original.crate"), bytes);
  symlinkSync(join(linked.assets, "original.crate"), linked.archive);
  assert.throws(linked.run, /链接/);
});
