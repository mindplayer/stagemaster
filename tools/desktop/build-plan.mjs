import { isAbsolute, join, relative, resolve, sep } from "node:path";

export function desktopBuildPlan(root, command, instance) {
  if (!["dev", "build", "build-internal-release"].includes(command))
    throw new Error("桌面构建命令无效");
  const project = resolve(root),
    internal = command === "build-internal-release";
  if (!internal)
    return {
      root: project,
      command,
      profile: "debug",
      sidecarDirectory: join(project, "tmp/desktop-bin"),
      hostArgs: [
        "build",
        "-p",
        "stagemaster-execution-host",
        "--features",
        "audio",
        "--locked",
        "--offline",
      ],
      cliArgs: [
        command === "dev" ? "dev" : "build",
        ...(command === "dev" ? [] : ["--debug", "--bundles", "app"]),
      ],
    };
  if (
    typeof instance !== "string" ||
    instance.length > 64 ||
    !/^desktop-release-[a-zA-Z0-9]+$/.test(instance)
  )
    throw new Error("内部发布验收实例名称无效");
  const temporary = join(project, "tmp", instance),
    identifier = `cn.stagemaster.acceptance.${instance.toLowerCase()}`;
  const sidecarDirectory = join(temporary, "sidecar"),
    configFile = join(temporary, "tauri.internal.json");
  const config = {
    identifier,
    productName: "舞台大师 内部发布验收",
    bundle: {
      externalBin: [
        relative(
          join(project, "apps/desktop"),
          join(sidecarDirectory, "stagemaster-execution-host"),
        ),
      ],
    },
  };
  return {
    root: project,
    command,
    instance,
    profile: "release",
    identifier,
    temporary,
    target: join(project, "tmp/desktop-release-target"),
    sidecarDirectory,
    archive: join(project, "data/DESKTOP-005", instance),
    logs: join(project, "logs/DESKTOP-005", instance),
    configFile,
    config,
    hostArgs: [
      "build",
      "--release",
      "-p",
      "stagemaster-execution-host",
      "--features",
      "audio",
      "--locked",
      "--offline",
    ],
    cliArgs: [
      "build",
      "--features",
      "internal-acceptance",
      "--bundles",
      "app",
      "--no-sign",
      "--config",
      configFile,
      "--",
      "--locked",
      "--offline",
    ],
    launchEnvironment: {
      STAGEMASTER_ACCEPTANCE_INSTANCE: instance,
      TMPDIR: join(project, "tmp", `desktop-${instance}/tmp`),
    },
  };
}

export function projectTarget(root, target) {
  const directory = resolve(root, target ?? "target"),
    inside = relative(resolve(root), directory);
  if (
    !inside ||
    isAbsolute(inside) ||
    inside === ".." ||
    inside.startsWith(`..${sep}`)
  )
    throw new Error("桌面构建目标必须在项目内且不能是项目根");
  return directory;
}

export function internalEnvironment(plan, original) {
  const env = Object.fromEntries(
    Object.entries(original).filter(
      ([key]) =>
        !key.startsWith("DYLD_") &&
        !key.startsWith("NODE_") &&
        !key.startsWith("APPLE_") &&
        !key.startsWith("TAURI_SIGNING_") &&
        key !== "CFFIXED_USER_HOME",
    ),
  );
  return {
    ...env,
    CARGO_HOME: join(plan.root, "tmp/cargo-home"),
    CARGO_TARGET_DIR: plan.target,
    TMPDIR: plan.temporary,
    npm_config_cache: join(plan.root, "tmp/npm-cache"),
    NODE_DISABLE_COMPILE_CACHE: "1",
  };
}

export function hostEnvironment(plan, original) {
  return {
    ...original,
    CARGO_HOME: join(plan.root, "tmp/cargo-home"),
    CARGO_TARGET_DIR: projectTarget(plan.root, original.CARGO_TARGET_DIR),
    TMPDIR:
      plan.profile === "release" ? plan.temporary : join(plan.root, "tmp"),
  };
}
