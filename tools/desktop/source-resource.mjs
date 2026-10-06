import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export const sourceResourceName = "stagemaster-source.json";
export function writeSourceResource(plan, sourceEvidence) {
  const path = join(plan.temporary, sourceResourceName);
  const bytes = Buffer.from(JSON.stringify(sourceEvidence, null, 2) + "\n");
  writeFileSync(path, bytes, { flag: "wx", mode: 0o600 });
  return { path, bytes };
}
export function internalSourceConfig(plan, base, resource) {
  const existing = base.bundle?.resources;
  let resources;
  if (Array.isArray(existing)) resources = [...existing, resource.path];
  else {
    if (
      existing !== undefined &&
      existing !== null &&
      typeof existing !== "object"
    )
      throw Error("内部来源资源配置无效");
    if (Object.values(existing ?? {}).includes(sourceResourceName))
      throw Error("内部来源资源名称已占用");
    resources = { ...(existing ?? {}), [resource.path]: sourceResourceName };
  }
  return {
    ...plan.config,
    bundle: { ...plan.config.bundle, resources },
    app: {
      windows: base.app.windows.map((window) => ({
        ...window,
        title: `${window.title} · 内部发布验收`,
      })),
    },
  };
}
export function verifySourceResource(bundle, files, expectedBytes) {
  const candidates = files.filter(
    (file) =>
      file.path.startsWith("Contents/Resources/") &&
      file.path.split("/").at(-1) === sourceResourceName,
  );
  assert.equal(candidates.length, 1, "归档必须有唯一内部来源文件");
  assert.deepEqual(
    readFileSync(join(bundle, candidates[0].path)),
    expectedBytes,
    "归档来源文件与本轮输入不一致",
  );
  return candidates[0].path;
}
