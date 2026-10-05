import assert from "node:assert/strict";
import { test } from "node:test";
import {
  mkdtempSync,
  writeFileSync,
  rmSync,
  mkdirSync,
  symlinkSync,
  readFileSync as requireBytes,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import {
  noticeHash,
  saveNoticeFiles,
} from "../previs/signalling-notices-files.mjs";
import {
  noticeOutput,
  verifyRustArchives,
} from "./collect-dependency-notices.mjs";
const root = fileURLToPath(new URL("../../", import.meta.url));
function fixture(
  t,
  members = [{ name: "sample-1.0.0/LICENSE", text: "fixture\r\n" }],
) {
  const directory = mkdtempSync(join(root, "tmp/desktop-notice-archive-test-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const result = spawnSync(
    "python3",
    [
      "-c",
      `import io,json,sys,tarfile
b=io.BytesIO()
with tarfile.open(fileobj=b,mode="w:gz") as t:
 for m in json.load(sys.stdin):
  i=tarfile.TarInfo(m["name"])
  if m.get("link"):
   i.type=tarfile.SYMTYPE;i.linkname="outside";t.addfile(i)
  else:
   v=(m["text"]*m.get("repeat",1)).encode();i.size=len(v);t.addfile(i,io.BytesIO(v))
sys.stdout.buffer.write(b.getvalue())`,
    ],
    {
      input: JSON.stringify(members),
      timeout: 6000,
      maxBuffer: 2 * 1024 * 1024,
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
  const archive = join(directory, "sample-1.0.0.crate");
  writeFileSync(archive, result.stdout);
  const source = "registry+https://github.com/rust-lang/crates.io-index";
  const lockText = `version = 4\n[[package]]\nname = "sample"\nversion = "1.0.0"\nsource = "${source}"\nchecksum = "${noticeHash(result.stdout)}"\n`;
  const entries = [
    {
      name: "sample",
      version: "1.0.0",
      source,
      archive,
      files: [
        { file: "LICENSE", sha256: noticeHash(Buffer.from("fixture\r\n")) },
      ],
    },
  ];
  return { directory, archive, lockText, entries };
}
test("原包摘要绑定Cargo锁，逐字节核对许可文件", (t) => {
  const f = fixture(t),
    result = verifyRustArchives(f.lockText, f.entries);
  assert.equal(result[0].filesCompared, 1);
  assert.equal(
    result[0].archiveSha256,
    noticeHash(Buffer.from(requireBytes(f.archive))),
  );
});
test("修改原包、修改缓存文本或缺文本均拒绝", (t) => {
  const f = fixture(t);
  const changed = structuredClone(f.entries);
  changed[0].files[0].sha256 = "b".repeat(64);
  assert.throws(() => verifyRustArchives(f.lockText, changed), /不符/);
  const missing = structuredClone(f.entries);
  missing[0].files[0].file = "NOTICE";
  assert.throws(() => verifyRustArchives(f.lockText, missing), /缺少/);
  writeFileSync(f.archive, "changed archive");
  assert.throws(() => verifyRustArchives(f.lockText, f.entries), /原包摘要/);
});
for (const kind of ["duplicate", "link", "oversized"]) {
  test(`原包${kind}告知拒绝`, (t) => {
    const member = { name: "sample-1.0.0/LICENSE", text: "fixture\r\n" };
    const members =
      kind === "duplicate"
        ? [member, member]
        : [
            {
              ...member,
              ...(kind === "link"
                ? { link: true }
                : { text: "x", repeat: 512 * 1024 + 1 }),
            },
          ];
    const f = fixture(t, members);
    assert.throws(
      () => verifyRustArchives(f.lockText, f.entries),
      /重复、类型或预算/,
    );
  });
}
test("错误锁结构、重复锁身份、原包链接拒绝", (t) => {
  const f = fixture(t);
  assert.throws(
    () => verifyRustArchives("not valid TOML =", f.entries),
    /核对失败/,
  );
  assert.throws(
    () =>
      verifyRustArchives(
        f.lockText + f.lockText.slice(f.lockText.indexOf("[[package]]")),
        f.entries,
      ),
    /重复/,
  );
  const linked = join(f.directory, "linked.crate");
  symlinkSync(f.archive, linked);
  const entries = structuredClone(f.entries);
  entries[0].archive = linked;
  assert.throws(() => verifyRustArchives(f.lockText, entries), /核对失败/);
});
test("输出路径仅限项目data任务目录，不覆盖或经过链接", (t) => {
  for (const path of [
    root,
    "output/release",
    "data/DESKTOP-006",
    "/tmp/outside",
  ])
    assert.throws(() => noticeOutput(path), /仅限/);
  const dir = fixture(t).directory,
    file = join(dir, "licenses");
  mkdirSync(file);
  saveNoticeFiles(dir, { commercialReleaseApproved: false }, "fixture\n");
  assert.throws(() => saveNoticeFiles(dir, {}, "replaced"), /已存在/);
  assert.equal(
    requireBytes(join(file, "THIRD-PARTY-NOTICES.txt"), "utf8"),
    "fixture\n",
  );
  const owned = mkdtempSync(join(root, "data/DESKTOP-006/output-guard-test-"));
  t.after(() => rmSync(owned, { recursive: true, force: true }));
  assert.throws(() => noticeOutput(owned), /已存在/);
  const linked = join(owned, "link");
  symlinkSync(dir, linked);
  assert.throws(() => noticeOutput(join(linked, "new")), /链接/);
});
