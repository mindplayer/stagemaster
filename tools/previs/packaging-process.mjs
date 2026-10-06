import { spawn } from "node:child_process";
import { closeSync, existsSync, openSync, writeFileSync } from "node:fs";
import { withoutLoaderOverrides } from "./packaging-tools.mjs";
import { reporterScope } from "./crash-reporter-scope.mjs";
import {
  cleanupReporters,
  prepareReporterCleanup,
} from "./crash-reporter-cleanup.mjs";

export async function runCommand(command, plan) {
  const scope = reporterScope(command, plan);
  if (scope) {
    if (existsSync(`${command.log}.crash-reporters.json`))
      throw new Error("已有报告助手收尾记录，拒绝覆盖或重放验证");
    prepareReporterCleanup(scope);
  }
  console.log(`执行：${command.program}\n日志：${command.log}`);
  const descriptor = openSync(command.log, "wx", 0o600);
  return new Promise((resolve, reject) => {
    const env = withoutLoaderOverrides({
      ...(plan.inheritEnvironment === false ? {} : process.env),
      ...plan.env,
    });
    delete env.UE_SKIP_UBT_SDK_SETUP;
    const child = spawn(command.program, command.args, {
      cwd: plan.root,
      env,
      detached: true,
      stdio: ["ignore", descriptor, descriptor],
    });
    closeSync(descriptor);
    let interrupted = false;
    let settled = false;
    const interrupt = () => {
      interrupted = true;
      if (!settled && child.pid)
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
    child.on("close", async (code, signal) => {
      settled = true;
      const commandFailure = () =>
        failure || interrupted || code !== 0
          ? (failure ??
            new Error(
              `子进程${interrupted ? "已取消" : "失败"}：${code ?? signal}，见 ${command.log}`,
            ))
          : null;
      try {
        const crashReporters = scope ? await cleanupReporters(scope) : null;
        if (crashReporters)
          writeFileSync(
            `${command.log}.crash-reporters.json`,
            `${JSON.stringify(crashReporters, null, 2)}\n`,
            { flag: "wx", mode: 0o600 },
          );
        const originalFailure = commandFailure();
        if (originalFailure) {
          if (crashReporters) originalFailure.crashReporters = crashReporters;
          reject(originalFailure);
        } else if (crashReporters?.status === "failed") {
          const error = new Error(`报告助手收尾失败：${crashReporters.error}`);
          error.crashReporters = crashReporters;
          reject(error);
        } else
          resolve({
            pid: child.pid,
            code,
            signal,
            ...(crashReporters ? { crashReporters } : {}),
          });
      } catch (error) {
        const originalFailure = commandFailure();
        if (originalFailure) {
          originalFailure.cleanupRecordError = error.message;
          reject(originalFailure);
        } else reject(error);
      } finally {
        process.removeListener("SIGINT", interrupt);
        process.removeListener("SIGTERM", interrupt);
      }
    });
  });
}
