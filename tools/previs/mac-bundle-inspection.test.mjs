import assert from "node:assert/strict";
import { test } from "node:test";
import { spawnSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  writeFileSync,
  rmSync,
  symlinkSync,
  realpathSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { inspectMacBundle, readMacImage } from "./mac-bundle-inspection.mjs";
import { withoutLoaderOverrides } from "./packaging-tools.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const dependency = (name, weak = false) => ({
  name,
  weak,
  command: weak ? "LC_LOAD_WEAK_DYLIB" : "LC_LOAD_DYLIB",
});
function setup(t) {
  const directory = mkdtempSync(join(root, "tmp/previs-dependencies-test-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const bundle = join(directory, "移位 舞台/StageMasterPreview.app");
  const program = join(bundle, "Contents/MacOS/StageMasterPreview");
  const metadata = new Map();
  const add = (file, fields = {}) => {
    mkdirSync(dirname(file), { recursive: true });
    writeFileSync(file, "fixture, not executable");
    metadata.set(realpathSync(file), {
      architectures: ["arm64"],
      minimumMacOS: "14.0",
      rpaths: [],
      dependencies: [],
      executable: file === program,
      dylib: file !== program,
      ...fields,
    });
    return file;
  };
  add(program);
  const read = [];
  const check = (limits = {}) =>
    inspectMacBundle(root, program, {
      ...limits,
      readImage: (file) => {
        read.push(file);
        assert.ok(metadata.has(file));
        return metadata.get(file);
      },
    });
  const main = metadata.get(realpathSync(program));
  return { directory, bundle, program, metadata, add, check, main, read };
}

test("间接依赖、继承搜索路径和循环不漏检，系统库不访问开发机", (t) => {
  const s = setup(t);
  const a = s.add(join(s.bundle, "Contents/Libraries/a.dylib"), {
    minimumMacOS: "14.1",
    dependencies: [dependency("@rpath/b.dylib")],
  });
  const b = s.add(join(s.bundle, "Contents/Libraries/b.dylib"), {
    dependencies: [
      dependency("@loader_path/a.dylib"),
      dependency("/usr/lib/libSystem.B.dylib"),
    ],
  });
  s.main.rpaths = ["@loader_path/../Libraries", "/developer/UE/lib"];
  s.main.dependencies = [dependency("@rpath/a.dylib")];
  const result = s.check();
  assert.deepEqual(s.read, [s.program, a, b]);
  assert.equal(result.images.length, 3);
  assert.equal(result.edges.length, 4);
  assert.equal(result.minimumMacOS, "14.1");
  assert.equal(result.edges[3].source, "macOS");
  assert.equal(result.scope, "static-linked-closure-only");
});

test("当前加载者路径优先于父链，主程序相对路径稳定", (t) => {
  const s = setup(t);
  const a = s.add(join(s.bundle, "Contents/Libraries/a.dylib"), {
    rpaths: ["@loader_path/private"],
    dependencies: [
      dependency("@rpath/b.dylib"),
      dependency("@executable_path/../Libraries/shared.dylib"),
    ],
  });
  const b = s.add(join(s.bundle, "Contents/Libraries/private/b.dylib"));
  s.add(join(s.bundle, "Contents/Libraries/b.dylib"), {
    architectures: ["x86_64"],
  });
  const shared = s.add(join(s.bundle, "Contents/Libraries/shared.dylib"));
  s.main.rpaths = ["@executable_path/../Libraries"];
  s.main.dependencies = [dependency("@rpath/a.dylib")];
  s.check();
  assert.deepEqual(s.read, [s.program, a, b, shared]);
});

test("已有包外 fallback 不补齐缺库，弱依赖也不得依赖开发机", (t) => {
  const s = setup(t);
  s.add(join(s.directory, "engine/only.dylib"));
  s.main.rpaths = [join(s.directory, "engine")];
  for (const weak of [false, true]) {
    s.main.dependencies = [dependency("@rpath/only.dylib", weak)];
    assert.throws(() => s.check(), /缺少可解析依赖/);
  }
  assert.ok(s.read.every((file) => file === s.program));
});

test("不能以任意系统前缀或绝对工程路径冒充可移动依赖", (t) => {
  const s = setup(t);
  const lib = s.add(join(s.bundle, "Contents/Libraries/fixed.dylib"));
  for (const name of [
    lib,
    "/usr/lib/../../usr/local/evil.dylib",
    "/System/Library/../../private/evil.dylib",
  ]) {
    s.main.dependencies = [dependency(name)];
    assert.throws(() => s.check(), /固定绝对路径/);
  }
});

test("内部别名允许，越出包的符号链接和目录拒绝", (t) => {
  const s = setup(t);
  const library = s.add(join(s.bundle, "Contents/Libraries/real.dylib"));
  symlinkSync("real.dylib", join(dirname(library), "alias.dylib"));
  s.main.rpaths = ["@loader_path/../Libraries"];
  s.main.dependencies = [dependency("@rpath/alias.dylib")];
  assert.equal(s.check().edges[0].target, library);
  const other = s.add(join(s.directory, "elsewhere.dylib"));
  symlinkSync(other, join(dirname(library), "escape.dylib"));
  s.main.dependencies = [dependency("@rpath/escape.dylib")];
  assert.throws(() => s.check(), /越出应用包/);
  mkdirSync(join(dirname(library), "folder.dylib"));
  s.main.dependencies = [dependency("@rpath/folder.dylib")];
  assert.throws(() => s.check(), /不是文件/);
});

test("程序入口越界、非 Game 路径和包外入口链接拒绝", (t) => {
  const s = setup(t);
  assert.throws(() => inspectMacBundle(root, s.directory), /Game 程序/);
  const allowed = join(s.directory, "allowed-project");
  mkdirSync(allowed);
  assert.throws(
    () =>
      inspectMacBundle(allowed, s.program, {
        readImage: () => {
          throw new Error("not reached");
        },
      }),
    /越出项目／应用包/,
  );
  const outside = s.add(join(s.directory, "outside-program"));
  rmSync(s.program);
  symlinkSync(outside, s.program);
  assert.throws(() => s.check(), /越出项目／应用包/);
});

test("架构、镜像角色和间接检查失败原子拒绝", (t) => {
  const s = setup(t);
  s.main.architectures = ["x86_64"];
  assert.throws(() => s.check(), /ARM64/);
  s.main.architectures = ["arm64"];
  s.main.executable = false;
  assert.throws(() => s.check(), /角色不符/);
  s.main.executable = true;
  const lib = s.add(join(s.bundle, "Contents/Libraries/a.dylib"), {
    dylib: false,
  });
  s.main.dependencies = [dependency("@loader_path/../Libraries/a.dylib")];
  assert.throws(() => s.check(), /角色不符/);
  s.metadata.get(lib).dylib = true;
  s.metadata.get(lib).architectures = ["x86_64"];
  assert.throws(() => s.check(), /ARM64/);
});

test("主程序不能借循环去重冒充动态库", (t) => {
  const s = setup(t);
  s.main.dependencies = [dependency("@loader_path/StageMasterPreview")];
  assert.throws(() => s.check(), /动态库依赖指向主程序/);
});

test("镜像和依赖预算实际生效，无效预算不能扩权", (t) => {
  const s = setup(t);
  s.add(join(s.bundle, "Contents/MacOS/a.dylib"));
  s.main.dependencies = [
    dependency("@loader_path/a.dylib"),
    dependency("/usr/lib/libSystem.B.dylib"),
  ];
  assert.throws(() => s.check({ maxImages: 1 }), /镜像预算/);
  assert.throws(() => s.check({ maxDependencies: 1 }), /依赖预算/);
  for (const value of [0, -1, 1.5, NaN, 129])
    assert.throws(() => s.check({ maxImages: value }), /预算无效/);
  assert.throws(() => s.check({ maxDependencies: 4097 }), /预算无效/);
});

test("真实工具拒绝文本伪装的 Mach-O，不执行其内容", (t) => {
  const s = setup(t);
  assert.throws(
    () => readMacImage(root, s.program),
    /lipo|architecture|Mach-O|Command failed/,
  );
});

test("只读 CLI 缺参／额外参数／无效程序明确失败", () => {
  const cli = fileURLToPath(new URL("./inspect-renderer.mjs", import.meta.url));
  for (const args of [[], ["a", "b"], [join(root, "tmp/no-preview-app")]]) {
    const result = spawnSync(process.execPath, [cli, ...args], {
      cwd: root,
      env: { ...process.env, TMPDIR: join(root, "tmp") },
      encoding: "utf8",
    });
    assert.equal(result.status, 1);
    assert.equal(result.stdout, "");
    assert.match(result.stderr, /只指定|Game 程序/);
  }
});

test("工具／运行环境去掉所有 DYLD 覆盖，不修改调用者或 HOME", () => {
  const original = {
    HOME: process.env.HOME,
    PATH: "existing",
    DYLD_LIBRARY_PATH: "outside",
    DYLD_INSERT_LIBRARIES: "outside",
    DYLD_FUTURE_OPTION: "outside",
    TMPDIR: join(root, "tmp"),
  };
  const clean = withoutLoaderOverrides(original);
  assert.deepEqual(clean, {
    HOME: process.env.HOME,
    PATH: "existing",
    TMPDIR: join(root, "tmp"),
  });
  assert.equal(original.DYLD_LIBRARY_PATH, "outside");
});
