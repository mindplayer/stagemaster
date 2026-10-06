import { createHash } from "node:crypto";
import { createReadStream, lstatSync } from "node:fs";
import { opendir } from "node:fs/promises";
import { dirname, join } from "node:path";
import {
  sourceRoots,
  sourceExtras,
  sourcePath,
  sourceLimits,
} from "./source-scope.mjs";
import { plainAncestors } from "../previs/plain-ancestors.mjs";

export async function sourceFiles(root, objectFormat, limits = sourceLimits) {
  let entries = 0,
    total = 0;
  const names = [];
  const add = (path) => {
    if (names.length >= limits.files) throw Error("构建来源文件数超过上限");
    names.push(path);
  };
  for (const folder of sourceRoots) {
    plainAncestors(join(root, folder));
    const pending = [folder];
    while (pending.length) {
      const directory = pending.pop();
      const handle = await opendir(join(root, directory), { bufferSize: 64 });
      for await (const entry of handle) {
        if (++entries > limits.entries) throw Error("构建来源目录扫描超过上限");
        const path = directory + "/" + entry.name;
        if (!sourcePath(path)) continue;
        if (entry.isSymbolicLink()) throw Error("构建来源不接受链接：" + path);
        if (entry.isDirectory()) pending.push(path);
        else if (entry.isFile()) add(path);
        else throw Error("构建来源不是普通文件：" + path);
      }
    }
  }
  for (const file of sourceExtras) {
    try {
      lstatSync(join(root, file));
      add(file);
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
    }
  }
  const rows = [];
  for (const path of names.sort()) {
    const full = join(root, path);
    plainAncestors(dirname(full));
    const stat = lstatSync(full);
    if (!stat.isFile() || stat.isSymbolicLink())
      throw Error("构建来源不是普通文件：" + path);
    if (stat.size > limits.fileBytes || total + stat.size > limits.totalBytes)
      throw Error("构建来源字节超过上限：" + path);
    const hash = createHash("sha256"),
      blob = createHash(objectFormat);
    blob.update(Buffer.from(`blob ${stat.size}\0`));
    let bytes = 0;
    for await (const chunk of createReadStream(full, {
      highWaterMark: 64 * 1024,
    })) {
      bytes += chunk.length;
      if (bytes > limits.fileBytes || total + bytes > limits.totalBytes)
        throw Error("构建来源读取超过上限：" + path);
      hash.update(chunk);
      blob.update(chunk);
    }
    if (bytes !== stat.size) throw Error("来源读取期间文件长度变化：" + path);
    total += bytes;
    rows.push({
      path,
      bytes,
      sha256: hash.digest("hex"),
      executable: !!(stat.mode & 0o111),
      gitBlob: blob.digest("hex"),
    });
  }
  return { rows, total };
}
