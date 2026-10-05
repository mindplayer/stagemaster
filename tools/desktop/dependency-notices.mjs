import { readdirSync, existsSync } from "node:fs";
import { dirname, relative, resolve, sep, isAbsolute } from "node:path";
import {
  lockedPackages,
  readJson,
} from "../previs/signalling-package-files.mjs";
import {
  noticeDirectory,
  noticePath,
  noticeText,
} from "../previs/signalling-notices-files.mjs";

const roots = ["stagemaster-desktop", "stagemaster-execution-host"];
const compare = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
function uniqueMap(entries, label) {
  if (!Array.isArray(entries) || entries.length > 1024)
    throw new Error(`${label}预算或结构无效`);
  const map = new Map();
  for (const entry of entries) {
    if (!entry || typeof entry.id !== "string" || map.has(entry.id))
      throw new Error(`${label}身份无效或重复`);
    map.set(entry.id, entry);
  }
  return map;
}
export function rustClosure(metadata) {
  const packages = uniqueMap(metadata.packages, "Rust包"),
    nodes = uniqueMap(metadata.resolve?.nodes, "Rust节点");
  const selected = new Set(),
    pending = roots.map((name) => {
      const matches = [...packages.values()].filter(
        (p) => p.name === name && p.source === null,
      );
      if (matches.length !== 1) throw new Error(`Rust入口缺失或重复：${name}`);
      return matches[0].id;
    });
  while (pending.length) {
    const id = pending.pop();
    if (selected.has(id)) continue;
    if (!packages.has(id)) throw new Error(`Rust闭包成员缺失：${id}`);
    const node = nodes.get(id);
    if (!node) throw new Error(`Rust闭包节点缺失：${id}`);
    if (!Array.isArray(node.deps) || node.deps.length > 1024)
      throw new Error("Rust依赖边预算无效");
    selected.add(id);
    for (const dep of node.deps) {
      if (
        !Array.isArray(dep.dep_kinds) ||
        !dep.dep_kinds.length ||
        dep.dep_kinds.some((k) => ![null, "build", "dev"].includes(k.kind))
      )
        throw new Error("Rust依赖种类无效");
      if (dep.dep_kinds.some((k) => k.kind !== "dev")) pending.push(dep.pkg);
    }
    if (pending.length > 16384) throw new Error("Rust依赖遍历预算超限");
  }
  return [...selected]
    .map((id) => packages.get(id))
    .sort((a, b) => compare(a.id, b.id));
}
function licenseFiles(directory, explicit = []) {
  const names = readdirSync(directory).filter((name) =>
    /^(licen[sc]e|copying|notice)([._-]|$)/i.test(name),
  );
  const files = [...new Set([...names, ...explicit])].sort(compare);
  if (files.length > 16) throw new Error(`单包告知文件预算超限：${directory}`);
  return files;
}
function collectFiles(directory, files, label, checksums, budget) {
  const chunks = [],
    notices = files.map((file) => {
      const notice = noticeText(directory, file);
      budget.bytes += notice.bytes;
      if (budget.bytes > 16 * 1024 * 1024)
        throw new Error("告知收集文本总预算超限");
      if (checksums && checksums[file] !== notice.sha256)
        throw new Error(`Rust告知校验不匹配：${label}/${file}`);
      chunks.push({ label: `${label} / ${file}`, text: notice.text });
      return { file, sha256: notice.sha256, bytes: notice.bytes };
    });
  return { chunks, notices };
}
export function collectUiNotices(value) {
  const directory = noticeDirectory(value);
  const lock = readJson(noticePath(directory, "package-lock.json"));
  const packages = lockedPackages(
    readJson(noticePath(directory, "package.json")),
    lock,
  );
  if (packages.length > 1024) throw new Error("界面依赖预算超限");
  const chunks = [],
    budget = { bytes: 0 },
    entries = packages
      .sort((a, b) => compare(a.path, b.path))
      .map((entry) => {
        const source = {
          ...entry,
          optional: lock.packages[entry.path].optional === true,
          archiveIntegrityVerified: false,
        };
        let manifestPath;
        try {
          manifestPath = noticePath(directory, `${entry.path}/package.json`);
        } catch (error) {
          if (error.code !== "ENOENT") throw error;
          return {
            ...source,
            ecosystem: "npm",
            materialStatus: "not-installed",
            notices: [],
          };
        }
        const manifest = readJson(manifestPath),
          name = entry.path.slice(entry.path.lastIndexOf("node_modules/") + 13);
        if (
          manifest.name !== name ||
          manifest.version !== entry.version ||
          manifest.license !== entry.license
        )
          throw new Error(`界面安装身份／版本／许可与锁不符：${entry.path}`);
        const base = noticeDirectory(dirname(manifestPath));
        const result = collectFiles(
          base,
          licenseFiles(base),
          `${name}@${entry.version}`,
          null,
          budget,
        );
        chunks.push(...result.chunks);
        return {
          ...source,
          name,
          ecosystem: "npm",
          manifestSha256: noticeText(directory, `${entry.path}/package.json`)
            .sha256,
          materialStatus: result.notices.length ? "collected" : "missing",
          notices: result.notices,
        };
      });
  return { entries, chunks };
}
export function collectRustNotices(metadata, registryValue) {
  const registry = noticeDirectory(registryValue),
    chunks = [],
    entries = [],
    sources = [],
    budget = { bytes: 0 };
  const closure = rustClosure(metadata);
  for (const entry of closure)
    if (
      entry.source !== null &&
      entry.source !== "registry+https://github.com/rust-lang/crates.io-index"
    )
      throw new Error(`Rust依赖来源尚不支持：${entry.name}`);
  for (const entry of closure) {
    if (entry.source === null) continue;
    if (
      entry.source !== "registry+https://github.com/rust-lang/crates.io-index"
    )
      throw new Error(`Rust依赖来源尚不支持：${entry.name}`);
    const inside = relative(registry, resolve(entry.manifest_path));
    if (
      !inside ||
      isAbsolute(inside) ||
      inside === ".." ||
      inside.startsWith(`..${sep}`)
    )
      throw new Error(`Rust注册包不在项目内缓存范围：${entry.name}`);
    const manifest = noticePath(registry, inside),
      base = noticeDirectory(dirname(manifest));
    const checksum = existsSync(`${base}/.cargo-checksum.json`)
      ? readJson(noticePath(base, ".cargo-checksum.json"))
      : null;
    if (
      checksum &&
      (!/^[a-f0-9]{64}$/.test(checksum.package) ||
        !checksum.files ||
        Array.isArray(checksum.files))
    )
      throw new Error(`Rust注册包校验清单无效：${entry.name}`);
    const manifestSha256 = noticeText(base, "Cargo.toml").sha256;
    if (checksum && checksum.files["Cargo.toml"] !== manifestSha256)
      throw new Error(`Rust注册包清单校验不匹配：${entry.name}`);
    const explicit = entry.license_file ? [entry.license_file] : [];
    const result = collectFiles(
      base,
      licenseFiles(base, explicit),
      `${entry.name}@${entry.version}`,
      checksum?.files,
      budget,
    );
    chunks.push(...result.chunks);
    entries.push({
      ecosystem: "cargo",
      name: entry.name,
      version: entry.version,
      license: entry.license ?? null,
      source: entry.source,
      manifestSha256,
      materialStatus: result.notices.length ? "collected" : "missing",
      notices: result.notices,
    });
    sources.push({
      name: entry.name,
      version: entry.version,
      source: entry.source,
      registryDirectory: inside.split(sep)[0],
      files: [
        { file: "Cargo.toml", sha256: manifestSha256 },
        ...result.notices,
      ],
    });
  }
  return { entries, chunks, sources };
}
