import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { noticeHash } from "../previs/signalling-notices-files.mjs";
import { fileInventory } from "../previs/signalling-package-files.mjs";
const project = fileURLToPath(new URL("../../", import.meta.url));
export const dependencies = [
  "Cargo.toml",
  "Cargo.lock",
  "apps/desktop/Cargo.toml",
  "apps/execution-host/Cargo.toml",
  "apps/ui-prototype/package.json",
  "apps/ui-prototype/package-lock.json",
];
export async function fixture(t) {
  const root = mkdtempSync(join(project, "tmp/desktop-011-test-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const write = (key, bytes) => {
    mkdirSync(join(root, key, ".."), { recursive: true });
    writeFileSync(join(root, key), bytes);
  };
  const json = (key, value) =>
    write(key, JSON.stringify(value, null, 2) + "\n");
  const sourceHashes = {};
  for (const key of dependencies) {
    const text = key + " fixture\n";
    write(key, text);
    sourceHashes[key] = noticeHash(Buffer.from(text));
  }
  const sourceCommit = "a".repeat(40),
    id = "desktop-release-Fixture";
  const bundle = join(root, "data/PREVIS-007", id, "舞台大师 内部验收.app");
  write(
    "data/PREVIS-007/" +
      id +
      "/舞台大师 内部验收.app/Contents/MacOS/stagemaster",
    "fake test binary\n",
  );
  for (const path of [
    "Contents/MacOS/stagemaster",
    "Contents/MacOS/stagemaster-execution-host",
    "Contents/Resources/previs/node",
    "Contents/Resources/previs/StageMasterPreview.app/Contents/MacOS/StageMasterPreview",
  ]) {
    write(
      "data/PREVIS-007/" + id + "/舞台大师 内部验收.app/" + path,
      "fake test binary\n",
    );
    chmodSync(join(bundle, path), 0o755);
  }
  for (const name of ["first-release.md", "first-release-operations.md"])
    write("docs/development/" + name, "# Test scope only\n");
  const files = await fileInventory(bundle);
  const rawRecord = "data/DESKTOP-005/" + id + "/build-record.json";
  const build = {
    status: "isolated-release-built",
    customerReleaseQualified: false,
    plan: { instance: id },
    commandResult: { code: 0 },
    bundle,
    files,
  };
  json(rawRecord, build);
  const buildText = JSON.stringify(build, null, 2) + "\n";
  const evidence = {
    sourceCommit,
    rawRecord: join(root, rawRecord),
    recordSha256: noticeHash(Buffer.from(buildText)),
    bundle,
    buildExit: 0,
    hostProfile: "release",
    customerReleaseQualified: false,
    sourceHashes,
    productSourceHashes: sourceHashes,
  };
  const evidencePath = "data/AUDIO-023/optimized-build.json";
  json(evidencePath, evidence);
  const qualification = {
    record: join(root, evidencePath),
    recordSha256: noticeHash(
      Buffer.from(JSON.stringify(evidence, null, 2) + "\n"),
    ),
    rawRecord: join(root, rawRecord),
    sourceCommit,
    instance: id,
    profile: "release",
    customerReleaseQualified: false,
  };
  const assemblyPath = "data/PREVIS-007/" + id + "/assembly-record.json";
  const assembly = {
    status: "development-assembled",
    customerPackageVerified: false,
    gpuVerified: false,
    plan: { root, id, bundle, desktop: bundle },
    original: { executable: "stagemaster", internalRelease: qualification },
    assembledFiles: files,
  };
  json(assemblyPath, assembly);
  const reference = {
    id,
    sourceCommit,
    bundle,
    rawRecord: assemblyPath,
    sourceProfile: "optimized-desktop-with-development-renderer",
    sourceQualification: qualification,
    assembledFiles: files.length,
    customerPackageVerified: false,
  };
  const referencePath = "data/AUDIO-023/assembly-ref.json";
  json(referencePath, reference);
  const text = "Fixture original notice text\r\n";
  const report = {
    format: "stagemaster.desktop-notice-materials",
    version: 1,
    target: "aarch64-apple-darwin",
    scope:
      "conservative-non-dev-source-materials-including-build-and-optional-packages",
    sourceHashes,
    packages: [
      {
        ecosystem: "cargo",
        name: "fixture",
        version: "1.0.0",
        materialStatus: "collected",
        notices: [
          {
            file: "LICENSE",
            bytes: text.length,
            sha256: noticeHash(Buffer.from(text)),
          },
        ],
      },
    ],
    missingPackages: [],
    textFile: "licenses/THIRD-PARTY-NOTICES.txt",
    textBytes: Buffer.byteLength(text),
    textSha256: noticeHash(Buffer.from(text)),
    commercialReleaseApproved: false,
    reviewStillRequired: true,
    bundledBinaryInventory: false,
  };
  const materials = "data/DESKTOP-009/materials-fixture";
  json(materials + "/licenses/notices.json", report);
  write(materials + "/licenses/THIRD-PARTY-NOTICES.txt", text);
  return {
    root,
    write,
    json,
    reference,
    referencePath,
    evidence,
    evidencePath,
    assembly,
    assemblyPath,
    report,
    materials,
    bundle,
    destination: "data/DESKTOP-011/新交接",
    output: join(root, "data/DESKTOP-011/新交接"),
  };
}
