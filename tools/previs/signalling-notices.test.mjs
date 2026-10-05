import assert from "node:assert/strict";
import { test } from "node:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { collectNotices, writeNotices } from "./signalling-notices.mjs";
import { fixture, hash, write } from "./signalling-notices-fixtures.mjs";
import {
  lockedPackages,
  packageLicenses,
} from "./signalling-package-files.mjs";

test("原独立许可原字节与运行时告知可查阅，不变成商业发行批准", (t) => {
  const f = fixture(t),
    bytes = Buffer.from("fixture copyright\r\npermission\r\n");
  write(join(f.bundle, "node_modules/a/LICENSE"), bytes);
  const { report, text } = collectNotices(f.bundle, f.assets);
  assert.equal(report.packages[0].materialStatus, "complete");
  assert.equal(report.packages[0].notices[0].sha256, hash(bytes));
  assert.equal(report.packages[0].notices[0].origin, "package");
  assert.equal(report.runtimeNotice.sha256, hash("fixture Node notice\n"));
  assert.ok(text.includes(bytes.toString()));
  assert.equal(report.commercialReleaseApproved, false);
  assert.equal(report.reviewStillRequired, true);
});

test("准确绑定的README嵌入告知补材料，旧无独立文件事实保持", (t) => {
  const f = fixture(t);
  f.embed();
  const { report, text } = collectNotices(f.bundle, f.assets);
  assert.equal(report.packages[0].materialStatus, "complete");
  assert.equal(report.packages[0].originalNeedsLicenseReview, true);
  assert.equal(report.packages[0].notices[0].origin, "embedded-package");
  assert.ok(text.includes(f.text));
  assert.equal(
    packageLicenses(f.bundle, lockedPackages(f.manifest, f.lock))[0]
      .needsLicenseReview,
    true,
  );
});

test("补充上游文本绑定固定提交、实际版本与锁完整性", (t) => {
  const f = fixture(t);
  f.upstream();
  const { report, text } = collectNotices(f.bundle, f.assets),
    notice = report.packages[0].notices[0];
  assert.equal(report.packages[0].materialStatus, "complete");
  assert.equal(notice.origin, "pinned-upstream");
  assert.equal(notice.gitHead, "a".repeat(40));
  assert.equal(notice.sha256, hash(f.text));
  assert.ok(text.includes(f.text));
  assert.equal(report.commercialReleaseApproved, false);
});

test("README及MIT元数据不自动补齐未知文本，缺项定位保持", (t) => {
  const f = fixture(t);
  write(
    join(f.bundle, "node_modules/a/README.md"),
    "MIT mentioned but no approved material",
  );
  const { report } = collectNotices(f.bundle, f.assets);
  assert.equal(report.packages[0].materialStatus, "missing");
  assert.deepEqual(report.packages[0].notices, []);
  assert.deepEqual(report.missingPackages, ["node_modules/a"]);
});

test("生成告知确定性、索引指向全文哈希，不修改依赖树", (t) => {
  const f = fixture(t);
  f.embed();
  const before = readFileSync(join(f.bundle, "node_modules/a/Readme.md"));
  assert.deepEqual(
    collectNotices(f.bundle, f.assets),
    collectNotices(f.bundle, f.assets),
  );
  const result = writeNotices(f.bundle, f.assets);
  const text = readFileSync(join(f.bundle, "licenses/THIRD-PARTY-NOTICES.txt"));
  const index = JSON.parse(
    readFileSync(join(f.bundle, "licenses/notices.json"), "utf8"),
  );
  assert.equal(index.textSha256, hash(text));
  assert.deepEqual(index, result.report);
  assert.deepEqual(
    readFileSync(join(f.bundle, "node_modules/a/Readme.md")),
    before,
  );
  assert.equal(index.commercialReleaseApproved, false);
});

for (const [field, value] of [
  ["version", "2.0.0"],
  ["integrity", "sha512-other"],
  ["license", "BSD-2-Clause"],
]) {
  test(`补充材料${field}不符拒绝，不根据名称猜版本`, (t) => {
    const f = fixture(t);
    f.embed();
    f.setCatalog([{ ...f.catalog(), [field]: value }]);
    assert.throws(() => collectNotices(f.bundle, f.assets), /不匹配/);
  });
}

for (const origin of ["embedded-package", "pinned-upstream"]) {
  test(`${origin}文本被改动先拒绝`, (t) => {
    const f = fixture(t);
    origin === "embedded-package" ? f.embed() : f.upstream();
    const file =
      origin === "embedded-package"
        ? join(f.bundle, "node_modules/a/Readme.md")
        : join(f.assets, "epic-LICENSE.md");
    write(file, f.text + "changed");
    assert.throws(() => collectNotices(f.bundle, f.assets), /哈希/);
  });
}

test("重复补充目标拒绝，不能用后一个覆盖前一个", (t) => {
  const f = fixture(t);
  f.embed();
  f.setCatalog([f.catalog(), f.catalog()]);
  assert.throws(() => collectNotices(f.bundle, f.assets), /重复/);
});

test("未锁定的补充目标拒绝，不混入无关许可", (t) => {
  const f = fixture(t);
  f.embed();
  f.setCatalog([{ ...f.catalog(), name: "other" }]);
  assert.throws(() => collectNotices(f.bundle, f.assets), /未锁定/);
});

test("固定上游URL必须匹配完整提交，不允许浮动master", (t) => {
  const f = fixture(t);
  f.upstream();
  f.setCatalog([
    {
      ...f.catalog("pinned-upstream"),
      sourceUrl:
        "https://raw.githubusercontent.com/EpicGames/PixelStreamingInfrastructure/master/LICENSE.md",
    },
  ]);
  assert.throws(() => collectNotices(f.bundle, f.assets), /固定提交/);
});

test("同名嵌套不同版本不能借用已核对版本的告知", (t) => {
  const f = fixture(t);
  f.embed();
  const key = "node_modules/a/node_modules/a";
  f.lock.packages[key] = {
    ...f.entry,
    version: "2.0.0",
    integrity: "sha512-other",
  };
  write(join(f.bundle, "package-lock.json"), JSON.stringify(f.lock));
  write(
    join(f.bundle, key, "package.json"),
    JSON.stringify({ name: "a", version: "2.0.0" }),
  );
  write(join(f.bundle, key, "Readme.md"), f.text);
  const { report } = collectNotices(f.bundle, f.assets);
  assert.equal(
    report.packages.find((entry) => entry.version === "1.0.0").materialStatus,
    "complete",
  );
  assert.equal(
    report.packages.find((entry) => entry.version === "2.0.0").materialStatus,
    "missing",
  );
  assert.deepEqual(report.missingPackages, [key]);
});
