import {
  constants,
  openSync,
  closeSync,
  fstatSync,
  lstatSync,
  readSync,
  realpathSync,
} from "node:fs";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseStrict } from "../project-format/check.mjs";
import { plainAncestors } from "../previs/desktop-assembly-files.mjs";
import { noticeHash } from "../previs/signalling-notices-files.mjs";
const project = resolve(fileURLToPath(new URL("../../", import.meta.url)));
export function handoffRoot(value) {
  const root = resolve(value);
  if (root !== project && !root.startsWith(project + "/tmp/desktop-011-test-"))
    throw new Error("交接项目根路径无效");
  plainAncestors(root);
  if (realpathSync(root) !== root) throw new Error("交接项目根链接拒绝");
  return root;
}
export function handoffPath(root, value) {
  if (typeof value !== "string" || !value || /[\\\x00-\x1f\x7f]/.test(value))
    throw new Error("交接路径无效");
  const key = isAbsolute(value) ? relative(root, value) : value;
  if (
    !key ||
    key.split("/").some((part) => !part || part === "." || part === "..") ||
    key.split("/")[0] === "output"
  )
    throw new Error("交接路径必须在项目内且不能访问output");
  return join(root, key);
}
export function handoffTarget(root, value) {
  const output = handoffPath(root, value),
    key = relative(root, output);
  if (!/^data\/DESKTOP-011\/[^/]{1,64}$/u.test(key))
    throw new Error("交接目标必须是本任务全新目录");
  plainAncestors(dirname(output));
  if (lstatSync(output, { throwIfNoEntry: false }))
    throw new Error("交接目标已存在，不覆盖");
  return output;
}
export function handoffBytes(root, value, limit = 8 * 1024 * 1024) {
  const file = handoffPath(root, value);
  plainAncestors(dirname(file));
  const stat = lstatSync(file);
  if (stat.isSymbolicLink() || !stat.isFile())
    throw new Error("交接输入链接或文件类型拒绝");
  const fd = openSync(
    file,
    constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK,
  );
  try {
    const size = fstatSync(fd).size;
    if (!size || size > limit) throw new Error("交接输入为空或预算超限");
    const buffer = Buffer.alloc(size + 1);
    let length = 0;
    while (length < buffer.length) {
      const count = readSync(fd, buffer, length, buffer.length - length, null);
      if (!count) break;
      length += count;
    }
    if (length !== size) throw new Error("交接输入读取中变化");
    const bytes = buffer.subarray(0, length);
    const text = new TextDecoder("utf-8", {
      fatal: true,
      ignoreBOM: true,
    }).decode(bytes);
    if (text.includes("\0")) throw new Error("交接文本包含NUL");
    return { path: file, bytes, sha256: noticeHash(bytes), text };
  } finally {
    closeSync(fd);
  }
}
export function handoffJson(root, value) {
  const input = handoffBytes(root, value);
  return { ...input, value: parseStrict(input.text) };
}
export const digest = (value) =>
  typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
