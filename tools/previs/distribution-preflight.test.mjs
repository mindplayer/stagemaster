import assert from "node:assert/strict";
import { test } from "node:test";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { inspectDistribution } from "./distribution-preflight.mjs";
const root = fileURLToPath(new URL("../../", import.meta.url));
function fixture(t) {
  const directory = mkdtempSync(join(root, "tmp/previs-distribution-test-")),
    bundle = join(directory, "中文 空格.app");
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const files = [
    "Contents/Info.plist",
    "Contents/MacOS/stagemaster-desktop",
    "Contents/MacOS/stagemaster-execution-host",
    "Contents/Resources/previs/node",
    "Contents/Resources/previs/StageMasterPreview.app/Contents/Info.plist",
    "Contents/Resources/previs/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
    "Contents/Resources/previs/StageMasterPreview.app/Contents/UE/StageMasterPreview/Content/Paks/StageMasterPreview-Mac.pak",
  ];
  for (const file of files) {
    const path = join(bundle, file);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, "fixture", { mode: 0o755 });
  }
  const calls = [];
  const execute = (program, args, options) => {
    calls.push({ program, args, options });
    let stdout = "";
    if (program.endsWith("PlistBuddy"))
      stdout = {
        "Print :CFBundleExecutable": "stagemaster-desktop",
        "Print :CFBundleIdentifier": "cn.stagemaster.desktop",
        "Print :LSMinimumSystemVersion": "14.0",
      }[args[1]];
    else if (args.includes("--display"))
      stdout =
        "Identifier=product\nCodeDirectory v=20500 size=42 flags=0x10000(runtime) hashes=1+1 location=embedded\nTeamIdentifier=ABCDEFGHIJ\nTimestamp=Jun 18, 2026 at 02:18:26\n";
    return { status: 0, stdout, stderr: "" };
  };
  const opts = {
    execute,
    readImage: () => ({
      architectures: ["arm64"],
      minimumMacOS: "14.0",
      executable: true,
    }),
    inspectDependencies: () => ({
      minimumMacOS: "14.0",
      scope: "static-linked-closure-only",
    }),
    os: "darwin",
    architecture: "arm64",
  };
  return { directory, bundle, calls, opts };
}
test("四角色中文空格路径实际采集，不签名／执行／下载，只报告静态先决", (t) => {
  const f = fixture(t),
    r = inspectDistribution(root, [f.bundle], f.opts);
  assert.equal(r.status, "static-prerequisites-passed");
  assert.equal(r.releaseQualified, false);
  assert.equal(r.readOnly, true);
  assert.equal(r.evidence.images.length, 4);
  assert.equal(f.calls.filter((c) => c.args.includes("-R")).length, 4);
  assert.ok(
    f.calls.every((c) =>
      ["/usr/libexec/PlistBuddy", "/usr/bin/codesign"].includes(c.program),
    ),
  );
  assert.ok(
    f.calls.every(
      (c) =>
        c.options.env.TMPDIR === join(root, "tmp") &&
        c.options.timeout === 10000,
    ),
  );
  assert.equal(
    f.calls.some(
      (c) => c.args.includes("--sign") || c.args.includes("--force"),
    ),
    false,
  );
});
test("缺件不使用系统Node或编辑器补齐，读取签名前拒绝", (t) => {
  const f = fixture(t);
  rmSync(join(f.bundle, "Contents/Resources/previs/node"));
  assert.throws(() => inspectDistribution(root, [f.bundle], f.opts));
  assert.equal(
    f.calls.some((c) => c.program.endsWith("codesign")),
    false,
  );
});
test("入口缺执行权限在签名检查前拒绝", (t) => {
  const f = fixture(t);
  chmodSync(join(f.bundle, "Contents/Resources/previs/node"), 0o644);
  assert.throws(() => inspectDistribution(root, [f.bundle], f.opts), /执行/);
  assert.equal(
    f.calls.some((c) => c.program.endsWith("codesign")),
    false,
  );
});
test("原签名Node仍有调试资格时明确阻止，不自动重签官方程序", (t) => {
  const f = fixture(t),
    original = f.opts.execute;
  f.opts.execute = (program, args, options) => {
    const response = original(program, args, options);
    if (args.includes("--entitlements") && args.at(-1).endsWith("/node"))
      response.stdout = "<plist>actual entitlement</plist>";
    if (program.endsWith("plutil"))
      response.stdout = JSON.stringify({
        "com.apple.security.get-task-allow": true,
      });
    return response;
  };
  const result = inspectDistribution(root, [f.bundle], f.opts);
  assert.ok(result.issues.some((i) => i.role === "node" && i.code === "debug"));
  assert.equal(result.releaseQualified, false);
  assert.equal(
    f.calls.some((c) => c.args.includes("--sign")),
    false,
  );
});
test("程序和包目录链接拒绝，不能读取链接目标", (t) => {
  const f = fixture(t),
    node = join(f.bundle, "Contents/Resources/previs/node"),
    other = join(f.directory, "external-node");
  writeFileSync(other, "node");
  rmSync(node);
  symlinkSync(other, node);
  assert.throws(() => inspectDistribution(root, [f.bundle], f.opts), /链接/);
  const linked = join(f.directory, "linked.app");
  symlinkSync(f.bundle, linked);
  assert.throws(() => inspectDistribution(root, [linked], f.opts), /规范/);
});
test("包外、根、错误数量与平台在任何工具运行前拒绝", (t) => {
  const f = fixture(t);
  for (const args of [
    [],
    [f.bundle, f.bundle],
    [root],
    ["/outside/app.app"],
    ["../escape.app"],
  ])
    assert.throws(() => inspectDistribution(root, args, f.opts));
  assert.throws(
    () => inspectDistribution(root, [f.bundle], { ...f.opts, os: "linux" }),
    /Mac ARM64/,
  );
  assert.equal(f.calls.length, 0);
});
test("工具错误、信号与未知完成状态没有成功报告", (t) => {
  const f = fixture(t);
  for (const response of [
    { error: new Error("timeout"), status: null },
    { signal: "SIGTERM", status: null },
    { status: null },
  ])
    assert.throws(
      () =>
        inspectDistribution(root, [f.bundle], {
          ...f.opts,
          execute: () => response,
        }),
      /未完成/,
    );
});
test("完整外层也须分别校验执行宿主证书，文字不能代替native拒绝", (t) => {
  const f = fixture(t),
    original = f.opts.execute;
  f.opts.execute = (program, args, options) => {
    const response = original(program, args, options);
    if (
      args.includes("-R") &&
      args.at(-1).endsWith("stagemaster-execution-host")
    )
      response.status = 1;
    return response;
  };
  const result = inspectDistribution(root, [f.bundle], f.opts);
  assert.equal(result.status, "blocked");
  assert.ok(
    result.issues.some(
      (i) => i.role === "execution-host" && i.code === "developer-id",
    ),
  );
});
test("签名显示失败与读取权限失败均保留原始拒绝，不当空权限", (t) => {
  const f = fixture(t),
    original = f.opts.execute;
  f.opts.execute = (program, args, options) => {
    const response = original(program, args, options);
    if (args.includes("--display") || args.includes("--entitlements")) {
      response.status = 1;
      response.stderr = "unsigned";
    }
    return response;
  };
  const result = inspectDistribution(root, [f.bundle], f.opts);
  assert.equal(result.status, "blocked");
  assert.ok(result.issues.some((i) => i.code === "signature-metadata"));
  assert.ok(result.issues.some((i) => i.code === "entitlements"));
});
