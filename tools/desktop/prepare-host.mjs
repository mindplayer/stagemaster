import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

export function prepareHost() {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const env = {
    ...process.env,
    CARGO_HOME: resolve(root, "tmp/cargo-home"),
    TMPDIR: resolve(root, "tmp"),
  };
  const info = spawnSync("rustc", ["-vV"], { encoding: "utf8", env });
  if (info.status !== 0) throw new Error("无法确定 Rust 目标平台");
  const triple = info.stdout.match(/^host: (\S+)$/m)?.[1];
  if (!triple) throw new Error("缺少 Rust 目标平台");
  const build = spawnSync(
    "cargo",
    ["build", "-p", "stagemaster-execution-host", "--locked", "--offline"],
    { cwd: root, env, stdio: "inherit" },
  );
  if (build.status !== 0) throw new Error("后台程序构建失败");
  const directory = resolve(root, "tmp/desktop-bin");
  mkdirSync(directory, { recursive: true });
  const suffix = process.platform === "win32" ? ".exe" : "";
  copyFileSync(
    resolve(root, `target/debug/stagemaster-execution-host${suffix}`),
    resolve(directory, `stagemaster-execution-host-${triple}${suffix}`),
  );
}
if (process.argv[1] === fileURLToPath(import.meta.url)) prepareHost();
