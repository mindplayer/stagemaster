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
import {
  collectUiNotices,
  rustClosure,
  collectRustNotices,
} from "./dependency-notices.mjs";
import { noticeHash } from "../previs/signalling-notices-files.mjs";
const root = fileURLToPath(new URL("../../", import.meta.url));
function fixture(t) {
  const dir = mkdtempSync(join(root, "tmp/desktop-notices-test-"));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  return dir;
}
const json = (file, value) => writeFileSync(file, JSON.stringify(value));
function ui(dir, installed = true) {
  const dependency = {
    version: "1.0.0",
    license: "MIT",
    integrity: "sha512-test",
    resolved: "https://registry.npmjs.org/sample/-/sample-1.0.0.tgz",
  };
  const manifest = { dependencies: { sample: "1.0.0" } };
  json(join(dir, "package.json"), manifest);
  json(join(dir, "package-lock.json"), {
    lockfileVersion: 3,
    packages: { "": manifest, "node_modules/sample": dependency },
  });
  if (installed) {
    mkdirSync(join(dir, "node_modules/sample"), { recursive: true });
    json(join(dir, "node_modules/sample/package.json"), {
      name: "sample",
      version: "1.0.0",
      license: "MIT",
    });
  }
  return dependency;
}
function graph() {
  return {
    packages: [
      { id: "desktop", name: "stagemaster-desktop", source: null },
      { id: "host", name: "stagemaster-execution-host", source: null },
      {
        id: "normal",
        name: "sample",
        source: "registry+https://github.com/rust-lang/crates.io-index",
      },
      {
        id: "build",
        name: "builder",
        source: "registry+https://github.com/rust-lang/crates.io-index",
      },
      {
        id: "dev",
        name: "test-only",
        source: "registry+https://github.com/rust-lang/crates.io-index",
      },
    ],
    resolve: {
      nodes: [
        {
          id: "desktop",
          deps: [
            { pkg: "normal", dep_kinds: [{ kind: null }] },
            { pkg: "build", dep_kinds: [{ kind: "build" }] },
            { pkg: "dev", dep_kinds: [{ kind: "dev" }] },
          ],
        },
        {
          id: "host",
          deps: [
            { pkg: "normal", dep_kinds: [{ kind: "dev" }, { kind: null }] },
          ],
        },
        ...["normal", "build", "dev"].map((id) => ({ id, deps: [] })),
      ],
    },
  };
}
test("界面原文和文件摘要原样收集，不把元数据当原文", (t) => {
  const dir = fixture(t);
  ui(dir);
  const text = "Copyright fixture\r\nMIT fixture text\r\n";
  writeFileSync(join(dir, "node_modules/sample/LICENSE-MIT"), text);
  const result = collectUiNotices(dir);
  assert.equal(result.entries.length, 1);
  assert.equal(
    result.entries[0].notices[0].sha256,
    noticeHash(Buffer.from(text)),
  );
  assert.equal(result.chunks[0].text, text);
});
test("未安装与只有许可声明都列缺项，不读取README猜测", (t) => {
  const dir = fixture(t);
  ui(dir, false);
  assert.equal(
    collectUiNotices(dir).entries[0].materialStatus,
    "not-installed",
  );
  ui(dir);
  writeFileSync(join(dir, "node_modules/sample/README.md"), "looks like MIT");
  assert.equal(collectUiNotices(dir).entries[0].materialStatus, "missing");
});
test("安装身份与锁不符拒绝", (t) => {
  const dir = fixture(t);
  ui(dir);
  json(join(dir, "node_modules/sample/package.json"), {
    name: "other",
    version: "1.0.0",
    license: "MIT",
  });
  assert.throws(() => collectUiNotices(dir), /身份/);
});
test("界面许可来源链接拒绝", (t) => {
  const dir = fixture(t);
  ui(dir);
  writeFileSync(join(dir, "original"), "MIT fixture");
  symlinkSync(join(dir, "original"), join(dir, "node_modules/sample/LICENSE"));
  assert.throws(() => collectUiNotices(dir), /链接/);
});
test("Rust闭包包含构建与普通依赖，排除仅dev，混合边保留", () => {
  assert.deepEqual(
    rustClosure(graph())
      .map((p) => p.id)
      .sort(),
    ["build", "desktop", "host", "normal"],
  );
});
test("Rust缺根、缺节点、重复身份及缺闭包成员拒绝", () => {
  const missingRoot = graph();
  missingRoot.packages.shift();
  assert.throws(() => rustClosure(missingRoot), /入口/);
  const noNode = graph();
  noNode.resolve.nodes.pop();
  noNode.resolve.nodes = noNode.resolve.nodes.filter((n) => n.id !== "normal");
  assert.throws(() => rustClosure(noNode), /节点/);
  const duplicate = graph();
  duplicate.packages.push(duplicate.packages[0]);
  assert.throws(() => rustClosure(duplicate), /重复/);
  const missingPackage = graph();
  missingPackage.packages = missingPackage.packages.filter(
    (p) => p.id !== "normal",
  );
  assert.throws(() => rustClosure(missingPackage), /成员/);
});
test("Rust注册包许可原文须匹配缓存校验清单", (t) => {
  const dir = fixture(t),
    registry = join(dir, "registry");
  mkdirSync(registry);
  const text = "MIT fixture\n";
  writeFileSync(join(registry, "LICENSE"), text);
  json(join(registry, "Cargo.toml"), {});
  json(join(registry, ".cargo-checksum.json"), {
    package: "a".repeat(64),
    files: {
      LICENSE: noticeHash(Buffer.from(text)),
      "Cargo.toml": noticeHash(Buffer.from("{}")),
    },
  });
  const metadata = graph();
  for (const p of metadata.packages)
    Object.assign(p, {
      version: "1.0.0",
      license: "MIT",
      license_file: null,
      manifest_path: join(registry, "Cargo.toml"),
    });
  const result = collectRustNotices(metadata, dir);
  assert.equal(result.entries.length, 2);
  assert.equal(result.chunks[0].text, text);
  writeFileSync(join(registry, "LICENSE"), "changed text");
  assert.throws(() => collectRustNotices(metadata, dir), /校验/);
});
test("外部Rust注册包和未支持来源拒绝", (t) => {
  const dir = fixture(t),
    metadata = graph();
  for (const p of metadata.packages)
    Object.assign(p, {
      version: "1.0.0",
      license: "MIT",
      manifest_path: "/tmp/outside/Cargo.toml",
    });
  assert.throws(() => collectRustNotices(metadata, dir), /项目内|范围/);
  metadata.packages.find((p) => p.id === "normal").source =
    "git+https://example.invalid/repo";
  assert.throws(() => collectRustNotices(metadata, dir), /来源/);
});
