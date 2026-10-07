import { mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import {
  runtimeCheck,
  runtimeAssetTest,
  successfulReport,
} from "./packaging-plan.mjs";
import { runCommand } from "./packaging-process.mjs";

export function rendererPackageOptions(args) {
  if (args.length === 0) return { buildOnly: false };
  if (args.length === 1 && args[0] === "--build-only")
    return { buildOnly: true };
  throw new Error("仅接受 --build-only；引擎路径使用 STAGEMASTER_UE_ROOT");
}

// Building a component never confers runtime or GPU qualification.
export async function checkRendererPackage(
  evidence,
  program,
  options,
  execute = runCommand,
) {
  evidence.program = program;
  if (options.buildOnly) {
    evidence.status = "component-built";
    console.log(`独立组件已构建：${program}（未运行资源／GPU 验收）`);
    return;
  }
  const check = runtimeCheck(evidence.plan, program);
  evidence.runtimeCheck = check;
  mkdirSync(check.report, { recursive: true });
  await execute(check, evidence.plan);
  const report = JSON.parse(
    readFileSync(join(check.report, "index.json"), "utf8").replace(
      /^\uFEFF/,
      "",
    ),
  );
  evidence.tests = successfulReport(report, runtimeAssetTest);
  evidence.status = "component-verified";
  console.log(
    `独立组件／必需资源通过：${program}（未代表桌面／GPU／客户安装验收）`,
  );
}
