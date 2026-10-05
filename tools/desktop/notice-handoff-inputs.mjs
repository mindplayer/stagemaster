import assert from "node:assert/strict";
import { lstatSync } from "node:fs";
import { dirname, relative } from "node:path";
import {
  fileHash,
  fileInventory,
} from "../previs/signalling-package-files.mjs";
import { plainAncestors } from "../previs/desktop-assembly-files.mjs";
import {
  digest,
  handoffBytes,
  handoffJson,
  handoffPath,
  handoffRoot,
  handoffTarget,
} from "./notice-handoff-paths.mjs";
import { permissionTree } from "./notice-handoff-files.mjs";
const dependencies = [
  "Cargo.toml",
  "Cargo.lock",
  "apps/desktop/Cargo.toml",
  "apps/execution-host/Cargo.toml",
  "apps/ui-prototype/package.json",
  "apps/ui-prototype/package-lock.json",
];
const check = (ok, message) => {
  if (!ok) throw new Error(message);
};
async function sourceHashes(root, hashes) {
  check(
    hashes &&
      typeof hashes === "object" &&
      !Array.isArray(hashes) &&
      Object.keys(hashes).length > 0 &&
      Object.keys(hashes).length <= 4096,
    "产品来源哈希预算无效",
  );
  for (const [key, sha256] of Object.entries(hashes)) {
    check(
      digest(sha256) &&
        /^(?:Cargo\.(?:toml|lock)|(?:apps|crates|tools)\/.+)$/.test(key),
      "产品来源路径或摘要无效",
    );
    const file = handoffPath(root, key);
    plainAncestors(dirname(file));
    check(
      lstatSync(file).isFile() && !lstatSync(file).isSymbolicLink(),
      "产品来源链接拒绝",
    );
    check((await fileHash(file)) === sha256, "产品来源文件已变化：" + key);
  }
}
function materialPackages(report) {
  check(
    Array.isArray(report.packages) &&
      report.packages.length > 0 &&
      report.packages.length <= 1024,
    "材料包预算无效",
  );
  check(
    Array.isArray(report.missingPackages) &&
      report.missingPackages.length <= 1024,
    "材料缺项预算无效",
  );
  const identities = new Set();
  let notInstalled = 0;
  for (const p of report.packages) {
    const name = p.ecosystem === "cargo" ? p.name : p.path;
    check(
      ["cargo", "npm"].includes(p.ecosystem) &&
        typeof name === "string" &&
        typeof p.version === "string" &&
        p.version,
      "材料包身份无效",
    );
    const key = p.ecosystem + "/" + name + "@" + p.version;
    check(!identities.has(key), "材料包身份重复");
    identities.add(key);
    check(
      ["collected", "not-installed"].includes(p.materialStatus),
      "已安装材料缺项或未知状态拒绝",
    );
    check(
      Array.isArray(p.notices) && p.notices.length <= 16,
      "材料单包原文预算无效",
    );
    if (p.materialStatus === "not-installed") {
      check(
        p.optional === true && p.notices.length === 0,
        "未安装资料不是可选缺项",
      );
      notInstalled++;
    } else {
      check(p.notices.length > 0, "已安装包缺原文");
      for (const notice of p.notices)
        check(
          typeof notice.file === "string" &&
            digest(notice.sha256) &&
            Number.isSafeInteger(notice.bytes) &&
            notice.bytes > 0 &&
            notice.bytes <= 512 * 1024,
          "材料单原文预算或摘要无效",
        );
    }
  }
  const missing = report.packages.filter(
    (p) => p.materialStatus === "not-installed",
  );
  const key = (p) =>
    p.ecosystem +
    "/" +
    (p.ecosystem === "cargo" ? p.name : p.path) +
    "@" +
    p.version;
  check(
    report.missingPackages.every(
      (p) =>
        p.status === "not-installed" && typeof p.name === "string" && p.name,
    ),
    "材料已安装缺项不得隐去",
  );
  assert.deepEqual(
    report.missingPackages
      .map((p) => p.ecosystem + "/" + p.name + "@" + p.version)
      .sort(),
    missing.map(key).sort(),
    "材料可选缺项清单不一致",
  );
  return {
    installedMissing: 0,
    notInstalled,
    packageCount: report.packages.length,
  };
}
export async function readHandoffInputs(
  project,
  referenceFile,
  materialDirectory,
  destination,
) {
  const root = handoffRoot(project),
    output = handoffTarget(root, destination);
  const refInput = handoffJson(root, referenceFile),
    ref = refInput.value;
  check(
    ref &&
      /^[a-f0-9]{40}$/.test(ref.sourceCommit) &&
      /^desktop-release-[A-Za-z0-9]{6,24}$/.test(ref.id) &&
      ref.customerPackageVerified === false &&
      ref.sourceProfile === "optimized-desktop-with-development-renderer",
    "候选来源身份或资格无效",
  );
  const qualification = ref.sourceQualification;
  check(
    qualification &&
      qualification.sourceCommit === ref.sourceCommit &&
      qualification.instance === ref.id &&
      qualification.profile === "release" &&
      qualification.customerReleaseQualified === false &&
      digest(qualification.recordSha256),
    "候选来源资格绑定无效",
  );
  const evidenceInput = handoffJson(root, qualification.record),
    evidence = evidenceInput.value;
  check(
    evidenceInput.sha256 === qualification.recordSha256 &&
      evidence.sourceCommit === ref.sourceCommit &&
      evidence.buildExit === 0 &&
      evidence.hostProfile === "release" &&
      evidence.customerReleaseQualified === false,
    "优化来源证据变化或未通过",
  );
  const rawBuildInput = handoffJson(root, evidence.rawRecord),
    build = rawBuildInput.value;
  check(
    rawBuildInput.sha256 === evidence.recordSha256 &&
      rawBuildInput.path === handoffPath(root, qualification.rawRecord) &&
      build.status === "isolated-release-built" &&
      build.commandResult?.code === 0 &&
      build.customerReleaseQualified === false &&
      build.plan?.instance === ref.id &&
      build.bundle === evidence.bundle,
    "原构建来源证据变化或身份不匹配",
  );
  await sourceHashes(root, evidence.sourceHashes);
  await sourceHashes(root, evidence.productSourceHashes);
  const assemblyInput = handoffJson(root, ref.rawRecord),
    assembly = assemblyInput.value;
  check(
    relative(root, assemblyInput.path) ===
      "data/PREVIS-007/" + ref.id + "/assembly-record.json" &&
      assembly.status === "development-assembled" &&
      assembly.customerPackageVerified === false &&
      assembly.plan?.root === root &&
      assembly.plan.id === ref.id &&
      assembly.plan.desktop === evidence.bundle &&
      assembly.plan.bundle === ref.bundle,
    "候选组装来源身份无效",
  );
  assert.deepEqual(
    assembly.original?.internalRelease,
    qualification,
    "组装与引用来源资格不一致",
  );
  const bundle = handoffPath(root, ref.bundle);
  plainAncestors(bundle);
  check(
    bundle.endsWith(".app") &&
      relative(root, bundle).startsWith("data/PREVIS-007/" + ref.id + "/") &&
      lstatSync(bundle).isDirectory(),
    "候选包来源路径无效",
  );
  permissionTree(bundle);
  const files = await fileInventory(bundle);
  check(
    files.length > 0 &&
      files.length <= 4096 &&
      files.length === ref.assembledFiles,
    "候选文件清单预算无效",
  );
  assert.deepEqual(files, assembly.assembledFiles, "候选文件字节清单已变化");
  const materialsRoot = handoffPath(root, materialDirectory);
  plainAncestors(materialsRoot);
  const reportInput = handoffJson(
      root,
      relative(root, materialsRoot) + "/licenses/notices.json",
    ),
    report = reportInput.value;
  check(
    report.format === "stagemaster.desktop-notice-materials" &&
      report.version === 1 &&
      report.target === "aarch64-apple-darwin" &&
      report.scope ===
        "conservative-non-dev-source-materials-including-build-and-optional-packages" &&
      report.commercialReleaseApproved === false &&
      report.reviewStillRequired === true &&
      report.bundledBinaryInventory === false,
    "材料不是未批准保守源码集",
  );
  for (const key of dependencies)
    check(
      digest(report.sourceHashes?.[key]) &&
        report.sourceHashes[key] === evidence.productSourceHashes[key],
      "材料依赖来源与候选不对应：" + key,
    );
  check(
    report.textFile === "licenses/THIRD-PARTY-NOTICES.txt",
    "材料全文路径无效",
  );
  const textInput = handoffBytes(
    root,
    relative(root, materialsRoot) + "/" + report.textFile,
    16 * 1024 * 1024,
  );
  check(
    textInput.text.trim() &&
      textInput.bytes.length === report.textBytes &&
      textInput.sha256 === report.textSha256,
    "材料原文全文字节或摘要变化",
  );
  return {
    root,
    output,
    bundle,
    reference: ref,
    sourceEvidence: evidence,
    assembly,
    files,
    materials: { report, reportInput, textInput, ...materialPackages(report) },
    evidence: [refInput, evidenceInput, rawBuildInput, assemblyInput].map(
      (i) => ({ path: relative(root, i.path), sha256: i.sha256 }),
    ),
  };
}
