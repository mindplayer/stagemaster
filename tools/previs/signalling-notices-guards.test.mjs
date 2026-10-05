import assert from "node:assert/strict";
import { test } from "node:test";
import { existsSync, renameSync, symlinkSync } from "node:fs";
import { join } from "node:path";
import { collectNotices, writeNotices } from "./signalling-notices.mjs";
import { fixture, write } from "./signalling-notices-fixtures.mjs";

for (const [name, bytes] of [
  ["空", Buffer.alloc(0)],
  ["非法UTF-8", Buffer.from([0xff])],
  ["NUL", Buffer.from("fixture\0notice")],
]) {
  test(`原独立告知${name}拒绝，不把存在文件当完整文本`, (t) => {
    const f = fixture(t);
    write(join(f.bundle, "node_modules/a/LICENSE"), bytes);
    assert.throws(() => collectNotices(f.bundle, f.assets), /文本/);
  });
}

test("单份文本超过512KiB拒绝", (t) => {
  const f = fixture(t);
  write(
    join(f.bundle, "node_modules/a/LICENSE"),
    Buffer.alloc(512 * 1024 + 1, 65),
  );
  assert.throws(() => collectNotices(f.bundle, f.assets), /预算/);
});

test("单包独立告知超过16份拒绝", (t) => {
  const f = fixture(t);
  for (let i = 0; i < 17; i++)
    write(join(f.bundle, `node_modules/a/LICENSE.${i}`), "fixture notice");
  assert.throws(() => collectNotices(f.bundle, f.assets), /预算/);
});

test("合并全文超过8MiB拒绝，不在写入后才判断", (t) => {
  const f = fixture(t);
  f.manifest.dependencies.b = "1.0.0";
  f.lock.packages["node_modules/b"] = { ...f.entry };
  write(join(f.bundle, "package.json"), JSON.stringify(f.manifest));
  write(join(f.bundle, "package-lock.json"), JSON.stringify(f.lock));
  write(
    join(f.bundle, "node_modules/b/package.json"),
    JSON.stringify({ name: "b", version: "1.0.0" }),
  );
  for (const name of ["a", "b"])
    for (let i = 0; i < 9; i++)
      write(
        join(f.bundle, `node_modules/${name}/LICENSE.${i}`),
        Buffer.alloc(512 * 1024, 65),
      );
  assert.throws(() => writeNotices(f.bundle, f.assets), /预算/);
  assert.equal(existsSync(join(f.bundle, "licenses/notices.json")), false);
});

test("补充路径穿越拒绝", (t) => {
  const f = fixture(t);
  f.embed();
  f.setCatalog([{ ...f.catalog(), file: "../outside" }]);
  assert.throws(() => collectNotices(f.bundle, f.assets), /路径/);
});

test("包内许可链接越界拒绝", (t) => {
  const f = fixture(t);
  write(join(f.directory, "outside"), "outside");
  symlinkSync(
    join(f.directory, "outside"),
    join(f.bundle, "node_modules/a/LICENSE"),
  );
  assert.throws(() => collectNotices(f.bundle, f.assets), /越界|链接/);
});

test("补充来源链接拒绝，不能用哈希相同解除边界", (t) => {
  const f = fixture(t);
  f.setCatalog([f.catalog("pinned-upstream")]);
  write(join(f.directory, "outside"), f.text);
  symlinkSync(join(f.directory, "outside"), join(f.assets, "epic-LICENSE.md"));
  assert.throws(() => collectNotices(f.bundle, f.assets), /链接/);
});

test("已有全文目标拒绝，不覆盖用户或旧产物", (t) => {
  const f = fixture(t);
  f.embed();
  write(join(f.bundle, "licenses/THIRD-PARTY-NOTICES.txt"), "sentinel");
  assert.throws(() => writeNotices(f.bundle, f.assets), /已存在/);
  assert.equal(existsSync(join(f.bundle, "licenses/notices.json")), false);
});

test("已有索引目标拒绝，不能先写新全文", (t) => {
  const f = fixture(t);
  f.embed();
  write(join(f.bundle, "licenses/notices.json"), "sentinel");
  assert.throws(() => writeNotices(f.bundle, f.assets), /已存在/);
  assert.equal(
    existsSync(join(f.bundle, "licenses/THIRD-PARTY-NOTICES.txt")),
    false,
  );
});

test("输出licenses为链接时写入前拒绝", (t) => {
  const f = fixture(t);
  f.embed();
  renameSync(join(f.bundle, "licenses"), join(f.directory, "owned-licenses"));
  symlinkSync(join(f.directory, "owned-licenses"), join(f.bundle, "licenses"));
  assert.throws(() => writeNotices(f.bundle, f.assets), /链接/);
  assert.equal(
    existsSync(join(f.directory, "owned-licenses/notices.json")),
    false,
  );
});

test("未知来源类别不能标完整", (t) => {
  const f = fixture(t);
  f.embed();
  f.setCatalog([{ ...f.catalog(), origin: "metadata-only" }]);
  assert.throws(() => collectNotices(f.bundle, f.assets), /来源/);
});

test("仅空白不是完整告知文本", (t) => {
  const f = fixture(t);
  write(join(f.bundle, "node_modules/a/LICENSE"), " \r\n\t ");
  assert.throws(() => collectNotices(f.bundle, f.assets), /文本/);
});
