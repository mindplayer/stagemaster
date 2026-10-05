import { execFileSync } from "node:child_process";
import { existsSync, realpathSync, statSync } from "node:fs";
import {
  dirname,
  isAbsolute,
  join,
  normalize,
  relative,
  resolve,
  sep,
} from "node:path";
import {
  parseMacLoadCommands,
  compareMacVersions,
} from "./mac-load-commands.mjs";
import { withoutLoaderOverrides } from "./packaging-tools.mjs";

const inside = (directory, file) => {
  const value = relative(directory, file);
  return (
    value === "" ||
    (!value.startsWith(`..${sep}`) && value !== ".." && !isAbsolute(value))
  );
};
const system = (name) => {
  const path = normalize(name);
  return path.startsWith("/System/Library/") || path.startsWith("/usr/lib/");
};

export function readMacImage(root, file) {
  const options = {
    cwd: root,
    encoding: "utf8",
    timeout: 10000,
    maxBuffer: 4 * 1024 * 1024,
    env: withoutLoaderOverrides({ ...process.env, TMPDIR: join(root, "tmp") }),
    stdio: ["ignore", "pipe", "pipe"],
  };
  const architectureOutput = execFileSync(
    "/usr/bin/lipo",
    ["-archs", file],
    options,
  );
  const architectures = architectureOutput.trim().split(/\s+/);
  if (!architectures.includes("arm64"))
    throw new Error(`缺少 ARM64 镜像：${file}`);
  const loadCommands = execFileSync(
    "/usr/bin/otool",
    ["-arch", "arm64", "-l", file],
    options,
  );
  return {
    ...parseMacLoadCommands(loadCommands),
    architectures,
    architectureOutput,
    loadCommands,
  };
}

// This checks linked images only. dlopen, GPU, sandbox and customer execution remain gates.
export function inspectMacBundle(
  root,
  program,
  {
    readImage = (file) => readMacImage(root, file),
    maxImages = 128,
    maxDependencies = 4096,
  } = {},
) {
  for (const [limit, ceiling] of [
    [maxImages, 128],
    [maxDependencies, 4096],
  ])
    if (!Number.isSafeInteger(limit) || limit < 1 || limit > ceiling)
      throw new Error("Mach-O 依赖检查预算无效");
  const project = realpathSync(root);
  const input = resolve(root, program);
  const bundlePath = dirname(dirname(dirname(input)));
  if (
    input !== join(bundlePath, "Contents/MacOS/StageMasterPreview") ||
    !bundlePath.endsWith("/StageMasterPreview.app")
  )
    throw new Error("必须检查独立预演 Game 程序");
  const bundle = realpathSync(bundlePath);
  const executable = realpathSync(input);
  if (!inside(project, bundle) || !inside(bundle, executable))
    throw new Error("预演程序或符号链接越出项目／应用包");
  const images = [];
  const edges = [];
  const seen = new Set();
  let minimumMacOS = "0.0";
  const expand = (name, loader) => {
    if (name === "@loader_path" || name.startsWith("@loader_path/"))
      return resolve(dirname(loader), `.${name.slice("@loader_path".length)}`);
    if (name === "@executable_path" || name.startsWith("@executable_path/"))
      return resolve(
        dirname(executable),
        `.${name.slice("@executable_path".length)}`,
      );
    return isAbsolute(name) ? normalize(name) : undefined;
  };
  const find = (name, loader, paths) => {
    let candidates;
    if (name.startsWith("@rpath/"))
      candidates = paths.map((path) => resolve(path, name.slice(7)));
    else {
      // Absolute non-system install names are not relocatable even when inside today's app.
      if (isAbsolute(name))
        throw new Error(`非系统依赖使用固定绝对路径：${name}`);
      const path = expand(name, loader);
      candidates = path ? [path] : [];
    }
    for (const path of candidates) {
      if (!inside(bundle, path) || !existsSync(path)) continue;
      const actual = realpathSync(path);
      if (!inside(bundle, actual))
        throw new Error(`动态库符号链接越出应用包：${path}`);
      if (!statSync(actual).isFile())
        throw new Error(`动态库不是文件：${path}`);
      return actual;
    }
    throw new Error(`应用包缺少可解析依赖：${name}（加载者 ${loader}）`);
  };
  const visit = (file, inherited) => {
    if (seen.has(file)) return;
    if (images.length >= maxImages) throw new Error("Mach-O 镜像预算超限");
    if (!statSync(file).isFile()) throw new Error(`镜像不是文件：${file}`);
    const image = readImage(file);
    if (!image.architectures.includes("arm64"))
      throw new Error(`缺少 ARM64 镜像：${file}`);
    if (
      (file === executable && !image.executable) ||
      (file !== executable && !image.dylib)
    )
      throw new Error(`Mach-O 镜像角色不符：${file}`);
    if (compareMacVersions(image.minimumMacOS, minimumMacOS) > 0)
      minimumMacOS = image.minimumMacOS;
    const paths = [
      ...new Set([
        ...image.rpaths.map((path) => expand(path, file)).filter(Boolean),
        ...inherited,
      ]),
    ];
    seen.add(file);
    images.push({ file, ...image });
    for (const dependency of image.dependencies) {
      if (edges.length >= maxDependencies)
        throw new Error("Mach-O 依赖预算超限");
      if (isAbsolute(dependency.name) && system(dependency.name))
        edges.push({ from: file, ...dependency, source: "macOS" });
      else {
        const target = find(dependency.name, file, paths);
        if (target === executable)
          throw new Error(`动态库依赖指向主程序：${dependency.name}`);
        edges.push({ from: file, ...dependency, source: "bundle", target });
        visit(target, paths);
      }
    }
  };
  visit(executable, []);
  return {
    scope: "static-linked-closure-only",
    bundle,
    program: executable,
    minimumMacOS,
    images,
    edges,
  };
}
