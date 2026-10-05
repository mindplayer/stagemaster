import { prepareHost } from "./prepare-host.mjs";
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { desktopBuildPlan } from "./build-plan.mjs";
import { prepareWorkspaceFolders } from "./workspace-folders.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const command = process.argv[2] ?? "dev";
if (process.argv.length > 3) throw new Error("桌面构建不接受额外参数");
desktopBuildPlan(
  root,
  command,
  command === "build-internal-release" ? "desktop-release-Validate" : undefined,
);
prepareWorkspaceFolders(root);
if (command === "build-internal-release") {
  const instance = basename(mkdtempSync(join(root, "tmp/desktop-release-")));
  const { buildInternalRelease } = await import("./release-build.mjs");
  try {
    await buildInternalRelease(root, instance);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
} else {
  prepareHost();
  const cli = resolve(
    root,
    "apps/ui-prototype/node_modules/@tauri-apps/cli/tauri.js",
  );
  const child = spawn(
    process.execPath,
    [
      cli,
      command,
      ...(command === "build" ? ["--debug", "--bundles", "app"] : []),
    ],
    {
      cwd: resolve(root, "apps/desktop"),
      env: {
        ...process.env,
        STAGEMASTER_NODE_BINARY: process.execPath,
        CARGO_HOME: resolve(root, "tmp/cargo-home"),
        TMPDIR: resolve(root, "tmp"),
        npm_config_cache: resolve(root, "tmp/npm-cache"),
      },
      stdio: "inherit",
    },
  );
  child.on("error", (error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
  child.on("exit", (code) => {
    process.exitCode = code ?? 1;
  });
}
