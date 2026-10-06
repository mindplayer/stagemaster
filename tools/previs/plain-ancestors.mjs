import { lstatSync, realpathSync } from "node:fs";
import { dirname, resolve } from "node:path";

export function plainAncestors(file) {
  for (let path = resolve(file); ; path = dirname(path)) {
    let stat;
    try {
      stat = lstatSync(path, { throwIfNoEntry: false });
    } catch (cause) {
      throw new Error(`目录无效，无法检查：${path}`, { cause });
    }
    if (stat && (!stat.isDirectory() || realpathSync(path) !== path))
      throw new Error(`目录无效或经过链接，写入前拒绝：${path}`);
    if (path === dirname(path)) break;
  }
}
