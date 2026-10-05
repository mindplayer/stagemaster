import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { desktopBuildPlan, hostEnvironment } from "./build-plan.mjs";
import { prepareWorkspaceFolders } from "./workspace-folders.mjs";

export function prepareHost(plan = undefined, environment = process.env) {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const selected = plan ?? desktopBuildPlan(root, "build");
  const env = hostEnvironment(selected, environment);
  prepareWorkspaceFolders(selected.root, [
    env.CARGO_TARGET_DIR,
    env.TMPDIR,
    selected.sidecarDirectory,
  ]);
  const info = spawnSync("rustc", ["-vV"], { encoding: "utf8", env });
  if (info.status !== 0) throw new Error("无法确定 Rust 目标平台");
  const triple = info.stdout.match(/^host: (\S+)$/m)?.[1];
  if (!triple) throw new Error("缺少 Rust 目标平台");
  const build = spawnSync("cargo", selected.hostArgs, {
    cwd: root,
    env,
    stdio: "inherit",
  });
  if (build.status !== 0) throw new Error("后台程序构建失败");
  const directory = selected.sidecarDirectory;
  mkdirSync(directory, { recursive: true });
  const suffix = process.platform === "win32" ? ".exe" : "";
  const targetDirectory = env.CARGO_TARGET_DIR;
  const output = resolve(
    directory,
    `stagemaster-execution-host-${triple}${suffix}`,
  );
  copyFileSync(
    resolve(
      targetDirectory,
      `${selected.profile}/stagemaster-execution-host${suffix}`,
    ),
    output,
  );
  return {
    profile: selected.profile,
    targetDirectory,
    output,
    args: selected.hostArgs,
    status: build.status,
    triple,
  };
}
if (process.argv[1] === fileURLToPath(import.meta.url)) prepareHost();
