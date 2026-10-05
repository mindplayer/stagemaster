import assert from "node:assert/strict";
import { test } from "node:test";
import { writeFileSync, symlinkSync } from "node:fs";
import { join } from "node:path";
import { noticeHash } from "../previs/signalling-notices-files.mjs";
import { supplementNotices } from "./notice-supplements.mjs";
import { fixture } from "./notice-supplement-fixtures.mjs";

test("固定原文原样补充，历史缺项与未安装保持可解释，不提升商业资格", (t) => {
  const f = fixture(t),
    result = f.run();
  assert.equal(result.report.packages[0].materialStatus, "collected");
  assert.equal(result.report.packages[0].originalMaterialStatus, "missing");
  assert.equal(result.report.packages[1].materialStatus, "not-installed");
  assert.equal(result.report.missingPackages.length, 1);
  assert.ok(result.text.includes(f.notice));
  assert.ok(result.text.startsWith(f.base.text));
  assert.equal(result.report.commercialReleaseApproved, false);
  assert.equal(result.report.reviewStillRequired, true);
  assert.equal(result.report.bundledBinaryInventory, false);
  assert.equal(f.base.report.packages[0].materialStatus, "missing");
  assert.deepEqual(f.run(), result);
});
test("补充身份、版本、许可、清单和原包摘要不匹配拒绝", (t) => {
  for (const [key, value] of [
    ["name", "other"],
    ["version", "2.0.0"],
    ["license", "Apache-2.0"],
    ["manifestSha256", "4".repeat(64)],
    ["archiveSha256", "4".repeat(64)],
  ]) {
    const f = fixture(t);
    f.source[key] = value;
    f.update();
    assert.throws(f.run);
  }
});
test("未锁定原文件、提交、目录或缺证明都拒绝", (t) => {
  const f = fixture(t);
  assert.throws(() => supplementNotices(f.base, f.assets, []));
  for (const key of [
    "archiveSha256",
    "manifestSha256",
    "commit",
    "cratePath",
    "repository",
  ]) {
    const proof = { ...f.proof, [key]: "wrong" };
    assert.throws(() => supplementNotices(f.base, f.assets, [proof]));
  }
});
test("重复补充、重复原包及重复证明拒绝", (t) => {
  const f = fixture(t);
  f.catalog.entries.push(f.source);
  f.update();
  assert.throws(f.run);
  f.catalog.entries.pop();
  f.update();
  assert.throws(() => supplementNotices(f.base, f.assets, [f.proof, f.proof]));
  f.base.report.packages.push(f.base.report.packages[0]);
  assert.throws(f.run);
});
test("浮动提交、未知类别及非绑定原文URL拒绝", (t) => {
  for (const mutate of [
    (s) => {
      s.provenance.commit = "main";
    },
    (s) => {
      s.provenance.kind = "guessed-template";
    },
    (s) => {
      s.notices[0].url =
        "https://raw.githubusercontent.com/other/repo/" +
        "a".repeat(40) +
        "/LICENSE";
    },
  ]) {
    const f = fixture(t);
    mutate(f.source);
    f.update();
    assert.throws(f.run);
  }
});
test("补充路径穿越、链接与原文篡改拒绝", (t) => {
  const f = fixture(t);
  f.source.notices[0].file = "../outside";
  f.update();
  assert.throws(f.run);
  const link = fixture(t);
  symlinkSync(
    join(link.assets, "texts/license.txt"),
    join(link.assets, "texts/link.txt"),
  );
  link.source.notices[0].file = "texts/link.txt";
  link.update();
  assert.throws(link.run);
  const changed = fixture(t);
  writeFileSync(join(changed.assets, "texts/license.txt"), "changed");
  assert.throws(changed.run);
});
test("原文UTF8、NUL、空白和单份预算仍拒绝", (t) => {
  for (const bytes of [
    Buffer.from([0xff]),
    Buffer.from("bad\0text"),
    Buffer.from(" \n"),
    Buffer.alloc(512 * 1024 + 1, 65),
  ]) {
    const f = fixture(t);
    writeFileSync(join(f.assets, "texts/license.txt"), bytes);
    f.source.notices[0].sha256 = noticeHash(bytes);
    f.source.notices[0].bytes = bytes.length;
    f.update();
    assert.throws(f.run);
  }
});
test("来源报告或原文变化、错误批准标志及合并超限均拒绝", (t) => {
  const f = fixture(t);
  f.base.reportSha256 = "4".repeat(64);
  assert.throws(f.run);
  const b = fixture(t);
  b.base.text = "changed";
  assert.throws(b.run);
  for (const [key, value] of [
    ["commercialReleaseApproved", true],
    ["reviewStillRequired", false],
    ["bundledBinaryInventory", true],
  ]) {
    const v = fixture(t);
    v.base.report[key] = value;
    assert.throws(v.run);
  }
  const big = fixture(t);
  big.base.text = "x".repeat(16 * 1024 * 1024);
  big.base.report.textSha256 = noticeHash(Buffer.from(big.base.text));
  big.base.report.textBytes = Buffer.byteLength(big.base.text);
  big.catalog.baseTextSha256 = big.base.report.textSha256;
  big.update();
  assert.throws(big.run);
});
