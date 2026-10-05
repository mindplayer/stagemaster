import { join } from "node:path";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import {
  noticeDirectory,
  noticePath,
  noticeText,
  noticeHash,
} from "../previs/signalling-notices-files.mjs";
import { readJson } from "../previs/signalling-package-files.mjs";
import { verifyRustArchives } from "./collect-dependency-notices.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const check = (ok, message) => {
  if (!ok) throw new Error(message);
};
const component = (value) =>
  typeof value === "string" &&
  /^[A-Za-z0-9_.+-]+$/.test(value) &&
  ![".", ".."].includes(value);
function repository(manifest) {
  const value = manifest.match(/^repository = "([^"]+)"$/m)?.[1];
  const match = value?.match(
    /^https:\/\/github\.com\/([A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+?)(?:\.git)?\/?$/,
  );
  check(match, "Rust原清单仓库不是限定官方来源");
  return match[1];
}
export function proveSupplementSources(report, assetDirectory, options = {}) {
  const assets = noticeDirectory(assetDirectory),
    catalog = readJson(noticePath(assets, "sources.json"));
  check(
    Array.isArray(catalog.entries) && catalog.entries.length <= 128,
    "补充证明预算无效",
  );
  const cargoHome = options.cargoHome ?? join(root, "tmp/cargo-home"),
    ui = noticeDirectory(
      options.uiDirectory ?? join(root, "apps/ui-prototype"),
    ),
    registry = options.registryName ?? "index.crates.io-1949cf8c6b5b557f";
  check(component(registry), "Rust注册目录无效");
  const sources = [],
    proofs = [];
  for (const source of catalog.entries) {
    const entry = report.packages.find(
      (p) =>
        p.ecosystem === source.ecosystem &&
        p.name === source.name &&
        p.version === source.version,
    );
    check(entry?.materialStatus === "missing", "补充目标不是已安装缺项");
    const proof = {
      ecosystem: entry.ecosystem,
      name: entry.name,
      version: entry.version,
      license: entry.license,
      manifestSha256: entry.manifestSha256,
    };
    if (source.ecosystem === "cargo") {
      check(
        component(entry.name) && component(entry.version),
        "Rust包身份路径无效",
      );
      const folder = noticeDirectory(
          join(
            cargoHome,
            "registry/src",
            registry,
            `${entry.name}-${entry.version}`,
          ),
        ),
        p = source.provenance;
      check(
        Array.isArray(p.sourceFiles) &&
          p.sourceFiles.length >= 3 &&
          p.sourceFiles.length <= 17,
        "Rust原文件预算无效",
      );
      const fileHashes = {};
      for (const descriptor of p.sourceFiles) {
        check(!Object.hasOwn(fileHashes, descriptor.file), "Rust原文件重复");
        const actual = noticeText(folder, descriptor.file);
        check(
          actual.sha256 === descriptor.sha256,
          `Rust缓存原文变化：${entry.name}/${descriptor.file}`,
        );
        fileHashes[descriptor.file] = actual.sha256;
      }
      check(
        fileHashes["Cargo.toml"] === entry.manifestSha256 &&
          fileHashes["Cargo.toml.orig"] &&
          fileHashes[".cargo_vcs_info.json"],
        "Rust发布清单或VCS证明缺失",
      );
      const vcs = readJson(noticePath(folder, ".cargo_vcs_info.json"));
      check(/^[a-f0-9]{40}$/.test(vcs.git?.sha1), "Rust原包没有有效固定提交");
      const cratePath = vcs.path_in_vcs ?? p.cratePath;
      check(
        p.upstreamManifest.sha256 === fileHashes["Cargo.toml.orig"],
        "Rust固定清单不等于原发布清单",
      );
      Object.assign(proof, {
        repository: repository(noticeText(folder, "Cargo.toml").text),
        commit: vcs.git.sha1,
        cratePath,
        fileHashes,
      });
      if (p.kind === "cargo-license-pointer") {
        const pointer = p.licensePointer,
          bytes = readFileSync(noticePath(folder, pointer.file));
        check(
          noticeHash(bytes) === pointer.sha256 &&
            Number.isSafeInteger(pointer.headerBytes) &&
            pointer.headerBytes > 0 &&
            pointer.headerBytes <= bytes.length,
          "源码许可告知头预算或摘要不符",
        );
        proof.licenseHeaderSha256 = noticeHash(
          bytes.subarray(0, pointer.headerBytes),
        );
      }
      sources.push({
        name: entry.name,
        version: entry.version,
        source: entry.source,
        archive: noticePath(
          join(cargoHome, "registry/cache", registry),
          `${entry.name}-${entry.version}.crate`,
        ),
        files: p.sourceFiles,
      });
    } else {
      check(
        source.ecosystem === "npm" && typeof entry.path === "string",
        "补充生态或安装路径无效",
      );
      const current = noticeText(ui, `${entry.path}/package.json`),
        manifest = readJson(noticePath(ui, `${entry.path}/package.json`));
      check(
        current.sha256 === entry.manifestSha256 &&
          manifest.name === entry.name &&
          manifest.version === entry.version &&
          manifest.license === entry.license,
        "npm当前安装身份或清单与原材料不符",
      );
      proof.integrity = entry.integrity;
      const parent = source.provenance.parent;
      if (parent) {
        const installed = noticeText(ui, `${parent.path}/package.json`),
          pm = readJson(noticePath(ui, `${parent.path}/package.json`));
        check(
          pm.name === parent.name &&
            pm.version === parent.version &&
            pm.license === parent.license,
          "npm当前发布父包身份不符",
        );
        proof.parent = {
          manifestSha256: installed.sha256,
          optionalTargetVersion: pm.optionalDependencies?.[entry.name],
          noticeSha256: noticeText(ui, `${parent.path}/LICENSE.md`).sha256,
        };
      }
    }
    proofs.push(proof);
  }
  if (sources.length) {
    const verified = verifyRustArchives(
      options.lockText ?? readFileSync(join(root, "Cargo.lock"), "utf8"),
      sources,
    );
    let index = 0;
    for (const proof of proofs)
      if (proof.ecosystem === "cargo") {
        Object.assign(proof, {
          archiveSha256: verified[index].archiveSha256,
          archiveFilesCompared: verified[index].filesCompared,
        });
        index++;
      }
  }
  return proofs;
}
