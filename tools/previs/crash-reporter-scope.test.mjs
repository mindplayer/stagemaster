import assert from "node:assert/strict";
import { test } from "node:test";
import { mkdirSync, mkdtempSync, rmSync, symlinkSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { reporterScope } from "./crash-reporter-scope.mjs";

const root = resolve(fileURLToPath(new URL("../../", import.meta.url)));
function setup(t) {
  const temporary = mkdtempSync(join(root, "tmp/previs-package-"));
  const engine = join(temporary, "engine");
  const plan = {
    root,
    engine,
    temporary,
    env: { CFFIXED_USER_HOME: join(temporary, "platform-user") },
  };
  const command = {
    program: join(engine, "Engine/Build/BatchFiles/RunUAT.sh"),
    crashReporters: "editor",
  };
  t.after(() => rmSync(temporary, { recursive: true, force: true }));
  return { plan, command, temporary };
}

test("无明确 UE 收尾声明的静态／Node／签名命令不探测", () => {
  assert.equal(reporterScope({ program: "/usr/bin/codesign" }, {}), null);
});

test("范围来自准确唯一布局与固定 UE 可执行文件，而非任意白名单", (t) => {
  const { plan, command } = setup(t);
  const scope = reporterScope(command, plan);
  assert.equal(scope.uid, process.getuid());
  assert.equal(
    scope.reportRoot,
    join(
      plan.env.CFFIXED_USER_HOME,
      "Library/Application Support/Epic/UnrealEngine/5.8/Saved/Crashes",
    ),
  );
  assert.equal(
    scope.executable,
    join(
      plan.engine,
      "Engine/Binaries/Mac/CrashReportClientEditor.app/Contents/MacOS/CrashReportClientEditor",
    ),
  );
});

test("临时根、工程根、其他项目、全局用户目录及不符实例名称均拒绝", (t) => {
  const { plan, command } = setup(t);
  for (const temporary of [
    root,
    join(root, "tmp"),
    join(root, "tmp/other-instance"),
    "/other/project/tmp/previs-package-x",
  ])
    assert.throws(() => reporterScope(command, { ...plan, temporary }));
  for (const home of [
    root,
    join(root, "tmp"),
    "/Users/sunqi/Library",
    join(root, "tmp/previs-package-other/platform-user"),
  ])
    assert.throws(() =>
      reporterScope(command, { ...plan, env: { CFFIXED_USER_HOME: home } }),
    );
});

test("命令／类型不符不得借 UAT 收尾权限", (t) => {
  const { plan, command } = setup(t);
  for (const changed of [
    { program: "/bin/sh" },
    { crashReporters: "unknown" },
    { crashReporters: "game" },
  ])
    assert.throws(() => reporterScope({ ...command, ...changed }, plan));
});

test("用户目录新建之前的链接祖先也拒绝，不沿链接扩到其他范围", (t) => {
  const { plan, command, temporary } = setup(t);
  const target = join(temporary, "other-home");
  mkdirSync(target);
  symlinkSync(target, plan.env.CFFIXED_USER_HOME);
  assert.throws(() => reporterScope(command, plan), /链接/);
});

test("项目内独立 Game 固定附属报告助手，不借其他实例或共享编辑器 fallback", (t) => {
  const { plan, temporary } = setup(t);
  const archive = join(root, "data/PREVIS-004", basename(temporary));
  const program = join(
    archive,
    "StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
  );
  const scope = reporterScope({ program, crashReporters: "game" }, plan);
  assert.equal(
    scope.executable,
    join(
      archive,
      "StageMasterPreview.app/Contents/UE/Engine/Binaries/Mac/CrashReportClient.app/Contents/MacOS/CrashReportClient",
    ),
  );
  for (const bad of [
    "/other/project/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
    join(
      root,
      "data/PREVIS-004/previs-package-other/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
    ),
    join(
      root,
      "output/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
    ),
  ])
    assert.throws(() =>
      reporterScope({ program: bad, crashReporters: "game" }, plan),
    );
});
