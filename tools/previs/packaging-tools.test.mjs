import assert from "node:assert/strict";
import { test } from "node:test";
import {
  metalToolchain,
  requirePlatformTempPermission,
} from "./packaging-tools.mjs";

test("没有明确系统临时例外时拒绝新 Xcode 构建", () => {
  for (const value of [undefined, "", "0", "true", "yes"])
    assert.throws(() => requirePlatformTempPermission(value), /用户允许/);
  requirePlatformTempPermission("1");
});

test("只读发现既有 Metal 工具链，不继承隔离发现路径、不改 HOME", () => {
  const result = metalToolchain(
    "/project",
    (program, args, options) => {
      assert.equal(program, "/usr/bin/xcrun");
      assert.deepEqual(args, ["--no-cache", "--find", "metallib"]);
      assert.equal(options.env.TMPDIR, "/project/tmp");
      assert.equal(options.env.HOME, process.env.HOME);
      assert.ok(!("CFFIXED_USER_HOME" in options.env));
      assert.equal(options.timeout, 10000);
      return {
        status: 0,
        stdout: "/installed/Metal.xctoolchain/usr/bin/metallib\n",
      };
    },
    () => true,
  );
  assert.equal(result.bin, "/installed/Metal.xctoolchain/usr/bin");
});

for (const [name, result] of [
  ["失败", { status: 1, stdout: "" }],
  ["超时", { status: null, error: new Error("timeout") }],
  ["相对路径", { status: 0, stdout: "bin/metallib" }],
])
  test(`拒绝 Metal 发现${name}`, () => {
    assert.throws(
      () =>
        metalToolchain(
          "/project",
          () => result,
          () => true,
        ),
      /Metal/,
    );
  });

test("实际程序缺项拒绝，不以工具发现输出为通过", () => {
  assert.throws(
    () =>
      metalToolchain(
        "/project",
        () => ({ status: 0, stdout: "/installed/metallib" }),
        (path) => path.endsWith("/metallib"),
      ),
    /缺少/,
  );
});
