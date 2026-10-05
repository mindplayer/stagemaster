import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { runtimeAssetTest, runtimeCheck } from "./packaging-plan.mjs";

export const fileAccessKey =
  "com.apple.security.temporary-exception.files.absolute-path.read-write";
const baseKeys = [
  "com.apple.security.app-sandbox",
  "com.apple.security.get-task-allow",
  "com.apple.security.network.client",
  "com.apple.security.network.server",
];
export function fileAccessPlan(
  root,
  source,
  id,
  platform = process.platform,
  arch = process.arch,
) {
  if (platform !== "darwin" || arch !== "arm64")
    throw new Error("文件资格只支持 Mac ARM64 Development 组件");
  if (!/^previs-file-access-[a-zA-Z0-9]+$/.test(id))
    throw new Error("文件资格实例名称无效");
  const project = resolve(root),
    input = resolve(project, source);
  const bundle = dirname(dirname(dirname(input)));
  const within = relative(project, input);
  if (
    within === ".." ||
    within.startsWith(`..${sep}`) ||
    isAbsolute(within) ||
    input !== join(bundle, "Contents/MacOS/StageMasterPreview") ||
    !bundle.endsWith("/StageMasterPreview.app")
  )
    throw new Error("来源必须是项目内独立 Game 主程序");
  const temporary = join(project, "tmp", id),
    archive = join(project, "data/PREVIS-004", id);
  const logs = join(project, "logs/PREVIS-004", id);
  const user = join(archive, "runtime-user"),
    cache = join(archive, "runtime-cache");
  const runtimeTemporary = join(temporary, "runtime-temp"),
    report = join(temporary, "runtime-report");
  return {
    root: project,
    id,
    source: input,
    sourceBundle: bundle,
    temporary,
    archive,
    logs,
    user,
    cache,
    runtimeTemporary,
    report,
    bundle: join(archive, "StageMasterPreview.app"),
    entitlementFile: join(temporary, "development.entitlements"),
    env: {
      TMPDIR: runtimeTemporary,
      CFFIXED_USER_HOME: join(user, "platform-user"),
      UE_LocalDataCachePath: cache,
      UE_DesktopUnrealProcess: "1",
    },
  };
}

export function scopedFileEntitlements(plan, base) {
  if (
    !isDeepStrictEqual(Object.keys(base).sort(), [...baseKeys].sort()) ||
    baseKeys.some((key) => base[key] !== true)
  )
    throw new Error("来源不是现行四项受限 Development 沙盒资格");
  const expected = fileAccessPlan(plan.root, plan.source, plan.id);
  const keys = ["user", "cache", "runtimeTemporary", "logs", "report"];
  if (keys.some((key) => plan[key] !== expected[key]))
    throw new Error("文件资格目录不属于本实例专用布局");
  return { ...base, [fileAccessKey]: keys.map((key) => `${plan[key]}/`) };
}

export function fileAccessRun(plan, program, name, report) {
  const command = runtimeCheck(plan, program, runtimeAssetTest, name);
  command.report = report;
  command.args = command.args.map((arg) =>
    arg.startsWith("-ReportExportPath=") ? `-ReportExportPath=${report}` : arg,
  );
  return command;
}

export function assertFileReport(report, html, engineLog) {
  if (
    !html.trim() ||
    !/<html\b/i.test(html) ||
    !engineLog.includes("Successfully wrote json results file!") ||
    !engineLog.includes("Successfully wrote html results file!") ||
    /Failed to (load|write) test report/.test(engineLog)
  )
    throw new Error("独立 Game 缺实际成功的 JSON／HTML导出");
  // Keep the original required resource/H264 and strict result validation.
  return report;
}
