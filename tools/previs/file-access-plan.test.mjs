import assert from "node:assert/strict";
import { test } from "node:test";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { join } from "node:path";
import {
  assertFileReport,
  fileAccessKey,
  fileAccessPlan,
  fileAccessRun,
  scopedFileEntitlements,
} from "./file-access-plan.mjs";
import { successfulReport, runtimeAssetTest } from "./packaging-plan.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const plan = () =>
  fileAccessPlan(
    root,
    "data/component/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
    "previs-file-access-A1",
  );
const base = () => ({
  "com.apple.security.app-sandbox": true,
  "com.apple.security.get-task-allow": true,
  "com.apple.security.network.client": true,
  "com.apple.security.network.server": true,
});
test("限定既有项目内 Mac ARM64 主程序和独立实例", () => {
  for (const [platform, arch] of [
    ["linux", "arm64"],
    ["darwin", "x64"],
  ])
    assert.throws(
      () =>
        fileAccessPlan(
          root,
          plan().source,
          "previs-file-access-A1",
          platform,
          arch,
        ),
      /Mac ARM64/,
    );
  for (const source of [
    "/outside/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
    "data/StageMasterPreview.app/other",
    "data/other.app/Contents/MacOS/StageMasterPreview",
  ])
    assert.throws(
      () => fileAccessPlan(root, source, "previs-file-access-A1"),
      /来源必须/,
    );
  for (const id of ["", "../x", "previs-file-access-a/else"])
    assert.throws(() => fileAccessPlan(root, plan().source, id), /名称无效/);
});
test("只保留 Development 沙盒四项加五个实例目录，均有尾斜线", () => {
  const p = plan(),
    previous = base(),
    rights = scopedFileEntitlements(p, previous);
  assert.equal(Object.keys(rights).length, 5);
  assert.equal(rights["com.apple.security.app-sandbox"], true);
  assert.deepEqual(
    rights[fileAccessKey],
    [p.user, p.cache, p.runtimeTemporary, p.logs, p.report].map(
      (dir) => `${dir}/`,
    ),
  );
  assert.deepEqual(previous, base());
  assert.equal(p.env.TMPDIR, p.runtimeTemporary);
  assert.ok(p.env.CFFIXED_USER_HOME.startsWith(`${p.user}/`));
  assert.equal("HOME" in p.env, false);
});
for (const key of Object.keys(base()))
  test(`缺失或关闭原资格拒绝：${key}`, () => {
    const values = base();
    delete values[key];
    assert.throws(() => scopedFileEntitlements(plan(), values), /四项/);
    values[key] = false;
    assert.throws(() => scopedFileEntitlements(plan(), values), /四项/);
  });
test("未知继承／设备／文件资格拒绝，不保留扩大权限", () => {
  for (const key of [
    "com.apple.security.inherit",
    "com.apple.security.device.usb",
    fileAccessKey,
  ])
    assert.throws(
      () => scopedFileEntitlements(plan(), { ...base(), [key]: true }),
      /四项/,
    );
});
for (const field of ["user", "cache", "runtimeTemporary", "logs", "report"])
  test(`拒绝扩大 ${field} 到项目根或更改实例`, () => {
    for (const value of [
      root,
      join(root, "tmp"),
      join(root, "data/PREVIS-004/previs-file-access-Other"),
    ])
      assert.throws(
        () => scopedFileEntitlements({ ...plan(), [field]: value }, base()),
        /专用布局/,
      );
  });
test("运行原 Game／原用例，路径作为单参数且不启 Xcode／设备／声音", () => {
  const p = plan(),
    report = join(p.report, "中文 报告");
  const command = fileAccessRun(p, "/program space", "allowed", report);
  assert.equal(command.program, "/program space");
  assert.ok(command.args.includes(`-ReportExportPath=${report}`));
  assert.equal(
    command.args.filter((arg) => arg.startsWith("-ReportExportPath=")).length,
    1,
  );
  for (const arg of [
    "-NullRHI",
    "-NoSound",
    "-NoTraceServer",
    `-ExecCmds=Automation RunTests ${runtimeAssetTest}; Automation Quit`,
  ])
    assert.ok(command.args.includes(arg));
  assert.ok(command.log.startsWith(p.logs));
});
const log =
  "Successfully wrote json results file!\nSuccessfully wrote html results file!";
const report = () => ({
  succeeded: 1,
  failed: 0,
  notRun: 0,
  inProcess: 0,
  tests: [{ fullTestPath: runtimeAssetTest, state: "Success", errors: 0 }],
});
test("实际 JSON／HTML导出保护叠加而非替换原报告校验", () => {
  assert.deepEqual(
    successfulReport(
      assertFileReport(report(), "<html></html>", log),
      runtimeAssetTest,
    ),
    [runtimeAssetTest],
  );
  const failed = report();
  failed.tests[0].state = "Fail";
  assert.throws(() =>
    successfulReport(
      assertFileReport(failed, "<html></html>", log),
      runtimeAssetTest,
    ),
  );
});
for (const [name, html, text] of [
  ["空HTML", "", log],
  ["非HTML", "placeholder", log],
  ["无JSON", "<html>", "Successfully wrote html results file!"],
  ["无HTML", "<html>", "Successfully wrote json results file!"],
  ["模板错误", "<html>", `${log}\nFailed to load test report html template`],
])
  test(`拒绝${name}`, () =>
    assert.throws(() => assertFileReport(report(), html, text), /实际成功/));
test("正式 CLI 缺参数／多参数在写入或签名前拒绝", () => {
  const script = fileURLToPath(
    new URL("./qualify-renderer-files.mjs", import.meta.url),
  );
  for (const args of [[], ["first", "other"]]) {
    const result = spawnSync(process.execPath, [script, ...args], {
      cwd: root,
      env: { ...process.env, TMPDIR: join(root, "tmp") },
      encoding: "utf8",
    });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /只接受一个/);
    assert.equal(result.stdout, "");
  }
});
