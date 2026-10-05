import assert from "node:assert/strict";
import { mkdirSync } from "node:fs";
import { basename, dirname, join, relative } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  fileHash,
  fileInventory,
} from "../previs/signalling-package-files.mjs";
import { readHandoffInputs } from "./notice-handoff-inputs.mjs";
import {
  copyHandoffBundle,
  inspectHandoffSignatures,
  permissionTree,
  progressReceipt,
  saveHandoffFile,
  unchangedHandoffInputs,
  validateSignatureReports,
} from "./notice-handoff-files.mjs";
import { noticeHash } from "../previs/signalling-notices-files.mjs";
import { handoffBytes } from "./notice-handoff-paths.mjs";
import { plainAncestors } from "../previs/desktop-assembly-files.mjs";
const instructions = `# 舞台大师：内部候选材料交接\n\n本目录只用于内部材料与文件审查，不是客户安装器或已批准发行包。请勿从此副本启动应用；本机绝对路径资格、内部身份与验收实例不能随复制转为客户资格，也不要同时运行相同实例副本。原产品包、签名与权限未修改。\n\nlicenses/保存桌面保守源码材料全文及索引，包含构建／可选平台依赖，不是最终二进制清单。已安装原文已收集不等于商业批准、所有内嵌组件／素材／UE许可充分；未安装可选项仍明列。包内原Node／信令告知保持，请一并核对。\n\n交接清单记录产品来源提交、原候选与材料摘要、完整文件／链接／模式和只读签名结果。保存修订、来源快照与包自身摘要各按既有格式定义，不能互相冒用。签名有效仅证明当前封套未改变，不证明客户权限、公证或最低系统资格。\n\n操作与恢复见docs/中的两份.source.txt来源原文；内容逐字节保留原Markdown，文本中的相对引用指向StageMaster仓库，本审查目录不附全套研究或历史文档，不把它们当作目录内可点击链接。不得仅凭这些说明开启声音、真实灯具、刷机或部署。听感余段、历史偶发、真实厂家档案、差分／实灯／完整最坏组合／8小时、客户Shipping／签名权限／最低系统／完整许可及独立灯光师三任务仍开放。\n`;
export async function assembleNoticeHandoff(
  root,
  reference,
  materials,
  destination,
  dependencies = {},
) {
  const inputs = await readHandoffInputs(
    root,
    reference,
    materials,
    destination,
  );
  const inspect = dependencies.inspect ?? inspectHandoffSignatures,
    copy = dependencies.copy ?? copyHandoffBundle;
  const write = dependencies.write ?? saveHandoffFile;
  const permissions = permissionTree(inputs.bundle);
  const record = {
    format: "stagemaster.internal-notice-handoff",
    version: 1,
    status: "assembling",
    startedAt: new Date().toISOString(),
    sourceCommit: inputs.reference.sourceCommit,
    instance: inputs.reference.id,
    sourceBundle: relative(inputs.root, inputs.bundle),
    customerReleaseQualified: false,
    commercialReleaseApproved: false,
    bundledBinaryInventory: false,
    sourceNativeQualificationTransferred: false,
    automaticLaunch: false,
    sourceEvidence: inputs.evidence,
    noticeReportSha256: inputs.materials.reportInput.sha256,
    noticeTextSha256: inputs.materials.textInput.sha256,
    packages: inputs.materials.packageCount,
    installedMissing: inputs.materials.installedMissing,
    notInstalled: inputs.materials.notInstalled,
  };
  record.toolSourceHashes = {};
  for (const name of [
    "notice-handoff.mjs",
    "notice-handoff-paths.mjs",
    "notice-handoff-inputs.mjs",
    "notice-handoff-files.mjs",
  ])
    record.toolSourceHashes["tools/desktop/" + name] = await fileHash(
      fileURLToPath(new URL(name, import.meta.url)),
    );
  mkdirSync(dirname(inputs.output), { recursive: true });
  plainAncestors(dirname(inputs.output));
  mkdirSync(inputs.output);
  const receipt = progressReceipt(join(inputs.output, "handoff.json"), record);
  try {
    const executable = inputs.assembly.original.executable;
    record.sourceSignatures = inspect(inputs.root, inputs.bundle, executable);
    validateSignatureReports(record.sourceSignatures);
    const bundle = join(inputs.output, basename(inputs.bundle));
    copy(inputs.bundle, bundle);
    assert.deepEqual(
      await fileInventory(bundle),
      inputs.files,
      "候选复制字节不一致",
    );
    assert.deepEqual(
      permissionTree(bundle),
      permissions,
      "候选复制权限／链接不一致",
    );
    record.copiedSignatures = inspect(inputs.root, bundle, executable);
    validateSignatureReports(record.copiedSignatures);
    // codesign embeds the inspected path in diagnostics; compare exact entitlements and signing metadata independently of that path.
    const canonical = (entries) =>
      entries.map((e) => ({
        ...e,
        details: e.details
          .split("\n")
          .filter((l) => !l.startsWith("Executable="))
          .join("\n"),
      }));
    assert.deepEqual(
      canonical(record.copiedSignatures),
      canonical(record.sourceSignatures),
      "复制后原签名／权限元数据变化",
    );
    mkdirSync(join(inputs.output, "licenses"));
    write(
      join(inputs.output, "licenses/notices.json"),
      inputs.materials.reportInput.bytes,
    );
    write(
      join(inputs.output, "licenses/THIRD-PARTY-NOTICES.txt"),
      inputs.materials.textInput.bytes,
    );
    assert.deepEqual(
      handoffBytes(inputs.root, join(inputs.output, "licenses/notices.json"))
        .bytes,
      inputs.materials.reportInput.bytes,
      "复制后材料索引变化",
    );
    assert.deepEqual(
      handoffBytes(
        inputs.root,
        join(inputs.output, "licenses/THIRD-PARTY-NOTICES.txt"),
        16 * 1024 * 1024,
      ).bytes,
      inputs.materials.textInput.bytes,
      "复制后材料原文变化",
    );
    mkdirSync(join(inputs.output, "docs"));
    record.operationDocuments = [];
    for (const name of ["first-release.md", "first-release-operations.md"]) {
      const source = handoffBytes(inputs.root, "docs/development/" + name);
      const targetName = name.replace(/\.md$/, ".source.txt");
      write(join(inputs.output, "docs", targetName), source.bytes);
      record.operationDocuments.push({
        file: "docs/" + targetName,
        source: "docs/development/" + name,
        sha256: source.sha256,
      });
    }
    write(join(inputs.output, "README.md"), instructions);
    await unchangedHandoffInputs(inputs);
    assert.deepEqual(
      permissionTree(inputs.bundle),
      permissions,
      "原候选权限／链接变化",
    );
    record.bundleFiles = inputs.files;
    record.bundlePermissions = permissions;
    record.copiedBundle = basename(bundle);
    record.readmeSha256 = noticeHash(Buffer.from(instructions));
    record.status = "internal-review-materials-ready";
  } catch (error) {
    record.status = "failed";
    record.error = String(error.message).slice(0, 2048);
    throw error;
  } finally {
    record.finishedAt = new Date().toISOString();
    try {
      receipt.update();
    } finally {
      receipt.close();
    }
  }
  return record;
}
if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  if (process.argv.length !== 5)
    throw new Error(
      "用法：node tools/desktop/notice-handoff.mjs 组装引用.json 材料目录 data/DESKTOP-011/新目标",
    );
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const record = await assembleNoticeHandoff(root, ...process.argv.slice(2));
  console.log(
    JSON.stringify({
      status: record.status,
      instance: record.instance,
      packages: record.packages,
      installedMissing: record.installedMissing,
      notInstalled: record.notInstalled,
      customerReleaseQualified: false,
    }),
  );
}
