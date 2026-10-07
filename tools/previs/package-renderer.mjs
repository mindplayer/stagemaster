import {
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
import { packagingPlan } from "./packaging-plan.mjs";
import {
  rendererPackageOptions,
  checkRendererPackage,
} from "./renderer-package-check.mjs";
import {
  metalToolchain,
  requirePlatformTempPermission,
} from "./packaging-tools.mjs";
import { inspectMacBundle } from "./mac-bundle-inspection.mjs";
import { runCommand } from "./packaging-process.mjs";
export { runCommand } from "./packaging-process.mjs";

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
  const options = rendererPackageOptions(process.argv.slice(2));
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
    options,
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
    evidence.staticDependencies = inspectMacBundle(root, program);
    await checkRendererPackage(evidence, program, options);
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
