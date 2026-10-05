import { spawnSync } from "node:child_process";
import { lstatSync, realpathSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { executableName, plainAncestors } from "./desktop-assembly-files.mjs";
import { readMacImage, inspectMacBundle } from "./mac-bundle-inspection.mjs";
import { withoutLoaderOverrides } from "./packaging-tools.mjs";
import { parseStrict } from "../project-format/check.mjs";
import {
  parseDistributionSignature,
  developerIdRequirement,
} from "./distribution-signature.mjs";
import { assessDistributionPrerequisites } from "./distribution-policy.mjs";

export function inspectDistribution(
  root,
  args,
  {
    execute = spawnSync,
    readImage = (file) => readMacImage(root, file),
    inspectDependencies = (program) => inspectMacBundle(root, program),
    os = process.platform,
    architecture = process.arch,
  } = {},
) {
  root = resolve(root);
  if (os !== "darwin" || architecture !== "arm64")
    throw new Error("此发行前检查仅支持Mac ARM64");
  if (
    !Array.isArray(args) ||
    args.length !== 1 ||
    typeof args[0] !== "string" ||
    /[\x00-\x1f\x7f]/.test(args[0])
  )
    throw new Error("请只指定一份项目内桌面.app路径");
  const bundle = resolve(root, args[0]),
    within = relative(root, bundle);
  if (
    !bundle.endsWith(".app") ||
    !within ||
    within === ".." ||
    within.startsWith(`..${sep}`) ||
    isAbsolute(within)
  )
    throw new Error("必须检查项目内桌面.app，不能检查根目录或包外路径");
  plainAncestors(dirname(bundle));
  let bundleInfo;
  try {
    bundleInfo = lstatSync(bundle);
  } catch (cause) {
    throw new Error("桌面包不存在或无法读取", { cause });
  }
  if (
    !bundleInfo.isDirectory() ||
    realpathSync(bundle) !== bundle ||
    realpathSync(root) !== root
  )
    throw new Error("桌面包或项目根不是规范目录");
  const required = (file) => {
    for (const entry of ancestors(file)) {
      let stat;
      try {
        stat = lstatSync(entry);
      } catch (cause) {
        throw new Error(`程序／清单缺件或无法读取：${relative(bundle, file)}`, {
          cause,
        });
      }
      if (
        stat.isSymbolicLink() ||
        (entry === file
          ? !stat.isFile() || stat.size === 0
          : !stat.isDirectory())
      )
        throw new Error(
          `程序／清单缺件或路径经过链接：${relative(bundle, file)}`,
        );
    }
    return file;
  };
  function* ancestors(file) {
    for (let entry = file; entry !== bundle; entry = dirname(entry)) {
      if (
        relative(bundle, entry).startsWith(`..${sep}`) ||
        entry === dirname(entry)
      )
        throw new Error("程序／清单路径越出桌面包");
      yield entry;
    }
  }
  const options = {
    cwd: root,
    encoding: "utf8",
    timeout: 10000,
    maxBuffer: 4 * 1024 * 1024,
    env: withoutLoaderOverrides({
      ...process.env,
      TMPDIR: join(root, "tmp"),
      LC_ALL: "C",
    }),
  };
  const commands = [];
  const command = (program, arguments_, input) => {
    const result = execute(program, arguments_, {
      ...options,
      ...(input === undefined ? {} : { input }),
    });
    if (result.error || result.signal || !Number.isSafeInteger(result.status))
      throw new Error(`发行前只读工具未完成：${program}`);
    const response = {
      program,
      args: arguments_,
      status: result.status,
      stdout: result.stdout ?? "",
      stderr: result.stderr ?? "",
    };
    commands.push(response);
    return response;
  };
  const info = required(join(bundle, "Contents/Info.plist"));
  const plist = (key) => {
    const result = command("/usr/libexec/PlistBuddy", [
      "-c",
      `Print :${key}`,
      info,
    ]);
    if (result.status !== 0) throw new Error(`桌面清单缺少${key}`);
    return result.stdout.trim();
  };
  const executable = executableName(plist("CFBundleExecutable"));
  const bundleId = plist("CFBundleIdentifier"),
    minimumMacOS = plist("LSMinimumSystemVersion");
  if (!bundleId || /\s/.test(bundleId)) throw new Error("桌面包标识无效");
  const component = join(bundle, "Contents/Resources/previs"),
    gameBundle = join(component, "StageMasterPreview.app");
  const programs = [
    ["desktop", join(bundle, "Contents/MacOS", executable), bundle],
    [
      "execution-host",
      join(bundle, "Contents/MacOS/stagemaster-execution-host"),
    ],
    ["node", join(component, "node")],
    ["game", join(gameBundle, "Contents/MacOS/StageMasterPreview"), gameBundle],
  ];
  for (const [, program] of programs) {
    required(program);
    if ((lstatSync(program).mode & 0o111) === 0)
      throw new Error(`程序入口不可执行：${relative(bundle, program)}`);
  }
  required(join(gameBundle, "Contents/Info.plist"));
  required(
    join(
      gameBundle,
      "Contents/UE/StageMasterPreview/Content/Paks/StageMasterPreview-Mac.pak",
    ),
  );
  const images = programs.map(([role, program, signedBundle]) => {
    const target = signedBundle ?? program;
    const image = readImage(program);
    const strict = command("/usr/bin/codesign", [
      "--verify",
      "--deep",
      "--strict",
      target,
    ]);
    const developerId = command("/usr/bin/codesign", [
      "--verify",
      "--strict",
      "-R",
      developerIdRequirement,
      target,
    ]);
    const display = command("/usr/bin/codesign", [
      "--display",
      "--verbose=4",
      target,
    ]);
    const signature =
      display.status === 0
        ? parseDistributionSignature(display.stdout + display.stderr)
        : null;
    const rights = command("/usr/bin/codesign", [
      "-d",
      "--entitlements",
      "-",
      "--xml",
      target,
    ]);
    let values = null;
    if (rights.status === 0) {
      if (!rights.stdout.trim()) values = {};
      else {
        const converted = command(
          "/usr/bin/plutil",
          ["-convert", "json", "-o", "-", "--", "-"],
          rights.stdout,
        );
        if (converted.status !== 0) throw new Error(`资格XML无法解析：${role}`);
        values = parseStrict(converted.stdout);
      }
    }
    return {
      role,
      program,
      minimumMacOS: image.minimumMacOS,
      architectures: image.architectures,
      executable: image.executable,
      image,
      strict,
      developerId,
      signature,
      entitlements: { status: rights.status, values },
    };
  });
  const dependencies = inspectDependencies(programs[3][1]);
  const evidence = {
    bundle,
    bundleId,
    minimumMacOS,
    closureMinimumMacOS: dependencies.minimumMacOS,
    images,
    dependencies,
    commands,
  };
  const result = assessDistributionPrerequisites(evidence);
  if (
    /^cn\.stagemaster\.acceptance\./.test(bundleId) &&
    !result.issues.some(
      (issue) => issue.code === "internal-id" && issue.role === "desktop",
    )
  ) {
    result.issues.push({
      role: "desktop",
      program: programs[0][1],
      code: "internal-id",
      message: "桌面清单仍是内部验收身份",
    });
    result.status = "blocked";
  }
  return { ...result, evidence, readOnly: true, customerLaunchVerified: false };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href)
  try {
    const report = inspectDistribution(
      fileURLToPath(new URL("../../", import.meta.url)),
      process.argv.slice(2),
    );
    console.log(JSON.stringify(report, null, 2));
    if (report.status !== "static-prerequisites-passed") process.exitCode = 1;
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
