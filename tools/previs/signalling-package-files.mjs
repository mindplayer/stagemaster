import { createHash } from "node:crypto";
import {
  createReadStream,
  readFileSync,
  readdirSync,
  realpathSync,
  statSync,
} from "node:fs";
import { isAbsolute, join, normalize, relative, sep } from "node:path";
import { readMacImage } from "./mac-bundle-inspection.mjs";
import { parseStrict } from "../project-format/check.mjs";
import { isDeepStrictEqual } from "node:util";

export const readJson = (file) =>
  parseStrict(readFileSync(file, "utf8").replace(/^\uFEFF/, ""));

export async function fileHash(file) {
  const digest = createHash("sha256");
  for await (const chunk of createReadStream(file)) digest.update(chunk);
  return digest.digest("hex");
}

export function lockedPackages(manifest, lock) {
  if (
    !manifest.dependencies ||
    Object.keys(manifest.dependencies).length === 0 ||
    lock.lockfileVersion !== 3 ||
    !lock.packages?.[""] ||
    !isDeepStrictEqual(manifest.dependencies, lock.packages[""].dependencies)
  )
    throw new Error("信令依赖清单与锁不一致");
  return Object.entries(lock.packages)
    .filter(([key, entry]) => key !== "" && !entry.dev)
    .map(([key, entry]) => {
      const pieces = key.split("/");
      if (
        !key.startsWith("node_modules/") ||
        isAbsolute(key) ||
        normalize(key) !== key ||
        pieces.some((piece) => !piece || piece === "." || piece === "..") ||
        entry.link ||
        typeof entry.version !== "string" ||
        !entry.version ||
        typeof entry.license !== "string" ||
        typeof entry.integrity !== "string" ||
        !entry.integrity
      )
        throw new Error(`信令锁定包元数据无效：${key}`);
      const url = new URL(entry.resolved);
      if (
        url.protocol !== "https:" ||
        url.hostname !== "registry.npmjs.org" ||
        url.username ||
        url.password
      )
        throw new Error(`信令依赖不是锁定的官方 npm 发布包：${key}`);
      return {
        path: key,
        version: entry.version,
        license: entry.license,
        integrity: entry.integrity,
        resolved: entry.resolved,
      };
    });
}

const contained = (root, file) => {
  const path = relative(root, realpathSync(file));
  if (path === ".." || path.startsWith(`..${sep}`) || isAbsolute(path))
    throw new Error(`组件链接越界：${file}`);
};

export function packageLicenses(bundle, packages) {
  const root = realpathSync(bundle);
  return packages.map((entry) => {
    const directory = join(bundle, entry.path);
    contained(root, directory);
    const manifestPath = join(directory, "package.json");
    contained(root, manifestPath);
    const manifest = readJson(manifestPath);
    if (manifest.version !== entry.version)
      throw new Error(`信令包版本不匹配：${entry.path}`);
    const files = readdirSync(directory)
      .filter((file) => /^(license|copying|notice)(\.|$)/i.test(file))
      .map((file) => {
        contained(root, join(directory, file));
        if (!statSync(join(directory, file)).isFile())
          throw new Error(`许可不是文件：${file}`);
        return join(entry.path, file);
      });
    return {
      ...entry,
      name: manifest.name,
      licenseFiles: files,
      needsLicenseReview: files.length === 0,
    };
  });
}

export function qualifyNode(root, file) {
  const image = readMacImage(root, file);
  if (
    !image.executable ||
    image.dylib ||
    image.dependencies.some((dependency) => {
      const name = normalize(dependency.name);
      return (
        !name.startsWith("/System/Library/") && !name.startsWith("/usr/lib/")
      );
    })
  )
    throw new Error("Node 不是 ARM64 自包含系统库运行时");
  return image;
}

export async function fileInventory(directory) {
  const root = realpathSync(directory);
  const files = [];
  const visit = async (folder) => {
    for (const entry of readdirSync(folder, { withFileTypes: true }).sort(
      (a, b) => a.name.localeCompare(b.name),
    )) {
      const file = join(folder, entry.name);
      contained(root, file);
      if (entry.isDirectory()) await visit(file);
      else if (entry.isFile() || entry.isSymbolicLink()) {
        if (!statSync(file).isFile() || file.endsWith(".node"))
          throw new Error(`组件不支持此文件类型：${file}`);
        files.push({
          path: relative(root, file),
          bytes: statSync(file).size,
          sha256: await fileHash(file),
        });
      } else throw new Error(`组件特殊文件拒绝：${file}`);
    }
  };
  await visit(directory);
  return files;
}
