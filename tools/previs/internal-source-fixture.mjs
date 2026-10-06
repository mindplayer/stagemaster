import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { sourceFixture } from "../desktop/source-evidence-fixture.mjs";
import { captureSourceEvidence } from "../desktop/source-evidence.mjs";
import { desktopBuildPlan } from "../desktop/build-plan.mjs";
import { desktopAssemblyPlan } from "./desktop-assembly-plan.mjs";
import { internalReleaseSource } from "./desktop-internal-release.mjs";
import { fileHash, fileInventory } from "./signalling-package-files.mjs";

// Real Git/files for metadata checks, never compiler or native qualification.
export async function snapshotAssemblyFixture(t, objectFormat = "sha1") {
  const f = sourceFixture(t, objectFormat);
  const write = (file, bytes) => {
    mkdirSync(dirname(file), { recursive: true });
    writeFileSync(file, bytes);
  };
  const build = desktopBuildPlan(
    f.root,
    "build-internal-release",
    "desktop-release-Snapshot1",
  );
  const bundle = join(build.archive, build.config.productName + ".app");
  for (const file of [
    "Contents/Info.plist",
    "Contents/MacOS/stagemaster-desktop",
    "Contents/MacOS/stagemaster-execution-host",
    "Contents/Resources/icon.icns",
  ])
    write(join(bundle, file), "fixture, not executable: " + file);
  const rawFile = join(build.archive, "build-record.json");
  const raw = {
    task: "DESKTOP-005",
    status: "isolated-release-built",
    plan: build,
    customerReleaseQualified: false,
    commandResult: { code: 0, signal: null },
    bundle,
    sourceEvidence: await captureSourceEvidence(f.root),
    sourceResourcePath: "Contents/Resources/stagemaster-source.json",
    host: {
      profile: "release",
      status: 0,
      targetDirectory: build.target,
      args: build.hostArgs,
    },
    cli: { args: ["tauri.js", ...build.cliArgs] },
  };
  let actual;
  const save = async (resource = true) => {
    if (resource)
      write(
        join(bundle, "Contents/Resources/stagemaster-source.json"),
        JSON.stringify(raw.sourceEvidence, null, 2) + "\n",
      );
    actual = await fileInventory(bundle);
    raw.files = structuredClone(actual);
    raw.originalFiles = structuredClone(actual);
    raw.host.sha256 = actual.find((row) =>
      row.path.endsWith("/stagemaster-execution-host"),
    ).sha256;
    write(rawFile, JSON.stringify(raw));
  };
  await save();
  const plan = desktopAssemblyPlan(
    f.root,
    [
      bundle,
      "data/Game/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
      "data/Node/previs",
    ],
    "previs-desktop-Check",
  );
  return {
    ...f,
    build,
    bundle,
    plan,
    raw,
    rawFile,
    save,
    write,
    actual: () => actual,
    check: (record = rawFile) =>
      internalReleaseSource(plan, record, actual, build.identifier),
    recordHash: () => fileHash(rawFile),
    runtimeExists: () =>
      existsSync(join(f.root, "tmp/desktop-" + build.instance)),
  };
}
