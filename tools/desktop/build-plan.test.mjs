import assert from "node:assert/strict";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { desktopBuildPlan } from "./build-plan.mjs";
const root = fileURLToPath(new URL("../../", import.meta.url));

test("原dev和build仍为debug，没有内部feature或发布路径", () => {
  for (const command of ["dev", "build"]) {
    const p = desktopBuildPlan(root, command);
    assert.equal(p.profile, "debug");
    assert.equal(p.sidecarDirectory, root + "tmp/desktop-bin");
    assert.equal(p.cliArgs.includes("internal-acceptance"), false);
    assert.equal(p.hostArgs.includes("--release"), false);
  }
});
test("内部release桌面和audio后台同profile，独立目标与sidecar", () => {
  const p = desktopBuildPlan(
    root,
    "build-internal-release",
    "desktop-release-AbC1",
  );
  assert.equal(p.profile, "release");
  assert.ok(p.hostArgs.includes("--release"));
  assert.ok(p.hostArgs.includes("audio"));
  assert.ok(p.hostArgs.includes("--locked"));
  assert.ok(p.hostArgs.includes("--offline"));
  assert.equal(p.cliArgs.includes("--debug"), false);
  assert.ok(p.cliArgs.includes("internal-acceptance"));
  assert.ok(p.cliArgs.includes("--no-sign"));
  assert.ok(p.cliArgs.includes("--locked"));
  assert.ok(p.cliArgs.includes("--offline"));
  assert.equal(p.target, root + "tmp/desktop-release-target");
  assert.ok(p.sidecarDirectory.startsWith(root + "tmp/desktop-release-AbC1/"));
});
test("内部身份／标题和准确实例绑定，不使用客户身份", () => {
  const p = desktopBuildPlan(
    root,
    "build-internal-release",
    "desktop-release-AbC1",
  );
  assert.equal(p.identifier, "cn.stagemaster.acceptance.desktop-release-abc1");
  assert.equal(p.config.productName, "舞台大师 内部发布验收");
  assert.equal(
    p.launchEnvironment.STAGEMASTER_ACCEPTANCE_INSTANCE,
    "desktop-release-AbC1",
  );
  assert.ok(p.archive.startsWith(root + "data/DESKTOP-005/"));
  assert.ok(p.logs.startsWith(root + "logs/DESKTOP-005/"));
});
for (const instance of [
  undefined,
  "",
  "../other",
  "desktop-release-a/other",
  "desktop-release-有空格",
  "desktop-release-" + "a".repeat(65),
]) {
  test(`内部实例${String(instance)}在写入前拒绝`, () =>
    assert.throws(
      () => desktopBuildPlan(root, "build-internal-release", instance),
      /实例/,
    ));
}
test("未知构建命令拒绝，不静默变成debug", () =>
  assert.throws(
    () => desktopBuildPlan(root, "shipping", "desktop-release-AbC1"),
    /命令/,
  ));
