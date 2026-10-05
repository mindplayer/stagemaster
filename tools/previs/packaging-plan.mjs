import { join } from "node:path";

export const runtimeAssetTest = "StageMaster.Previs.RuntimeAssets";
const cacheGraph =
  "(ProjectPak,InstalledProjectPak,EnginePak=InstalledEnginePak,Local)";

export function packagingPlan(
  root,
  engine,
  id,
  platform = process.platform,
  arch = process.arch,
) {
  if (platform !== "darwin" || arch !== "arm64")
    throw new Error("本打包增量仅支持 Mac ARM64");
  if (!/^previs-package-[a-zA-Z0-9]+$/.test(id))
    throw new Error("打包实例名称无效");
  const project = join(root, "apps/previs-unreal/StageMasterPreview.uproject");
  const temporary = join(root, "tmp", id);
  const logs = join(root, "logs/PREVIS-004", id);
  const archive = join(root, "data/PREVIS-004", id);
  const user = join(root, "data/previs-user");
  const cache = join(root, "data/previs-derived-cache");
  const env = {
    TMPDIR: join(root, "tmp"),
    // macOS .NET uses NSApplicationSupportDirectory, not XDG_CONFIG_HOME.
    // Scoped CoreFoundation home keeps UBT writable settings local; HOME is unchanged.
    CFFIXED_USER_HOME: join(temporary, "platform-user"),
    DOTNET_CLI_HOME: join(temporary, "dotnet"),
    DOTNET_CLI_TELEMETRY_OPTOUT: "1",
    NUGET_PACKAGES: join(root, "tmp/nuget-packages"),
    UE_LocalDataCachePath: cache,
    UE_DesktopUnrealProcess: "1",
    uebp_LogFolder: join(logs, "uat"),
    uebp_FinalLogFolder: join(logs, "uat"),
    uebp_EngineSavedFolder: join(temporary, "automation-saved"),
  };
  const cooker = [
    `-UserDir="${user}"`,
    `-LocalDataCachePath="${cache}"`,
    "-SkipZenStore",
    "-NoZenAutoLaunch",
    "-NoEditorDomain",
    "-unattended",
    "-NoSound",
    "-NoSplash",
    "-NoTraceServer",
  ].join(" ");
  const uat = {
    program: join(engine, "Engine/Build/BatchFiles/RunUAT.sh"),
    args: [
      "BuildCookRun",
      `-project=${project}`,
      "-target=StageMasterPreview",
      "-platform=Mac",
      "-clientarchitecture=arm64",
      "-editorarchitecture=arm64",
      "-clientconfig=Development",
      "-installed",
      "-nocompileuat",
      "-nop4",
      "-unattended",
      "-utf8output",
      "-build",
      "-cook",
      "-stage",
      "-package",
      "-pak",
      "-archive",
      "-ubtargs=-NoUBA",
      `-stagingdirectory=${join(temporary, "stage")}`,
      `-archivedirectory=${archive}`,
      `-ddc=${cacheGraph}`,
      `-AdditionalCookerOptions=${cooker}`,
    ],
    log: join(logs, "build-cook-archive.log"),
  };
  return {
    root,
    engine,
    project,
    temporary,
    logs,
    archive,
    user,
    cache,
    env,
    uat,
  };
}

export function runtimeCheck(
  plan,
  program,
  name = runtimeAssetTest,
  reportName = "runtime-assets",
) {
  const report = join(plan.temporary, reportName);
  return {
    program,
    report,
    args: [
      "-unattended",
      "-NullRHI",
      "-NoSound",
      "-NoSplash",
      "-NoP4",
      "-NoTraceServer",
      `-UserDir=${plan.user}`,
      `-LocalDataCachePath=${plan.cache}`,
      `-DDC=${cacheGraph}`,
      `-ExecCmds=Automation RunTests ${name}; Automation Quit`,
      "-TestExit=Automation Test Queue Empty",
      `-ReportExportPath=${report}`,
      `-abslog=${join(plan.logs, `${reportName}-engine.log`)}`,
    ],
    log: join(plan.logs, `${reportName}-process.log`),
  };
}

export function successfulReport(report, required) {
  if (
    !Array.isArray(report.tests) ||
    report.tests.length === 0 ||
    report.failed !== 0 ||
    report.notRun !== 0 ||
    report.inProcess !== 0 ||
    report.succeeded !== report.tests.length ||
    report.tests.some(
      (test) =>
        !test.fullTestPath?.startsWith("StageMaster.Previs.") ||
        test.state !== "Success" ||
        test.errors !== 0,
    ) ||
    !report.tests.some((test) => test.fullTestPath === required)
  )
    throw new Error("预演自动化报告为空、缺项或存在未成功测试");
  return report.tests.map((test) => test.fullTestPath);
}
