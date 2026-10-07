import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  checkRendererPackage,
  rendererPackageOptions,
} from "./renderer-package-check.mjs";
import { runtimeAssetTest } from "./packaging-plan.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));

test("默认保留运行验收，仅明确的单一选项进入构建模式", () => {
  assert.deepEqual(rendererPackageOptions([]), { buildOnly: false });
  assert.deepEqual(rendererPackageOptions(["--build-only"]), {
    buildOnly: true,
  });
  for (const args of [
    ["--skip-tests"],
    ["--build-only", "extra"],
    ["--build-only", "--build-only"],
  ])
    assert.throws(() => rendererPackageOptions(args), /仅接受/);
});

test("仅构建不启动 Game，不记成运行验收通过", async () => {
  const evidence = { status: "running" };
  await checkRendererPackage(
    evidence,
    "/fixture/Game",
    { buildOnly: true },
    () => {
      assert.fail("构建模式不得运行 Game");
    },
  );
  assert.equal(evidence.status, "component-built");
  assert.equal(evidence.program, "/fixture/Game");
  assert.equal(Object.hasOwn(evidence, "tests"), false);
  assert.equal(Object.hasOwn(evidence, "runtimeCheck"), false);
});

for (const result of ["missing", "failed", "passed"])
  test(`默认验收必须取得成功报告：${result}`, async (t) => {
    mkdirSync(join(root, "tmp"), { recursive: true });
    const temporary = mkdtempSync(join(root, "tmp/renderer-check-test-"));
    t.after(() => rmSync(temporary, { recursive: true, force: true }));
    const evidence = {
      status: "running",
      plan: { temporary, logs: temporary, user: temporary, cache: temporary },
    };
    let calls = 0;
    const execute = async (check) => {
      calls += 1;
      if (result !== "missing")
        writeFileSync(
          join(check.report, "index.json"),
          JSON.stringify({
            tests: [
              {
                fullTestPath: runtimeAssetTest,
                state: result === "passed" ? "Success" : "Fail",
                errors: 0,
              },
            ],
            succeeded: 1,
            failed: 0,
            notRun: 0,
            inProcess: 0,
          }),
        );
    };
    const run = checkRendererPackage(
      evidence,
      "/fixture/Game",
      { buildOnly: false },
      execute,
    );
    if (result === "passed") {
      await run;
      assert.equal(evidence.status, "component-verified");
      assert.deepEqual(evidence.tests, [runtimeAssetTest]);
    } else {
      await assert.rejects(run);
      assert.notEqual(evidence.status, "component-verified");
    }
    assert.equal(calls, 1);
  });
