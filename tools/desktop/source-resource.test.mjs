import assert from "node:assert/strict";
import { test } from "node:test";
import {
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
  readFileSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { desktopBuildPlan } from "./build-plan.mjs";
import {
  writeSourceResource,
  internalSourceConfig,
  verifySourceResource,
  sourceResourceName,
} from "./source-resource.mjs";
const root = fileURLToPath(new URL("../../", import.meta.url));
function fixture(t) {
  const folder = mkdtempSync(join(root, "tmp/desktop-013-resource-test-"));
  t.after(() => rmSync(folder, { recursive: true, force: true }));
  const plan = desktopBuildPlan(
    folder,
    "build-internal-release",
    "desktop-release-Own1",
  );
  mkdirSync(plan.temporary, { recursive: true });
  return { folder, plan };
}
test("来源文件独占写入，资源映射／数组与原窗口配置保持", (t) => {
  const { plan } = fixture(t),
    resource = writeSourceResource(plan, { version: 1, git: { dirty: true } });
  assert.throws(() => writeSourceResource(plan, {}), /EEXIST/);
  assert.deepEqual(readFileSync(resource.path), resource.bytes);
  const base = {
    app: { windows: [{ title: "舞台大师", width: 1440 }] },
    bundle: { resources: { "existing.txt": "existing.txt" } },
  };
  const config = internalSourceConfig(plan, base, resource);
  assert.equal(config.bundle.resources["existing.txt"], "existing.txt");
  assert.equal(config.bundle.resources[resource.path], sourceResourceName);
  assert.equal(config.app.windows[0].width, 1440);
  base.bundle.resources = ["existing-directory/"];
  assert.deepEqual(
    internalSourceConfig(plan, base, resource).bundle.resources,
    ["existing-directory/", resource.path],
  );
});
test("已占用来源名和不合法资源类型拒绝，不丢弃旧资源", (t) => {
  const { plan } = fixture(t),
    resource = writeSourceResource(plan, {}),
    base = {
      app: { windows: [] },
      bundle: { resources: { "other.json": sourceResourceName } },
    };
  assert.throws(() => internalSourceConfig(plan, base, resource), /已占用/);
  base.bundle.resources = "invalid";
  assert.throws(() => internalSourceConfig(plan, base, resource), /无效/);
});
test("真实随包文件必须唯一且逐字节等于本轮来源", (t) => {
  const { folder } = fixture(t),
    relative = "Contents/Resources/" + sourceResourceName;
  mkdirSync(join(folder, "Contents/Resources"), { recursive: true });
  const bytes = Buffer.from('{"version":1}\n');
  writeFileSync(join(folder, relative), bytes);
  assert.equal(
    verifySourceResource(folder, [{ path: relative }], bytes),
    relative,
  );
  assert.throws(() => verifySourceResource(folder, [], bytes), /唯一/);
  assert.throws(
    () =>
      verifySourceResource(
        folder,
        [
          { path: relative },
          { path: "Contents/Resources/other/" + sourceResourceName },
        ],
        bytes,
      ),
    /唯一/,
  );
  assert.throws(
    () =>
      verifySourceResource(folder, [{ path: relative }], Buffer.from("wrong")),
    /不一致/,
  );
});
