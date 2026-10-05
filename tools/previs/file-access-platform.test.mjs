import assert from "node:assert/strict";
import { test } from "node:test";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { basename, join } from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import {
  canonicalFolders,
  entitlementXml,
  prepareFileAccessParents,
  sandboxDenials,
} from "./file-access-platform.mjs";
import { fileAccessKey, fileAccessPlan } from "./file-access-plan.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url)).replace(
  /\/$/,
  "",
);
const target = `${root}/tmp/previs-file-access-Test/denied-report`;
const pid = 1234;
const denial = () =>
  `kernel[0] Sandbox: StageMasterPreview(${pid}) deny(1) file-write-create ${target}/index.html\nkernel[0] Sandbox: StageMasterPreview(${pid}) deny(1) file-write-data ${target}/index.json\n`;
const execute = (stdout) => () => ({ status: 0, stdout });
test("真实所属 Game PID 的 JSON／HTML两项系统拒绝才合格", () => {
  assert.equal(sandboxDenials(root, target, pid, execute(denial())), denial());
});
for (const [name, text] of [
  ["普通日志／命令回显", `log run noninteractively deny ${target}`],
  ["其他PID", denial().replaceAll(`(${pid})`, "(5678)")],
  ["其他程序", denial().replaceAll("StageMasterPreview", "Other")],
  ["其他目录", denial().replaceAll(target, `${target}-other`)],
  ["只有HTML", denial().split("\n")[0]],
  ["只有JSON", denial().split("\n")[1]],
])
  test(`不能误收沙盒拒绝：${name}`, () =>
    assert.throws(
      () => sandboxDenials(root, target, pid, execute(text)),
      /实际沙盒拒绝/,
    ));
test("读取系统日志失败明确拒绝，不替换为假拒绝", () => {
  assert.throws(
    () =>
      sandboxDenials(root, target, pid, () => ({
        status: 1,
        stdout: denial(),
      })),
    /不能取得/,
  );
});
test("官方 plutil 完整往返中文空格资格，不丢布尔或尾斜线", () => {
  const values = {
    "com.apple.security.app-sandbox": true,
    [fileAccessKey]: [`${root}/tmp/中文 空格/`],
  };
  const xml = entitlementXml(root, values);
  assert.match(xml, /中文 空格/);
  const json = execFileSync(
    "/usr/bin/plutil",
    ["-convert", "json", "-o", "-", "--", "-"],
    {
      cwd: root,
      env: { ...process.env, TMPDIR: join(root, "tmp") },
      input: xml,
      encoding: "utf8",
    },
  );
  assert.deepEqual(JSON.parse(json), values);
});
test("所有运行目录必须真实规范路径，包内链接也拒绝", (t) => {
  const temporary = mkdtempSync(join(root, "tmp/previs-file-access-"));
  const p = fileAccessPlan(
    root,
    join(
      root,
      "data/source/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
    ),
    basename(temporary),
  );
  for (const folder of [
    p.archive,
    p.logs,
    p.user,
    p.cache,
    p.runtimeTemporary,
    p.report,
  ])
    mkdirSync(folder, { recursive: true });
  t.after(() => {
    for (const folder of [p.archive, p.logs, p.temporary])
      rmSync(folder, { recursive: true, force: true });
  });
  canonicalFolders(p);
  const alias = join(temporary, "alias");
  symlinkSync(p.user, alias);
  assert.throws(() => canonicalFolders({ ...p, user: alias }), /链接/);
});

test("首次父目录缺失正常创建；链接父目录在任何新写入前拒绝", (t) => {
  const fixture = mkdtempSync(join(root, "tmp/previs-file-parent-test-"));
  t.after(() => rmSync(fixture, { recursive: true, force: true }));
  const ordinary = join(fixture, "ordinary");
  mkdirSync(ordinary);
  prepareFileAccessParents(ordinary);
  assert.ok(existsSync(join(ordinary, "logs/PREVIS-004")));
  const unsafe = join(fixture, "unsafe"),
    outside = join(fixture, "outside");
  mkdirSync(unsafe);
  mkdirSync(outside);
  symlinkSync(outside, join(unsafe, "data"));
  assert.throws(() => prepareFileAccessParents(unsafe), /写入前拒绝/);
  assert.equal(existsSync(join(unsafe, "tmp")), false);
  assert.equal(existsSync(join(outside, "PREVIS-004")), false);
});
