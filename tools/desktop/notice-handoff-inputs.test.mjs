import assert from "node:assert/strict";
import { existsSync, mkdirSync, symlinkSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { fixture, dependencies } from "./notice-handoff-fixtures.mjs";
import { readHandoffInputs } from "./notice-handoff-inputs.mjs";
const read = (f, destination = f.destination) =>
  readHandoffInputs(f.root, f.referencePath, f.materials, destination);
test("交接输入绑定准确候选、六依赖与未批准原文，不预建目标", async (t) => {
  const f = await fixture(t),
    result = await read(f);
  assert.equal(result.bundle, f.bundle);
  assert.equal(result.reference.sourceCommit, f.reference.sourceCommit);
  assert.equal(result.materials.report.textSha256, f.report.textSha256);
  assert.equal(result.output, f.output);
  assert.equal(result.materials.installedMissing, 0);
  assert.equal(existsSync(f.output), false);
});
test("外部、越级、控制字符、错误任务目标和已有目录拒绝", async (t) => {
  const f = await fixture(t);
  for (const dest of [
    "/outside",
    "../escape",
    "data/DESKTOP-011/../old",
    "data/DESKTOP-010/x",
    "data/DESKTOP-011",
    "data/DESKTOP-011/x\n",
  ])
    await assert.rejects(read(f, dest), /目标|路径|任务/);
  mkdirSync(f.output, { recursive: true });
  await assert.rejects(read(f), /已存在/);
});
test("目标链接祖先与输入文件链接拒绝，不写入指向目录", async (t) => {
  const f = await fixture(t);
  mkdirSync(join(f.root, "data/DESKTOP-011"));
  symlinkSync(join(f.root, "data/DESKTOP-009"), f.output, "dir");
  await assert.rejects(read(f), /已存在|链接/);
  const other = "data/AUDIO-023/ref-link.json";
  symlinkSync(join(f.root, f.referencePath), join(f.root, other));
  await assert.rejects(
    readHandoffInputs(f.root, other, f.materials, "data/DESKTOP-011/other"),
    /链接/,
  );
});
test("来源提交／实例／资格字段变化拒绝", async (t) => {
  for (const key of ["sourceCommit", "id", "customerPackageVerified"]) {
    const f = await fixture(t);
    f.reference[key] = key === "customerPackageVerified" ? true : "wrong";
    f.json(f.referencePath, f.reference);
    await assert.rejects(read(f), /候选|身份|资格|来源/);
  }
});
test("优化证据与原构建记录篡改拒绝", async (t) => {
  const f = await fixture(t);
  f.evidence.buildExit = 1;
  f.json(f.evidencePath, f.evidence);
  await assert.rejects(read(f), /证据|来源/);
});
test("候选文件字节变化拒绝，不能借旧清单登记成功", async (t) => {
  const f = await fixture(t);
  f.write(
    "data/PREVIS-007/" +
      f.reference.id +
      "/舞台大师 内部验收.app/Contents/MacOS/stagemaster",
    "changed\n",
  );
  await assert.rejects(read(f), /候选|清单|变化/);
});
test("产品来源变化和材料锁哈希不对应拒绝", async (t) => {
  const f = await fixture(t);
  f.write(dependencies[0], "changed source\n");
  await assert.rejects(read(f), /来源|变化/);
  const g = await fixture(t);
  g.report.sourceHashes[dependencies[0]] = "b".repeat(64);
  g.json(g.materials + "/licenses/notices.json", g.report);
  await assert.rejects(read(g), /依赖|来源/);
});
test("原文全文篡改／声明字节变化拒绝", async (t) => {
  const f = await fixture(t);
  f.write(f.materials + "/licenses/THIRD-PARTY-NOTICES.txt", "changed\n");
  await assert.rejects(read(f), /原文|全文/);
  const g = await fixture(t);
  g.report.textBytes++;
  g.json(g.materials + "/licenses/notices.json", g.report);
  await assert.rejects(read(g), /原文|全文/);
});
test("许可或二进制假批准、非保守资料拒绝", async (t) => {
  for (const [key, value] of [
    ["commercialReleaseApproved", true],
    ["reviewStillRequired", false],
    ["bundledBinaryInventory", true],
    ["scope", "customer-ready"],
  ]) {
    const f = await fixture(t);
    f.report[key] = value;
    f.json(f.materials + "/licenses/notices.json", f.report);
    await assert.rejects(read(f), /材料|批准|保守/);
  }
});
test("已安装缺项、未知状态和重复包不能删掉或隐去", async (t) => {
  for (const state of ["missing", "invented"]) {
    const f = await fixture(t);
    f.report.packages[0].materialStatus = state;
    f.json(f.materials + "/licenses/notices.json", f.report);
    await assert.rejects(read(f), /缺项|状态|原文/);
  }
  const f = await fixture(t);
  f.report.packages.push(f.report.packages[0]);
  f.json(f.materials + "/licenses/notices.json", f.report);
  await assert.rejects(read(f), /重复/);
});
test("可选未安装项保留，不计为已安装材料缺项", async (t) => {
  const f = await fixture(t);
  f.report.packages.push({
    ecosystem: "npm",
    path: "node_modules/optional",
    version: "1.0.0",
    materialStatus: "not-installed",
    optional: true,
    notices: [],
  });
  f.report.missingPackages.push({
    ecosystem: "npm",
    name: "node_modules/optional",
    version: "1.0.0",
    status: "not-installed",
  });
  f.json(f.materials + "/licenses/notices.json", f.report);
  const result = await read(f);
  assert.equal(result.materials.notInstalled, 1);
});
test("原材料缺项摘要的name字段精确对应包path，不接受误名或缺名", async (t) => {
  const f = await fixture(t);
  f.report.packages.push({
    ecosystem: "npm",
    path: "node_modules/optional",
    version: "1.0.0",
    materialStatus: "not-installed",
    optional: true,
    notices: [],
  });
  f.report.missingPackages = [
    {
      ecosystem: "npm",
      name: "node_modules/optional",
      version: "1.0.0",
      status: "not-installed",
    },
  ];
  f.json(f.materials + "/licenses/notices.json", f.report);
  assert.equal((await read(f)).materials.notInstalled, 1);
  f.report.missingPackages[0].name = "node_modules/wrong";
  f.json(f.materials + "/licenses/notices.json", f.report);
  await assert.rejects(read(f), /缺项/);
  delete f.report.missingPackages[0].name;
  f.json(f.materials + "/licenses/notices.json", f.report);
  await assert.rejects(read(f), /缺项/);
});
test("来源路径越级、包／原文预算和缺依赖声明拒绝", async (t) => {
  const f = await fixture(t);
  delete f.report.sourceHashes[dependencies[0]];
  f.json(f.materials + "/licenses/notices.json", f.report);
  await assert.rejects(read(f), /依赖|来源/);
  const g = await fixture(t);
  g.report.packages = Array.from({ length: 1025 }, () => g.report.packages[0]);
  g.json(g.materials + "/licenses/notices.json", g.report);
  await assert.rejects(read(g), /预算/);
  const h = await fixture(t);
  h.reference.rawRecord = "../escape";
  h.json(h.referencePath, h.reference);
  await assert.rejects(read(h), /路径|来源/);
});
