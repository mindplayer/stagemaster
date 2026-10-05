import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, isAbsolute, join } from "node:path";

export function withoutLoaderOverrides(environment) {
  return Object.fromEntries(
    Object.entries(environment).filter(([key]) => !key.startsWith("DYLD_")),
  );
}

// This switch records the caller's explicit permission; it is not itself authorization.
// macOS Foundation/Xcode can ignore TMPDIR for OS-managed temporary script files.
export function requirePlatformTempPermission(value) {
  if (value !== "1")
    throw new Error(
      "Xcode 会使用系统自管理临时文件；用户允许该例外后才能设置 STAGEMASTER_ALLOW_PLATFORM_TEMP=1 打包",
    );
}

// Resolve the installed Apple component before isolating NSApplicationSupportDirectory.
// xcrun --no-cache does not create/reuse the user's discovery cache or install anything.
export function metalToolchain(
  root,
  execute = spawnSync,
  present = existsSync,
) {
  const env = { ...process.env, TMPDIR: join(root, "tmp") };
  delete env.CFFIXED_USER_HOME;
  const result = execute(
    "/usr/bin/xcrun",
    ["--no-cache", "--find", "metallib"],
    {
      cwd: root,
      env,
      encoding: "utf8",
      timeout: 10000,
    },
  );
  if (result.error || result.status !== 0)
    throw new Error(
      "未找到已安装的 Metal 工具链；需核对 Xcode 组件，不能跳过着色器编译",
    );
  const library = result.stdout.trim();
  const bin = dirname(library);
  if (!isAbsolute(library) || !present(library) || !present(join(bin, "metal")))
    throw new Error("Metal 工具链缺少实际 metal／metallib 程序");
  return { library, bin };
}
