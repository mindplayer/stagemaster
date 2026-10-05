import { createHash } from "node:crypto";
import {
  constants,
  closeSync,
  fstatSync,
  lstatSync,
  openSync,
  readSync,
  writeFileSync,
} from "node:fs";
import { isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const project = resolve(fileURLToPath(new URL("../../", import.meta.url)));
export const noticeHash = (bytes) =>
  createHash("sha256").update(bytes).digest("hex");
export function noticeDirectory(value) {
  const directory = resolve(value),
    inside = relative(project, directory);
  if (
    !inside ||
    isAbsolute(inside) ||
    inside === ".." ||
    inside.startsWith(`..${sep}`)
  )
    throw new Error("告知目录必须在项目内且不能是项目根");
  let cursor = project;
  for (const part of inside.split(sep)) {
    cursor = join(cursor, part);
    const stat = lstatSync(cursor);
    if (stat.isSymbolicLink() || !stat.isDirectory())
      throw new Error(`告知目录链接或类型拒绝：${cursor}`);
  }
  return directory;
}
export function noticePath(root, key) {
  if (
    typeof key !== "string" ||
    !key ||
    key.includes("\\") ||
    /[\x00-\x1f\x7f]/.test(key) ||
    isAbsolute(key) ||
    key.split("/").some((part) => !part || part === "." || part === "..")
  )
    throw new Error(`告知路径无效：${key}`);
  const directory = noticeDirectory(root),
    parts = key.split("/");
  let cursor = directory;
  for (const [index, part] of parts.entries()) {
    cursor = join(cursor, part);
    const stat = lstatSync(cursor);
    if (stat.isSymbolicLink()) throw new Error(`告知来源链接拒绝：${cursor}`);
    if (index < parts.length - 1 ? !stat.isDirectory() : !stat.isFile())
      throw new Error(`告知文本不是文件：${cursor}`);
  }
  return cursor;
}
export function noticeText(root, key) {
  const file = noticePath(root, key),
    fd = openSync(
      file,
      constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK,
    );
  try {
    const stat = fstatSync(fd),
      size = stat.size;
    if (!stat.isFile()) throw new Error(`告知文本不是文件：${key}`);
    if (size > 512 * 1024) throw new Error(`告知单文本预算超限：${key}`);
    if (!size) throw new Error(`告知文本为空：${key}`);
    // One bounded extra byte detects growth without unbounded readFile allocation.
    const buffer = Buffer.alloc(size + 1);
    let length = 0;
    while (length < buffer.length) {
      const count = readSync(fd, buffer, length, buffer.length - length, null);
      if (!count) break;
      length += count;
    }
    if (length !== size) throw new Error(`告知文本读取中变化：${key}`);
    const bytes = buffer.subarray(0, length);
    let text;
    try {
      text = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(
        bytes,
      );
    } catch {
      throw new Error(`告知文本UTF-8无效：${key}`);
    }
    if (!text.trim() || text.includes("\0"))
      throw new Error(`告知文本为空白或包含NUL：${key}`);
    return { text, bytes: bytes.length, sha256: noticeHash(bytes) };
  } finally {
    closeSync(fd);
  }
}

export function saveNoticeFiles(bundle, report, text) {
  const directory = noticeDirectory(join(bundle, "licenses"));
  const targets = ["THIRD-PARTY-NOTICES.txt", "notices.json"].map((file) =>
    join(directory, file),
  );
  for (const target of targets) {
    try {
      lstatSync(target);
    } catch (error) {
      if (error.code === "ENOENT") continue;
      throw error;
    }
    throw new Error(`告知目标已存在：${target}`);
  }
  const handles = [];
  try {
    // Reserve both paths exclusively before writing; partial failure never returns success.
    for (const target of targets)
      handles.push(
        openSync(
          target,
          constants.O_CREAT |
            constants.O_EXCL |
            constants.O_WRONLY |
            constants.O_NOFOLLOW,
          0o644,
        ),
      );
    writeFileSync(handles[0], text);
    writeFileSync(handles[1], JSON.stringify(report, null, 2) + "\n");
  } finally {
    for (const fd of handles) closeSync(fd);
  }
}
