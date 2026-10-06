import assert from "node:assert/strict";
import { test } from "node:test";
import { chmodSync, rmSync, symlinkSync } from "node:fs";
import { join } from "node:path";
import {
  captureSourceEvidence,
  sameSourceEvidence,
} from "./source-evidence.mjs";
import { sourceLimits } from "./source-scope.mjs";
import { sourceFixture } from "./source-evidence-fixture.mjs";

test("真实 Git 干净输入有稳定内容指纹，时间不伪装源码版本", async (t) => {
  const f = sourceFixture(t),
    first = await captureSourceEvidence(f.root),
    next = await captureSourceEvidence(f.root);
  assert.equal(first.git.head, f.git(["rev-parse", "HEAD"]));
  assert.equal(first.git.dirty, false);
  sameSourceEvidence(first, next);
  assert.ok(
    first.inputs.some((row) => row.path === "apps/ui-prototype/src/界面.ts"),
  );
  assert.ok(first.inputs.every((row) => row.path[0] !== "/"));
});
test("已修改／未提交新增、非 ASCII 与换行路径都完整记录", async (t) => {
  const f = sourceFixture(t),
    first = await captureSourceEvidence(f.root);
  f.write("crates/sample/src/lib.rs", "pub const VALUE: u8 = 2;\n");
  const added = "apps/ui-prototype/src/甲 空间/换行\n文件.ts";
  f.write(added, "new source\n");
  const next = await captureSourceEvidence(f.root);
  assert.equal(next.git.dirty, true);
  assert.deepEqual(
    next.git.changedPaths,
    [added, "crates/sample/src/lib.rs"].sort(),
  );
  assert.throws(() => sameSourceEvidence(first, next), /输入发生变化/);
});
test("删除源码和执行位变化不能登记为纯提交输入", async (t) => {
  const f = sourceFixture(t);
  rmSync(join(f.root, "crates/sample/src/lib.rs"));
  chmodSync(join(f.root, "apps/ui-prototype/src/界面.ts"), 0o755);
  const next = await captureSourceEvidence(f.root);
  assert.equal(next.git.dirty, true);
  assert.deepEqual(
    next.git.changedPaths,
    ["apps/ui-prototype/src/界面.ts", "crates/sample/src/lib.rs"].sort(),
  );
});
test("排除测试／生成／依赖树／output，不遍历故意设坏的链接", async (t) => {
  const f = sourceFixture(t),
    before = await captureSourceEvidence(f.root);
  f.write("apps/ui-prototype/tests/fixture.ts", "test only");
  f.write("crates/sample/src/item_tests.rs", "test only");
  f.write("apps/ui-prototype/dist/client.js", "generated");
  symlinkSync(join(f.root, "output"), join(f.root, "output"));
  symlinkSync(
    join(f.root, "absent"),
    join(f.root, "apps/ui-prototype/node_modules"),
  );
  const next = await captureSourceEvidence(f.root);
  sameSourceEvidence(before, next);
});
test("普通来源之外的链接被明确拒绝，不借目标文件内容", async (t) => {
  const f = sourceFixture(t);
  symlinkSync(
    join(f.root, "Cargo.lock"),
    join(f.root, "apps/ui-prototype/src/linked.ts"),
  );
  await assert.rejects(captureSourceEvidence(f.root), /链接/);
});
for (const [name, value] of [
  ["files", 1],
  ["entries", 1],
  ["fileBytes", 2],
  ["totalBytes", 2],
]) {
  test(`来源预算${name}超过时不省略输入`, async (t) => {
    const f = sourceFixture(t);
    await assert.rejects(
      captureSourceEvidence(f.root, { ...sourceLimits, [name]: value }),
      /超过上限/,
    );
  });
}
test("必要输入丢失及非工作区根目录均拒绝", async (t) => {
  const f = sourceFixture(t);
  rmSync(join(f.root, "Cargo.lock"));
  await assert.rejects(captureSourceEvidence(f.root), /必要输入/);
  await assert.rejects(
    captureSourceEvidence(join(f.root, "apps/ui-prototype")),
    /工作区根目录/,
  );
});
test("只有文档的 Git 基线变化也必须显式重新确认构建", async (t) => {
  const f = sourceFixture(t),
    before = await captureSourceEvidence(f.root);
  f.write("README.md", "doc only\n");
  f.commit();
  const next = await captureSourceEvidence(f.root);
  assert.equal(next.fingerprint, before.fingerprint);
  assert.throws(() => sameSourceEvidence(before, next), /Git 基线/);
});
