import assert from "node:assert/strict";
import { test } from "node:test";
import {
  mkdirSync,
  mkdtempSync,
  writeFileSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  packagingPlan,
  runtimeCheck,
  runtimeAssetTest,
  successfulReport,
} from "./packaging-plan.mjs";
import { packagedProgram } from "./package-renderer.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const plan = () =>
  packagingPlan(
    "/workspace/stage master",
    "/installed/UE 5.8",
    "previs-package-Ab1234",
    "darwin",
    "arm64",
  );
const passing = () => ({
  succeeded: 1,
  failed: 0,
  notRun: 0,
  inProcess: 0,
  tests: [{ fullTestPath: runtimeAssetTest, state: "Success", errors: 0 }],
});

test("只支持实际限定的 Mac ARM64，实例拒绝路径穿越", () => {
  for (const [platform, arch] of [
    ["linux", "arm64"],
    ["darwin", "x64"],
    ["win32", "arm64"],
  ])
    assert.throws(
      () =>
        packagingPlan(root, "/engine", "previs-package-Ab1234", platform, arch),
      /仅支持/,
    );
  for (const id of [
    "",
    "../other",
    "previs-package-A/other",
    "previs-package-A..",
  ])
    assert.throws(
      () => packagingPlan(root, "/engine", id, "darwin", "arm64"),
      /名称无效/,
    );
});

test("官方分阶段构建不部署／运行，保留实际 SDK 检查", () => {
  const { uat, env } = plan();
  for (const flag of [
    "-build",
    "-cook",
    "-stage",
    "-package",
    "-pak",
    "-archive",
    "-ubtargs=-NoUBA",
  ])
    assert.ok(uat.args.includes(flag));
  assert.ok(uat.args.includes("-target=StageMasterPreview"));
  assert.ok(uat.args.includes("-clientarchitecture=arm64"));
  assert.ok(!uat.args.includes("-deploy") && !uat.args.includes("-run"));
  assert.ok(!uat.args.includes("-skipbuildeditor"));
  assert.ok(!("UE_SKIP_UBT_SDK_SETUP" in env) && !("HOME" in env));
});

test("缓存／日志／暂存／包归档均项目内，路径空格仍为独立参数", () => {
  const value = plan();
  for (const key of ["temporary", "logs", "archive", "user", "cache"])
    assert.ok(value[key].startsWith(`${value.root}/`));
  for (const key of [
    "TMPDIR",
    "CFFIXED_USER_HOME",
    "DOTNET_CLI_HOME",
    "NUGET_PACKAGES",
    "UE_LocalDataCachePath",
    "uebp_LogFolder",
    "uebp_FinalLogFolder",
    "uebp_EngineSavedFolder",
  ])
    assert.ok(value.env[key].startsWith(`${value.root}/`));
  assert.ok(value.uat.args.includes(`-project=${value.project}`));
  const cooker = value.uat.args.find((arg) =>
    arg.startsWith("-AdditionalCookerOptions="),
  );
  assert.ok(cooker.includes(`-UserDir="${value.user}"`));
  assert.ok(
    cooker.includes("-SkipZenStore") &&
      cooker.includes("-NoZenAutoLaunch") &&
      cooker.includes("-NoEditorDomain"),
  );
});

test("Game 资源验收不借用编辑器或工程，报告和运行产物分开", () => {
  const value = plan();
  const check = runtimeCheck(
    value,
    "/package/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
  );
  assert.ok(!check.args.includes("-game"));
  assert.ok(check.args.every((arg) => !arg.includes(".uproject")));
  assert.ok(
    check.args.includes(
      `-ExecCmds=Automation RunTests ${runtimeAssetTest}; Automation Quit`,
    ),
  );
  assert.ok(check.args.includes("-NullRHI"));
  assert.ok(check.report.startsWith(`${value.temporary}/`));
  assert.ok(check.log.startsWith(`${value.logs}/`));
});

test("实际完整成功报告才接纳", () => {
  assert.deepEqual(successfulReport(passing(), runtimeAssetTest), [
    runtimeAssetTest,
  ]);
});

for (const [name, change] of [
  [
    "空报告",
    (report) => {
      report.tests = [];
      report.succeeded = 0;
    },
  ],
  [
    "缺必需用例",
    (report) => {
      report.tests[0].fullTestPath = "StageMaster.Previs.Unrelated";
    },
  ],
  [
    "失败项",
    (report) => {
      report.tests[0].state = "Fail";
    },
  ],
  [
    "错误计数",
    (report) => {
      report.tests[0].errors = 1;
    },
  ],
  [
    "汇总失败",
    (report) => {
      report.failed = 1;
    },
  ],
  [
    "未运行",
    (report) => {
      report.notRun = 1;
    },
  ],
  [
    "仍在运行",
    (report) => {
      report.inProcess = 1;
    },
  ],
  [
    "成功数不符",
    (report) => {
      report.succeeded = 0;
    },
  ],
  [
    "其他套件",
    (report) => {
      report.tests.push({
        fullTestPath: "Other.Test",
        state: "Success",
        errors: 0,
      });
      report.succeeded = 2;
    },
  ],
])
  test(`报告拒绝${name}`, () => {
    const report = passing();
    change(report);
    assert.throws(() => successfulReport(report, runtimeAssetTest), /报告/);
  });

test("归档中要求唯一真实 Game 程序；缺项／两包／符号链接不冒成功", (t) => {
  const temporary = mkdtempSync(join(root, "tmp/previs-package-test-"));
  t.after(() => rmSync(temporary, { recursive: true, force: true }));
  assert.throws(() => packagedProgram(temporary), /唯一/);
  const binary = join(
    temporary,
    "Mac/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
  );
  mkdirSync(dirname(binary), { recursive: true });
  writeFileSync(binary, "test fixture only");
  assert.equal(packagedProgram(temporary), binary);
  const other = join(
    temporary,
    "Other/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
  );
  mkdirSync(dirname(other), { recursive: true });
  writeFileSync(other, "test fixture only");
  assert.throws(() => packagedProgram(temporary), /唯一/);
  const links = join(temporary, "links");
  mkdirSync(links);
  symlinkSync(
    join(temporary, "Mac/StageMasterPreview.app"),
    join(links, "StageMasterPreview.app"),
  );
  assert.throws(() => packagedProgram(links), /唯一/);
});
