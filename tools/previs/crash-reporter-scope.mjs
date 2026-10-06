import { existsSync, lstatSync, realpathSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";

export function reporterScope(command, plan) {
  if (command.crashReporters === undefined) return null;
  if (process.platform !== "darwin")
    throw new Error("报告助手收尾仅支持已验证的 macOS 工具");
  const root = realpathSync(plan.root);
  const temporary = resolve(plan.temporary);
  const id = basename(temporary);
  if (temporary !== join(root, "tmp", id))
    throw new Error("报告收尾须属于项目内独立临时实例");
  let home;
  if (/^previs-package-[a-zA-Z0-9]+$/.test(id))
    home = join(temporary, "platform-user");
  else if (/^previs-file-access-[a-zA-Z0-9]+$/.test(id))
    home = join(root, "data/PREVIS-004", id, "runtime-user/platform-user");
  else throw new Error("报告收尾实例名称无效");
  if (plan.env.CFFIXED_USER_HOME !== home)
    throw new Error("报告收尾目录不匹配本实例用户布局");
  let executable;
  if (command.crashReporters === "editor") {
    const engine = resolve(plan.engine);
    if (
      resolve(command.program) !==
      join(engine, "Engine/Build/BatchFiles/RunUAT.sh")
    )
      throw new Error("编辑器报告收尾不属于原 UAT 命令");
    executable = join(
      engine,
      "Engine/Binaries/Mac/CrashReportClientEditor.app/Contents/MacOS/CrashReportClientEditor",
    );
  } else if (command.crashReporters === "game") {
    const program = resolve(command.program);
    const bundle = dirname(dirname(dirname(program)));
    const archive = join(root, "data/PREVIS-004", id);
    if (
      program !== join(bundle, "Contents/MacOS/StageMasterPreview") ||
      !bundle.startsWith(`${archive}/`) ||
      !bundle.endsWith("/StageMasterPreview.app")
    )
      throw new Error("Game 报告收尾须属于项目内独立组件");
    executable = join(
      bundle,
      "Contents/UE/Engine/Binaries/Mac/CrashReportClient.app/Contents/MacOS/CrashReportClient",
    );
  } else throw new Error("报告收尾类型无效");
  const reportRoot = join(
    home,
    "Library/Application Support/Epic/UnrealEngine/5.8/Saved/Crashes",
  );
  for (let path = reportRoot; path !== root; path = dirname(path)) {
    if (existsSync(path)) {
      if (!lstatSync(path).isDirectory() || realpathSync(path) !== path)
        throw new Error("报告收尾目录存在链接或非真实目录");
    }
  }
  return { reportRoot, executable, uid: process.getuid() };
}
