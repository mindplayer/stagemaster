import { prepareHost } from "./prepare-host.mjs";
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../", import.meta.url));
const command = process.argv[2] ?? "dev";
if (!["dev", "build"].includes(command)) throw new Error("仅支持 dev 或 build");
for (const directory of ["tmp", "tmp/cargo-home", "logs/DESKTOP-001"])
  mkdirSync(resolve(root, directory), { recursive: true });
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
