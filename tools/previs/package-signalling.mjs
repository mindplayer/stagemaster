import {
  chmodSync,
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  writeFileSync,
} from "node:fs";
import { basename, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { runCommand } from "./package-renderer.mjs";
import { writeNotices } from "./signalling-notices.mjs";
import {
  fileHash,
  fileInventory,
  lockedPackages,
  packageLicenses,
  qualifyNode,
  readJson,
} from "./signalling-package-files.mjs";
import {
  signallingEnvironment,
  signallingPackagePlan,
  signallingTests,
  successfulSignallingReport,
} from "./signalling-package-plan.mjs";

async function qualify(plan, bundle, name) {
  const saved = join(plan.temporary, `qualification-${name}`);
  mkdirSync(saved);
  try {
    for (const file of signallingTests)
      copyFileSync(join(plan.root, "tools/previs", file), join(bundle, file));
    const command = {
      program: join(bundle, "node"),
      args: [
        "--test",
        "--test-reporter=tap",
        ...signallingTests.map((file) => join(bundle, file)),
      ],
      log: join(plan.logs, `${name}-tests.tap`),
    };
    await runCommand(command, plan);
    return {
      command,
      tests: successfulSignallingReport(readFileSync(command.log, "utf8")),
    };
  } finally {
    for (const file of signallingTests)
      if (existsSync(join(bundle, file)))
        renameSync(join(bundle, file), join(saved, file));
  }
}

export async function packageSignalling() {
  if (process.argv.length > 2) throw new Error("组件组装不接受额外参数");
  if (process.version !== "v24.17.0")
    throw new Error("本增量只复用已验证的 Node 24.17.0，不自动安装其他运行时");
  if (process.platform !== "darwin" || process.arch !== "arm64")
    throw new Error("本组件组装只支持 Mac ARM64");
  const root = fileURLToPath(new URL("../../", import.meta.url));
  mkdirSync(join(root, "tmp"), { recursive: true });
  const temporary = mkdtempSync(join(root, "tmp/previs-signalling-"));
  const plan = signallingPackagePlan(
    root,
    process.execPath,
    basename(temporary),
  );
  for (const folder of [
    plan.archive,
    plan.bundle,
    plan.logs,
    plan.env.NODE_COMPILE_CACHE,
  ])
    mkdirSync(folder, { recursive: true });
  writeFileSync(plan.config, "", { flag: "wx" });
  writeFileSync(plan.globalConfig, "", { flag: "wx" });
  const evidence = {
    status: "running",
    plan,
    startedAt: new Date().toISOString(),
  };
  const record = () =>
    writeFileSync(
      join(plan.archive, "build-record.json"),
      `${JSON.stringify(evidence, null, 2)}\n`,
    );
  record();
  // Qualify only owned processes, without ambient npm or Node loader configuration.
  const environment = signallingEnvironment(process.env);
  for (const key of Object.keys(process.env))
    if (!(key in environment) && !(key in plan.env)) plan.env[key] = undefined;
  try {
    if (!existsSync(plan.license) || !existsSync(plan.npm))
      throw new Error("已安装 Node 缺少完整许可或 npm 工具");
    const source = join(root, "tools/previs");
    const packages = lockedPackages(
      readJson(join(source, "package.json")),
      readJson(join(source, "package-lock.json")),
    );
    evidence.sourceFiles = [];
    for (const file of [
      "package.json",
      "package-lock.json",
      "signalling.mjs",
    ]) {
      copyFileSync(join(source, file), join(plan.bundle, file));
      evidence.sourceFiles.push({
        file,
        sha256: await fileHash(join(source, file)),
      });
    }
    copyFileSync(plan.node, join(plan.bundle, "node"));
    chmodSync(join(plan.bundle, "node"), 0o755);
    mkdirSync(join(plan.bundle, "licenses"));
    copyFileSync(plan.license, join(plan.bundle, "licenses/node-LICENSE"));
    evidence.node = qualifyNode(root, join(plan.bundle, "node"));
    evidence.node.version = process.version;
    evidence.node.sha256 = await fileHash(plan.node);
    if ((await fileHash(join(plan.bundle, "node"))) !== evidence.node.sha256)
      throw new Error("Node 复制字节不等价");
    await runCommand(plan.install, plan);
    evidence.licenses = packageLicenses(plan.bundle, packages);
    writeFileSync(
      join(plan.bundle, "licenses/npm-packages.json"),
      `${JSON.stringify(evidence.licenses, null, 2)}\n`,
    );
    evidence.notices = writeNotices(plan.bundle).report;
    evidence.original = await qualify(plan, plan.bundle, "original");
    const moved = join(plan.temporary, "中文 移位/previs");
    cpSync(plan.bundle, moved, { recursive: true, verbatimSymlinks: true });
    evidence.moved = await qualify(plan, moved, "moved");
    evidence.files = await fileInventory(plan.bundle);
    if (
      JSON.stringify(await fileInventory(moved)) !==
      JSON.stringify(evidence.files)
    )
      throw new Error("信令组件移位内容不等价");
    evidence.status = "signalling-component-verified";
    evidence.customerPackageVerified = false;
    console.log(
      `Node／信令组件通过：${plan.bundle}（未代表 UE／桌面／客户安装验收）`,
    );
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
  packageSignalling().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
