import assert from "node:assert/strict";
import { test } from "node:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { desktopBuildPlan } from "./build-plan.mjs";
import { prepareHost } from "./prepare-host.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
test("显式后台计划在所属源码工作区编译，不误用模块所在主工作区", (t) => {
  const folder = mkdtempSync(join(root, "tmp/desktop-013-host-source-"));
  t.after(() => rmSync(folder, { recursive: true, force: true }));
  const plan = desktopBuildPlan(
    folder,
    "build-internal-release",
    "desktop-release-Own1",
  );
  let cwd;
  const host = prepareHost(plan, process.env, (program, args, options) => {
    if (program === "rustc")
      return {
        status: 0,
        stdout: "rustc fixture\nhost: aarch64-apple-darwin\n",
      };
    assert.equal(program, "cargo");
    cwd = options.cwd;
    const target = join(options.env.CARGO_TARGET_DIR, "release");
    mkdirSync(target, { recursive: true });
    writeFileSync(
      join(target, "stagemaster-execution-host"),
      "fixture; not actual compiler qualification",
    );
    return { status: 0 };
  });
  console.log(
    JSON.stringify({ expected: folder, observed: cwd, realCompilerRun: false }),
  );
  assert.equal(cwd, folder);
  assert.equal(host.status, 0);
});
