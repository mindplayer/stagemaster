import assert from "node:assert/strict";
import { test } from "node:test";
import {
  parseMacLoadCommands,
  compareMacVersions,
  macVersion,
} from "./mac-load-commands.mjs";

const output = (...blocks) =>
  `fixture:\n${blocks.map((block, i) => `Load command ${i}\n${block}`).join("\n")}\n`;
const version = "cmd LC_BUILD_VERSION\n platform 1\n minos 14.0\n sdk 26.1";

test("解析原始加载命令，ID 不误当依赖，保留弱依赖与路径空格", () => {
  const result = parseMacLoadCommands(
    output(
      version,
      "cmd LC_MAIN",
      "cmd LC_ID_DYLIB\n name @rpath/own.dylib (offset 24)",
      "cmd LC_LOAD_DYLIB\n name @rpath/stage library.dylib (offset 24)",
      "cmd LC_LOAD_WEAK_DYLIB\n name /usr/lib/libSystem.B.dylib (offset 24)",
      "cmd LC_RPATH\n path @loader_path/../stage library (offset 12)",
    ),
  );
  assert.equal(result.minimumMacOS, "14.0");
  assert.equal(result.executable, true);
  assert.equal(result.dylib, true);
  assert.deepEqual(result.rpaths, ["@loader_path/../stage library"]);
  assert.deepEqual(
    result.dependencies.map(({ name, weak }) => [name, weak]),
    [
      ["@rpath/stage library.dylib", false],
      ["/usr/lib/libSystem.B.dylib", true],
    ],
  );
});

test("旧 macOS 最低版本与 UNIXTHREAD 入口仍可解析", () => {
  const result = parseMacLoadCommands(
    output(
      "cmd LC_VERSION_MIN_MACOSX\n version 10.13\n sdk 10.15",
      "cmd LC_UNIXTHREAD",
    ),
  );
  assert.equal(result.minimumMacOS, "10.13");
  assert.equal(result.executable, true);
});

test("转发、向上及延迟依赖不漏检", () => {
  for (const command of [
    "LC_REEXPORT_DYLIB",
    "LC_LOAD_UPWARD_DYLIB",
    "LC_LAZY_LOAD_DYLIB",
  ])
    assert.equal(
      parseMacLoadCommands(
        output(version, `cmd ${command}\n name @rpath/child.dylib (offset 24)`),
      ).dependencies[0].command,
      command,
    );
});

for (const [name, text] of [
  ["空输出", ""],
  ["无加载命令", "not a Mach-O file"],
  ["缺最低版本", output("cmd LC_MAIN")],
  ["重复最低版本", output(version, version)],
  ["非法版本", output(version.replace("14.0", "x.0"))],
  ["非 Mac 平台", output(version.replace("platform 1", "platform 2"))],
  [
    "旧 iOS 平台",
    output(version, "cmd LC_VERSION_MIN_IPHONEOS\n version 12.0"),
  ],
  ["缺命令", output(version, "cmdsize 32")],
  ["缺库名", output(version, "cmd LC_LOAD_DYLIB")],
  ["缺搜索路径", output(version, "cmd LC_RPATH")],
  [
    "环境注入",
    output(
      version,
      "cmd LC_DYLD_ENVIRONMENT\n name DYLD_LIBRARY_PATH=/engine (offset 12)",
    ),
  ],
  [
    "路径控制字符",
    output(
      version,
      "cmd LC_LOAD_DYLIB\n name @rpath/x\u0000.dylib (offset 24)",
    ),
  ],
])
  test(`拒绝${name}`, () =>
    assert.throws(() => parseMacLoadCommands(text), /Mach-O/));

test("系统版本按数字比较，缺省补零，不按字符串排序", () => {
  assert.ok(compareMacVersions("14.0", "9.9") > 0);
  assert.ok(compareMacVersions("14.10", "14.9.9") > 0);
  assert.equal(compareMacVersions("14", "14.0.0"), 0);
  assert.ok(compareMacVersions("14.0.1", "14.1") < 0);
  assert.deepEqual(macVersion("10.13"), [10, 13, 0]);
  for (const value of [
    undefined,
    "",
    "14.0.0.1",
    "-1",
    "Infinity",
    "99999999999999999999",
    "1.x",
    "65536.0",
  ])
    assert.throws(() => macVersion(value), /无效/);
});
