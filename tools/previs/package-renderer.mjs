import { spawn } from "node:child_process";
import {
  openSync,
  closeSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { basename, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  packagingPlan,
  runtimeCheck,
  runtimeAssetTest,
  successfulReport,
} from "./packaging-plan.mjs";
import {
  metalToolchain,
  requirePlatformTempPermission,
} from "./packaging-tools.mjs";

export async function runCommand(command, plan) {
  console.log(`执行：${command.program}\n日志：${command.log}`);
  const descriptor = openSync(command.log, "wx", 0o600);
  return new Promise((resolve, reject) => {
    const env = { ...process.env, ...plan.env };
    // Build-time toolchain verification must not inherit the editor runtime shortcut.
    delete env.UE_SKIP_UBT_SDK_SETUP;
    const child = spawn(command.program, command.args, {
      cwd: plan.root,
      env,
      detached: true,
      stdio: ["ignore", descriptor, descriptor],
    });
    closeSync(descriptor);
    // Own the entire group. RunUAT.sh's ps --ppid cancellation is not portable to macOS.
    let interrupted = false;
    const interrupt = () => {
      interrupted = true;
      if (child.pid)
        try {
          process.kill(-child.pid, "SIGTERM");
        } catch (error) {
          if (error.code !== "ESRCH") throw error;
        }
    };
    process.once("SIGINT", interrupt);
    process.once("SIGTERM", interrupt);
    let failure;
    child.on("error", (error) => {
      failure = error;
    });
    child.on("close", (code, signal) => {
      process.removeListener("SIGINT", interrupt);
      process.removeListener("SIGTERM", interrupt);
      if (failure || interrupted || code !== 0)
        reject(
          failure ??
            new Error(
              `子进程${interrupted ? "已取消" : "失败"}：${code ?? signal}，见 ${command.log}`,
            ),
        );
      else resolve();
    });
  });
}

export function packagedProgram(archive) {
  const candidates = [];
  const visit = (directory, depth) => {
    if (depth > 3) return;
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (!entry.isDirectory()) continue;
      const path = join(directory, entry.name);
      if (entry.name === "StageMasterPreview.app") {
        const program = join(path, "Contents/MacOS/StageMasterPreview");
        if (existsSync(program) && statSync(program).isFile())
          candidates.push(program);
      } else visit(path, depth + 1);
    }
  };
  visit(archive, 0);
  if (candidates.length !== 1) throw new Error("归档必须包含唯一独立预演程序");
  return candidates[0];
}

export async function packageRenderer() {
  if (process.argv.length > 2)
    throw new Error("此命令不接受额外参数；引擎路径使用 STAGEMASTER_UE_ROOT");
  const root = fileURLToPath(new URL("../../", import.meta.url));
  if (process.platform !== "darwin" || process.arch !== "arm64")
    throw new Error("本打包增量仅支持 Mac ARM64");
  requirePlatformTempPermission(process.env.STAGEMASTER_ALLOW_PLATFORM_TEMP);
  const engine =
    process.env.STAGEMASTER_UE_ROOT ?? "/Users/Shared/Epic Games/UE_5.8";
  const project = join(root, "apps/previs-unreal/StageMasterPreview.uproject");
  const version = JSON.parse(
    readFileSync(join(engine, "Engine/Build/Build.version"), "utf8"),
  );
  if (version.MajorVersion !== 5 || version.MinorVersion !== 8)
    throw new Error("需要已验证的 UE 5.8 工具链");
  if (
    !existsSync(project) ||
    !existsSync(join(engine, "Engine/Build/BatchFiles/RunUAT.sh"))
  )
    throw new Error("缺少引擎打包工具或预演工程");
  mkdirSync(join(root, "tmp"), { recursive: true });
  const temporary = mkdtempSync(join(root, "tmp/previs-package-"));
  const plan = packagingPlan(root, engine, basename(temporary));
  const metal = metalToolchain(root);
  plan.env.PATH = `${metal.bin}:${process.env.PATH ?? "/usr/bin:/bin"}`;
  for (const path of [
    plan.logs,
    plan.user,
    plan.cache,
    ...Object.entries(plan.env)
      .filter(([key, value]) => key !== "PATH" && value.startsWith(root))
      .map(([, value]) => value),
  ])
    mkdirSync(path, { recursive: true });
  mkdirSync(plan.archive, { recursive: true });
  const evidence = {
    status: "running",
    version,
    metal,
    plan,
    startedAt: new Date().toISOString(),
  };
  const record = () =>
    writeFileSync(
      join(plan.archive, "build-record.json"),
      `${JSON.stringify(evidence, null, 2)}\n`,
    );
  record();
  try {
    await runCommand(plan.uat, plan);
    const program = packagedProgram(plan.archive);
    const check = runtimeCheck(plan, program);
    evidence.runtimeCheck = check;
    // Packaged Game lacks the editor HTML helper which otherwise creates this directory.
    // JSON evidence is required independently; never accept only its exit code or text log.
    mkdirSync(check.report, { recursive: true });
    await runCommand(check, plan);
    const report = JSON.parse(
      readFileSync(join(check.report, "index.json"), "utf8").replace(
        /^\uFEFF/,
        "",
      ),
    );
    evidence.tests = successfulReport(report, runtimeAssetTest);
    evidence.program = program;
    evidence.status = "component-verified";
    console.log(
      `独立组件／必需资源通过：${program}（未代表桌面／GPU／客户安装验收）`,
    );
  } catch (error) {
    evidence.status = "failed";
    evidence.error = error.message;
    throw error;
  } finally {
    evidence.finishedAt = new Date().toISOString();
    record();
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href)
  packageRenderer().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
