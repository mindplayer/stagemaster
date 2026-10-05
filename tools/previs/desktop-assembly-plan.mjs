import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { developmentFileEntitlements } from "./file-access-plan.mjs";

function input(project, value) {
  const file = resolve(project, value),
    inside = relative(project, file);
  if (
    !inside ||
    inside === ".." ||
    inside.startsWith(`..${sep}`) ||
    isAbsolute(inside)
  )
    throw new Error("组装来源必须在项目内且不能是项目根");
  return file;
}

export function desktopAssemblyPlan(
  root,
  sources,
  id,
  platform = process.platform,
  arch = process.arch,
) {
  if (platform !== "darwin" || arch !== "arm64")
    throw new Error("仅支持内部 Mac ARM64 Development 组装");
  if (!/^previs-desktop-[a-zA-Z0-9]+$/.test(id))
    throw new Error("桌面组装实例名称无效");
  if (
    !Array.isArray(sources) ||
    sources.length !== 3 ||
    sources.some((value) => typeof value !== "string" || !value)
  )
    throw new Error("需明确三个来源：桌面包、Game 主程序、信令组件");
  const project = resolve(root);
  const [desktop, game, signalling] = sources.map((value) =>
    input(project, value),
  );
  const gameBundle = dirname(dirname(dirname(game)));
  if (
    !desktop.endsWith(".app") ||
    game !== join(gameBundle, "Contents/MacOS/StageMasterPreview") ||
    !gameBundle.endsWith("/StageMasterPreview.app")
  )
    throw new Error("来源需桌面 .app 和独立 Game 主程序");
  const temporary = join(project, "tmp", id),
    archive = join(project, "data/PREVIS-007", id);
  const instance = join(project, "tmp", `desktop-${id}`);
  const runtime = {
    user: join(instance, "previs/user"),
    cache: join(instance, "previs/cache"),
    temporary: join(instance, "tmp/previs"),
    logs: join(instance, "logs/previs"),
    report: join(instance, "tmp/previs/report"),
  };
  const bundle = join(archive, "舞台大师 内部验收.app");
  const component = join(bundle, "Contents/Resources/previs");
  return {
    root: project,
    id,
    desktop,
    game,
    gameBundle,
    signalling,
    temporary,
    archive,
    instance,
    runtime,
    bundle,
    component,
    logs: join(project, "logs/PREVIS-007", id),
    copiedGame: join(component, "StageMasterPreview.app"),
    entitlementFile: join(temporary, "development.entitlements"),
    bundleId: `cn.stagemaster.acceptance.${id.toLowerCase()}`,
    minimumMacOS: "14.0",
    env: { TMPDIR: temporary },
    launchEnvironment: {
      STAGEMASTER_ACCEPTANCE_INSTANCE: id,
      TMPDIR: join(instance, "tmp"),
    },
  };
}

export function desktopFileEntitlements(plan, base) {
  const expected = desktopAssemblyPlan(
    plan.root,
    [plan.desktop, plan.game, plan.signalling],
    plan.id,
  );
  const keys = ["user", "cache", "temporary", "logs", "report"];
  if (keys.some((key) => plan.runtime[key] !== expected.runtime[key]))
    throw new Error("文件资格不属于准确桌面实例布局");
  return developmentFileEntitlements(
    base,
    keys.map((key) => plan.runtime[key]),
  );
}
